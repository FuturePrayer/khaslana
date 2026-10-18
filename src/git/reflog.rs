//! 直接读取 files 引用后端的本地日志；不能用会在缺失时创建日志的 libgit2 reflog API。

use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use git2::{BranchType, ErrorCode, Reference, ReferenceType, Repository};

use super::GitService;
use crate::types::{
    GitError, RecoveryRepoIdentity, ReflogEntry, ReflogPage, ReflogQuery, ReflogScope,
    ReflogSnapshot, ReflogSources, Result,
};

const RAW_BYTE_LIMIT: u64 = 64 * 1024 * 1024;
const ENTRY_LIMIT: usize = 200_000;
const LINE_BYTE_LIMIT: usize = 256 * 1024;
const DIAGNOSTIC_LIMIT: usize = 1000;
const PAGE_LIMIT: usize = 100;

#[derive(Clone, Copy)]
struct ReadLimits {
    raw_bytes: u64,
    entries: usize,
    line_bytes: usize,
}

impl Default for ReadLimits {
    fn default() -> Self {
        Self {
            raw_bytes: RAW_BYTE_LIMIT,
            entries: ENTRY_LIMIT,
            line_bytes: LINE_BYTE_LIMIT,
        }
    }
}

#[derive(Default)]
struct ReadBudget {
    raw_bytes: u64,
    entries: usize,
}

#[derive(Default)]
struct Diagnostics {
    messages: Vec<String>,
    omitted: usize,
}

impl Diagnostics {
    fn push(&mut self, message: String) {
        // 坏行也占原始预算，但不能让大量短坏行生成无界的诊断字符串。
        if self.messages.len() < DIAGNOSTIC_LIMIT {
            self.messages.push(message);
        } else {
            self.omitted += 1;
        }
    }

    fn finish(mut self, sources_read: usize, sources_total: usize) -> Vec<String> {
        if self.omitted > 0 {
            self.messages
                .push(format!("另有 {} 条日志诊断未逐条展示", self.omitted));
        }
        if sources_read < sources_total {
            self.messages.push(format!(
                "日志覆盖不完整：成功读取 {sources_read}/{sources_total} 个来源；缺失或不可读的来源未计入结果"
            ));
        }
        self.messages
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LogSource {
    ref_name: String,
    path: PathBuf,
    root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    changed: (i64, i64),
}

impl FileStamp {
    fn from_metadata(metadata: &Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
            #[cfg(unix)]
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Observation {
    Missing,
    Ready { path: PathBuf, stamp: FileStamp },
    Unavailable(String),
}

#[derive(Debug)]
enum ReadFailure {
    Source(io::Error),
    Fatal(GitError),
}

impl GitService {
    /// 来源仅包含当前仓库已有的本地分支；不扫描历史遗留的日志路径。
    pub fn reflog_sources(&self, repo: &Repository) -> Result<ReflogSources> {
        let identity = self.recovery_repo_identity(repo)?;
        require_files_backend(repo)?;
        Ok(ReflogSources {
            identity,
            current_branch: current_branch(repo)?,
            branches: local_branches(repo, None)?,
        })
    }

    /// 快照内包含日志证据，不要求两侧对象仍存在，也不按 OID 合并不同来源。
    pub fn read_reflog_snapshot(
        &self,
        repo: &Repository,
        scope: &ReflogScope,
        cancelled: &AtomicBool,
    ) -> Result<ReflogSnapshot> {
        check_cancelled(cancelled)?;
        let identity = self.recovery_repo_identity(repo)?;
        require_files_backend(repo)?;
        for _ in 0..2 {
            check_cancelled(cancelled)?;
            if let Some(snapshot) =
                read_snapshot_attempt(repo, &identity, scope, cancelled, ReadLimits::default())?
            {
                return Ok(snapshot);
            }
        }
        Err(GitError::Message(
            "读取期间 Reflog 日志持续变化，无法生成一致快照，请重试".into(),
        ))
    }
}

impl ReflogSnapshot {
    /// offset 只在当前快照及当前筛选条件内有效；只有本页条目会被克隆。
    pub fn query(&self, query: &ReflogQuery, offset: usize, limit: usize) -> ReflogPage {
        let limit = limit.min(PAGE_LIMIT);
        let search = query.search.to_lowercase();
        let mut entries = Vec::with_capacity(limit);
        let mut total = 0;
        for entry in &self.entries {
            if query.since.is_some_and(|since| entry.timestamp < since)
                || (!search.is_empty()
                    && ![
                        entry.message.as_str(),
                        entry.ref_name.as_str(),
                        entry.actor.as_str(),
                        entry.actor_email.as_str(),
                        entry.old_oid.as_str(),
                        entry.new_oid.as_str(),
                    ]
                    .iter()
                    .any(|field| field.to_lowercase().contains(&search)))
            {
                continue;
            }
            if total >= offset && entries.len() < limit {
                entries.push(entry.clone());
            }
            total += 1;
        }
        let end = offset.saturating_add(entries.len());
        let next_offset = if !entries.is_empty() && end < total {
            Some(end)
        } else {
            None
        };
        ReflogPage {
            entries,
            total,
            next_offset,
        }
    }
}

fn require_files_backend(repo: &Repository) -> Result<()> {
    match repo.config()?.get_string("extensions.refStorage") {
        Ok(backend) if backend.eq_ignore_ascii_case("files") => Ok(()),
        Ok(backend) => Err(GitError::Message(format!(
            "不支持引用后端 {backend}：本地 Reflog 目前仅支持 files 引用后端"
        ))),
        Err(error) if error.code() == ErrorCode::NotFound => Ok(()),
        Err(error) => Err(GitError::Message(format!(
            "无法确认引用后端，未读取本地 Reflog：{error}"
        ))),
    }
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(GitError::Message("Reflog 读取已取消".into()))
    } else {
        Ok(())
    }
}

fn branch_ref(name: &str) -> Result<String> {
    let full_name = format!("refs/heads/{name}");
    if name.is_empty()
        || name.contains('\\')
        || !Reference::is_valid_name(&full_name)
        || Path::new(name)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(GitError::InvalidBranchName(name.into()));
    }
    Ok(full_name)
}

fn current_branch(repo: &Repository) -> Result<Option<String>> {
    let head = match repo.find_reference("HEAD") {
        Ok(head) => head,
        Err(error) if error.code() == ErrorCode::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if head.kind() != Some(ReferenceType::Symbolic) {
        return Ok(None);
    }
    let target = head
        .symbolic_target()
        .map_err(|error| GitError::Message(format!("HEAD 的符号引用名不可读：{error}")))?
        .ok_or_else(|| GitError::Message("HEAD 缺少符号引用目标".into()))?;
    let Some(name) = target.strip_prefix("refs/heads/") else {
        return Ok(None);
    };
    branch_ref(name)?;
    Ok(Some(name.into()))
}

fn local_branches(repo: &Repository, cancelled: Option<&AtomicBool>) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        if let Some(cancelled) = cancelled {
            check_cancelled(cancelled)?;
        }
        let (branch, _) = branch?;
        let full_name = branch
            .get()
            .name()
            .map_err(|error| GitError::Message(format!("本地分支引用名不可读：{error}")))?;
        let name = full_name
            .strip_prefix("refs/heads/")
            .ok_or_else(|| GitError::Message(format!("不是本地分支引用：{full_name}")))?;
        if branch_ref(name)? != full_name {
            return Err(GitError::InvalidBranchName(name.into()));
        }
        names.push(name.to_owned());
    }
    names.sort();
    Ok(names)
}

fn select_sources(
    repo: &Repository,
    identity: &RecoveryRepoIdentity,
    scope: &ReflogScope,
    cancelled: &AtomicBool,
    diagnostics: &mut Diagnostics,
) -> Result<Vec<LogSource>> {
    check_cancelled(cancelled)?;
    let head = || LogSource {
        ref_name: "HEAD".into(),
        path: identity.git_dir.join("logs").join("HEAD"),
        root: identity.git_dir.clone(),
    };
    let branch = |name: &str| -> Result<LogSource> {
        let ref_name = branch_ref(name)?;
        Ok(LogSource {
            path: identity.common_dir.join("logs").join(&ref_name),
            root: identity.common_dir.clone(),
            ref_name,
        })
    };
    match scope {
        ReflogScope::Head => Ok(vec![head()]),
        ReflogScope::CurrentBranch => match current_branch(repo)? {
            Some(name) => Ok(vec![branch(&name)?]),
            None => {
                diagnostics.push("当前 HEAD 未指向本地分支，没有当前分支日志来源".into());
                Ok(Vec::new())
            }
        },
        ReflogScope::LocalBranch(name) => {
            branch_ref(name)?;
            repo.find_branch(name, BranchType::Local)
                .map_err(|error| GitError::Message(format!("无法读取本地分支 {name}：{error}")))?;
            Ok(vec![branch(name)?])
        }
        ReflogScope::All => {
            let mut sources = vec![head()];
            for name in local_branches(repo, Some(cancelled))? {
                sources.push(branch(&name)?);
            }
            Ok(sources)
        }
    }
}

fn observe_source(source: &LogSource) -> Result<Observation> {
    let path = match fs::canonicalize(&source.path) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Observation::Missing),
        Err(error) => return Ok(Observation::Unavailable(error.to_string())),
    };
    // 校验词法路径之外也核对现有文件的真实位置，阻止日志目录中的符号链接逃逸。
    let root = fs::canonicalize(&source.root)?;
    if !path.starts_with(&root) {
        return Err(GitError::Message(format!(
            "日志路径超出仓库目录，拒绝读取：{}",
            source.ref_name
        )));
    }
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(Observation::Ready {
            path,
            stamp: FileStamp::from_metadata(&metadata),
        }),
        Ok(_) => Ok(Observation::Unavailable("日志路径不是普通文件".into())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Observation::Missing),
        Err(error) => Ok(Observation::Unavailable(error.to_string())),
    }
}

fn read_snapshot_attempt(
    repo: &Repository,
    identity: &RecoveryRepoIdentity,
    scope: &ReflogScope,
    cancelled: &AtomicBool,
    limits: ReadLimits,
) -> Result<Option<ReflogSnapshot>> {
    let mut diagnostics = Diagnostics::default();
    let sources = select_sources(repo, identity, scope, cancelled, &mut diagnostics)?;
    let mut observations = Vec::with_capacity(sources.len());
    for source in &sources {
        check_cancelled(cancelled)?;
        observations.push(observe_source(source)?);
    }
    let mut entries = Vec::new();
    let mut sources_read = 0;
    let mut budget = ReadBudget::default();
    let mut source_error = None;
    for (source, before) in sources.iter().zip(&observations) {
        check_cancelled(cancelled)?;
        let failure = match before {
            Observation::Missing => {
                diagnostics.push(format!(
                    "{}：日志不存在，该来源未覆盖；只读服务未创建文件",
                    source.ref_name
                ));
                continue;
            }
            Observation::Unavailable(reason) => Some(reason.clone()),
            Observation::Ready { path, stamp } => {
                match read_source(
                    path,
                    source,
                    stamp,
                    cancelled,
                    limits,
                    &mut budget,
                    &mut diagnostics,
                ) {
                    Ok(Some(mut source_entries)) => {
                        sources_read += 1;
                        entries.append(&mut source_entries);
                        None
                    }
                    Ok(None) => return Ok(None),
                    Err(ReadFailure::Fatal(error)) => return Err(error),
                    Err(ReadFailure::Source(error)) => Some(error.to_string()),
                }
            }
        };
        if let Some(reason) = failure {
            let message = format!("{}：日志不可读，该来源未覆盖：{reason}", source.ref_name);
            if matches!(scope, ReflogScope::All) {
                diagnostics.push(message);
            } else {
                source_error = Some(GitError::Message(message));
            }
        }
    }
    // 所有来源都在读取前后检查，能捕获读取较早的文件在读取其它来源期间变化。
    for (source, before) in sources.iter().zip(&observations) {
        check_cancelled(cancelled)?;
        if observe_source(source)? != *before {
            return Ok(None);
        }
    }
    let mut ignored_diagnostics = Diagnostics::default();
    if select_sources(repo, identity, scope, cancelled, &mut ignored_diagnostics)? != sources {
        return Ok(None);
    }
    if let Some(error) = source_error {
        return Err(error);
    }
    check_cancelled(cancelled)?;
    // sort_by 保持同一引用、同一时间的原文件行序，不把相同 OID 的证据合并。
    entries.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| left.ref_name.cmp(&right.ref_name))
    });
    check_cancelled(cancelled)?;
    Ok(Some(ReflogSnapshot {
        identity: identity.clone(),
        entries,
        diagnostics: diagnostics.finish(sources_read, sources.len()),
        sources_read,
        sources_total: sources.len(),
    }))
}

fn read_source(
    path: &Path,
    source: &LogSource,
    before: &FileStamp,
    cancelled: &AtomicBool,
    limits: ReadLimits,
    budget: &mut ReadBudget,
    diagnostics: &mut Diagnostics,
) -> std::result::Result<Option<Vec<ReflogEntry>>, ReadFailure> {
    // File::open 只申请读取权限；缺失时绝不会创建文件或父目录。
    let mut file = File::open(path).map_err(ReadFailure::Source)?;
    let metadata = file.metadata().map_err(ReadFailure::Source)?;
    if !metadata.is_file() || FileStamp::from_metadata(&metadata) != *before {
        return Ok(None);
    }
    if before.len > limits.raw_bytes.saturating_sub(budget.raw_bytes) {
        return Err(ReadFailure::Fatal(raw_budget_error()));
    }
    let entries = read_records(
        &mut file,
        &source.ref_name,
        cancelled,
        limits,
        budget,
        diagnostics,
    )?;
    let after = file.metadata().map_err(ReadFailure::Source)?;
    if FileStamp::from_metadata(&after) != *before {
        return Ok(None);
    }
    if before.len == 0 {
        diagnostics.push(format!("{}：日志为空", source.ref_name));
    }
    Ok(Some(entries))
}

fn raw_budget_error() -> GitError {
    GitError::Message("Reflog 原始日志累计超过 64 MiB，未生成部分快照".into())
}

fn read_records(
    reader: &mut impl Read,
    ref_name: &str,
    cancelled: &AtomicBool,
    limits: ReadLimits,
    budget: &mut ReadBudget,
    diagnostics: &mut Diagnostics,
) -> std::result::Result<Vec<ReflogEntry>, ReadFailure> {
    let mut entries = Vec::new();
    let mut buffer = [0u8; 16 * 1024];
    let mut line = Vec::new();
    let mut line_number = 1u64;
    let mut overlong = false;
    loop {
        check_cancelled(cancelled).map_err(ReadFailure::Fatal)?;
        let count = match reader.read(&mut buffer) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ReadFailure::Source(error)),
        };
        if count == 0 {
            break;
        }
        budget.raw_bytes = budget.raw_bytes.saturating_add(count as u64);
        if budget.raw_bytes > limits.raw_bytes {
            return Err(ReadFailure::Fatal(raw_budget_error()));
        }
        for segment in buffer[..count].split_inclusive(|byte| *byte == b'\n') {
            let complete = segment.last() == Some(&b'\n');
            let content = if complete {
                &segment[..segment.len() - 1]
            } else {
                segment
            };
            if !overlong {
                if content.len() > limits.line_bytes.saturating_sub(line.len()) {
                    diagnostics.push(format!(
                        "{ref_name} 第 {line_number} 行超过单行 256 KiB 上限，已跳过"
                    ));
                    line.clear();
                    overlong = true;
                } else {
                    line.extend_from_slice(content);
                }
            }
            if complete {
                if !overlong {
                    match parse_record(&line, ref_name, line_number) {
                        Ok(entry) => {
                            budget.entries += 1;
                            if budget.entries > limits.entries {
                                return Err(ReadFailure::Fatal(GitError::Message(
                                    "Reflog 有效条目累计超过 200000 条，未生成部分快照".into(),
                                )));
                            }
                            if entry.lossy {
                                diagnostics.push(format!(
                                    "{ref_name} 第 {line_number} 行含非 UTF-8 文本，已使用替换字符显示"
                                ));
                            }
                            entries.push(entry);
                        }
                        Err(reason) => diagnostics.push(format!(
                            "{ref_name} 第 {line_number} 行格式无效，已跳过：{reason}"
                        )),
                    }
                }
                line.clear();
                overlong = false;
                line_number += 1;
            }
        }
    }
    check_cancelled(cancelled).map_err(ReadFailure::Fatal)?;
    if !line.is_empty() || overlong {
        diagnostics.push(format!(
            "{ref_name} 第 {line_number} 行不完整（缺少行结束符），已跳过"
        ));
    }
    Ok(entries)
}

fn parse_record(
    line: &[u8],
    ref_name: &str,
    line_number: u64,
) -> std::result::Result<ReflogEntry, &'static str> {
    let (header, message) = match line.iter().position(|byte| *byte == b'\t') {
        Some(tab) => (&line[..tab], &line[tab + 1..]),
        None => (line, &[][..]),
    };
    if header.len() < 83 || header.get(40) != Some(&b' ') || header.get(81) != Some(&b' ') {
        return Err("缺少完整的两侧 OID");
    }
    let old_oid = parse_oid(&header[..40])?;
    let new_oid = parse_oid(&header[41..81])?;
    let mut actor_and_time = header[82..].rsplitn(3, |byte| *byte == b' ');
    let timezone = actor_and_time.next().ok_or("缺少时区")?;
    let timestamp = actor_and_time.next().ok_or("缺少时间戳")?;
    let actor_and_email = actor_and_time.next().ok_or("缺少操作者")?;
    let timezone_offset = parse_timezone(timezone)?;
    let timestamp = std::str::from_utf8(timestamp)
        .ok()
        .and_then(|timestamp| timestamp.parse::<i64>().ok())
        .ok_or("时间戳无效或超出范围")?;
    if actor_and_email.last() != Some(&b'>') {
        return Err("操作者邮箱格式无效");
    }
    let email_start = actor_and_email
        .iter()
        .rposition(|byte| *byte == b'<')
        .ok_or("操作者邮箱格式无效")?;
    if email_start == 0 || actor_and_email[email_start - 1] != b' ' {
        return Err("操作者姓名与邮箱分隔无效");
    }
    let actor = &actor_and_email[..email_start - 1];
    let actor_email = &actor_and_email[email_start + 1..actor_and_email.len() - 1];
    let lossy = [actor, actor_email, message]
        .iter()
        .any(|text| std::str::from_utf8(text).is_err());
    Ok(ReflogEntry {
        id: format!("{ref_name}:{line_number}"),
        ref_name: ref_name.into(),
        old_oid,
        new_oid,
        actor: String::from_utf8_lossy(actor).into_owned(),
        actor_email: String::from_utf8_lossy(actor_email).into_owned(),
        timestamp,
        timezone_offset,
        message: String::from_utf8_lossy(message).into_owned(),
        lossy,
    })
}

fn parse_oid(bytes: &[u8]) -> std::result::Result<String, &'static str> {
    if bytes.len() != 40 || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return Err("OID 必须是完整的 40 位十六进制值");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "OID 含非 ASCII 字节")?;
    git2::Oid::from_str(text)
        .map(|oid| oid.to_string())
        .map_err(|_| "OID 无效")
}

fn parse_timezone(bytes: &[u8]) -> std::result::Result<i32, &'static str> {
    if bytes.len() != 5
        || !matches!(bytes[0], b'+' | b'-')
        || !bytes[1..].iter().all(u8::is_ascii_digit)
    {
        return Err("时区必须为 +/-HHMM");
    }
    let hours = i32::from(bytes[1] - b'0') * 10 + i32::from(bytes[2] - b'0');
    let minutes = i32::from(bytes[3] - b'0') * 10 + i32::from(bytes[4] - b'0');
    if hours > 23 || minutes > 59 {
        return Err("时区超出有效范围");
    }
    let offset = hours * 60 + minutes;
    Ok(if bytes[0] == b'-' { -offset } else { offset })
}

#[cfg(test)]
#[path = "../tests/git/reflog.rs"]
mod tests;
