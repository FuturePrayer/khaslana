use crate::types::domain_ui_resources;
use std::fs;
use std::path::Path;
use std::str;

use git2::build::CheckoutBuilder;
use git2::{ErrorCode, MergeFileOptions, Repository};

#[path = "conflicts/side_ranges.rs"]
mod side_ranges;

use super::{GitService, ensure_worktree_relative_path, path_to_git, remove_worktree_path};
use crate::{
    ConflictBlock, ConflictBlockStatus, ConflictDraftStatus, ConflictFileKind, ConflictFileView,
    ConflictResolutionSide, ExternalMergeSettings, GitError, OperationEvent, RepositorySnapshot,
    Result, external_merge,
};

impl GitService {
    pub fn conflict_file_view(&self, repo: &Repository, path: &Path) -> Result<ConflictFileView> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_READ_CONFLICT_DETAILS))?;
        let git_path = path_to_git(path);

        if repo
            .index()?
            .conflict_get(path)
            .is_err_and(|err| err.code() == ErrorCode::NotFound)
        {
            let workdir = repo
                .workdir()
                .ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::A_BARE_REPOSITORY_HAS_NO_WORKING_TREE, &[])))?;
            let content = fs::read_to_string(workdir.join(path))?;
            if !super::stash::has_complete_conflict_markers(&content) {
                return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THIS_FILE_HAS_NO_CONFLICTS, &[("git_path", &format!("{}", git_path))])));
            }
            return Ok(ConflictFileView {
                path: git_path,
                kind: ConflictFileKind::WorktreeOnly,
                draft: String::new(),
                ours_text: String::new(),
                theirs_text: String::new(),
                blocks: Vec::new(),
                draft_status: ConflictDraftStatus::Clean,
                fallback_reason: Some(
                    crate::ui_i18n::text(domain_ui_resources::RESTORING_THE_STASH_WROTE_CONFLICT_MARKERS_TO_THE_WORKING_TREE_BUT_THE_GIT_INDEX_HAS).into(),
                ),
            });
        }

        match diff3_merge_text(repo, path)? {
            ConflictMergeText::Unsupported => Ok(ConflictFileView {
                path: git_path,
                kind: ConflictFileKind::Unsupported,
                draft: String::new(),
                ours_text: String::new(),
                theirs_text: String::new(),
                blocks: Vec::new(),
                draft_status: ConflictDraftStatus::Clean,
                fallback_reason: Some(crate::ui_i18n::text(domain_ui_resources::THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_USE_THE_QUICK_RESOLUTION_BUTTONS).into()),
            }),
            ConflictMergeText::Binary => Ok(ConflictFileView {
                path: git_path,
                kind: ConflictFileKind::Binary,
                draft: String::new(),
                ours_text: String::new(),
                theirs_text: String::new(),
                blocks: Vec::new(),
                draft_status: ConflictDraftStatus::Clean,
                fallback_reason: Some(crate::ui_i18n::text(domain_ui_resources::THE_TEXT_MERGE_EDITOR_CANNOT_OPEN_THIS_CONFLICT_FILE).into()),
            }),
            ConflictMergeText::Text(merged_bytes) => {
                let merged_text = str::from_utf8(&merged_bytes).map_err(|_| {
                    GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_THE_VISUAL_EDITOR_IS_UNAVAILABLE, &[]))
                })?;
                let (draft, projected_ours, projected_theirs, mut blocks) =
                    parse_diff3_conflict_text(merged_text)?;

                // diff3 的上下文已经合入两侧非冲突修改，不能冒充原始版本。
                // 三栏必须从索引 blob 读取原文，再以 diff 行映射定位各冲突块。
                let conflict = conflict_for_path(&repo.index()?, path)?;
                let read_side = |entry: Option<&git2::IndexEntry>| -> Result<String> {
                    let entry = entry.ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_CONFLICT_STAGE_IS_NO_LONGER_VALID, &[])))?;
                    let blob = repo.find_blob(entry.id)?;
                    str::from_utf8(blob.content())
                        .map(str::to_owned)
                        .map_err(|_| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_CONFLICT_VERSION_IS_NOT_UTF_8_TEXT, &[])))
                };
                let ours_text = read_side(conflict.our.as_ref())?;
                let theirs_text = read_side(conflict.their.as_ref())?;
                side_ranges::map_side_ranges(&projected_ours, &ours_text, &mut blocks, true)?;
                side_ranges::map_side_ranges(&projected_theirs, &theirs_text, &mut blocks, false)?;

                Ok(ConflictFileView {
                    path: git_path,
                    kind: ConflictFileKind::Text,
                    draft,
                    ours_text,
                    theirs_text,
                    blocks,
                    draft_status: ConflictDraftStatus::Clean,
                    fallback_reason: None,
                })
            }
        }
    }

    /// 读取冲突文件的 diff3 原始文本（带 OURS/BASE/THEIRS 标记），
    /// 供 AI 合并建议构造 prompt 使用。
    pub fn conflict_diff3_text(&self, repo: &Repository, path: &Path) -> Result<String> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_GENERATE_AI_MERGE_SUGGESTIONS))?;
        match diff3_merge_text(repo, path)? {
            ConflictMergeText::Unsupported => Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE, &[]))),
            ConflictMergeText::Binary => Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::BINARY_CONFLICT_FILES_DO_NOT_SUPPORT_AI_MERGE_SUGGESTIONS_USE_THE_QUICK_RESOLUTION_BU, &[]))),
            ConflictMergeText::Text(bytes) => {
                str::from_utf8(&bytes).map(str::to_string).map_err(|_| {
                    GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE, &[]))
                })
            }
        }
    }

    pub fn apply_conflict_draft(
        &self,
        repo: &mut Repository,
        path: &Path,
        draft: &str,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_APPLY_THE_CONFLICT_DRAFT))?;
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::text(domain_ui_resources::APPLYING_CONFLICT_DRAFT).into()));
        conflict_for_path(&repo.index()?, path)?;
        write_conflict_draft(repo, path, draft)?;
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::text(domain_ui_resources::CONFLICT_DRAFT_APPLIED_TO_THE_WORKING_TREE).into()));
        self.snapshot_after_operation(repo)
    }

    pub fn apply_conflict_draft_and_resolve(
        &self,
        repo: &mut Repository,
        path: &Path,
        draft: &str,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_APPLY_AND_RESOLVE_THE_CONFLICT))?;
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::text(domain_ui_resources::APPLYING_RESULT_AND_MARKING_CONFLICT_AS_RESOLVED).into()));
        conflict_for_path(&repo.index()?, path)?;
        write_conflict_draft(repo, path, draft)?;
        let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::text(domain_ui_resources::CONFLICT_RESULT_APPLIED_AND_MARKED_AS_RESOLVED).into()));
        Ok(snapshot)
    }

    pub fn resolve_conflict_with_intellij_idea(
        &self,
        repo: &mut Repository,
        path: &Path,
    ) -> Result<RepositorySnapshot> {
        self.resolve_conflict_with_intellij_idea_settings(
            repo,
            path,
            &ExternalMergeSettings::default(),
        )
    }

    pub fn resolve_conflict_with_intellij_idea_settings(
        &self,
        repo: &mut Repository,
        path: &Path,
        settings: &ExternalMergeSettings,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_RESOLVE_CONFLICTS_USING_INTELLIJ_IDEA))?;
        self.progress.emit(OperationEvent::Started(
            crate::ui_i18n::text(domain_ui_resources::WAITING_FOR_INTELLIJ_IDEA_TO_FINISH_MERGING).into(),
        ));
        conflict_for_path(&repo.index()?, path)?;
        let result = external_merge::run_intellij_idea_merge_with_settings(repo, path, settings)?;
        write_conflict_result_bytes(repo, path, &result)?;
        let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
        self.progress.emit(OperationEvent::Finished(
            crate::ui_i18n::text(domain_ui_resources::INTELLIJ_IDEA_MERGE_RESULT_APPLIED).into(),
        ));
        Ok(snapshot)
    }

    pub fn resolve_conflict_with_side(
        &self,
        repo: &mut Repository,
        path: &Path,
        side: ConflictResolutionSide,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_RESOLVE_THE_CONFLICT))?;
        let label = match side {
            ConflictResolutionSide::Ours => crate::ui_i18n::text(domain_ui_resources::CURRENT_VERSION),
            ConflictResolutionSide::Theirs => crate::ui_i18n::text(domain_ui_resources::INCOMING_VERSION),
        };
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::format(domain_ui_resources::RESOLVING_CONFLICT_USING_THE, &[("label", &format!("{}", label))])));

        let mut index = repo.index()?;
        let conflict = conflict_for_path(&index, path)?;
        let selected_entry = match side {
            ConflictResolutionSide::Ours => conflict.our.as_ref(),
            ConflictResolutionSide::Theirs => conflict.their.as_ref(),
        };

        if selected_entry.is_some() {
            let mut checkout = CheckoutBuilder::new();
            checkout
                .force()
                .path(path)
                .disable_pathspec_match(true)
                .update_index(false);
            match side {
                ConflictResolutionSide::Ours => {
                    checkout.use_ours(true);
                }
                ConflictResolutionSide::Theirs => {
                    checkout.use_theirs(true);
                }
            }
            super::worktree_compat::checkout_index_preserving_locked_directories(
                repo,
                Some(&mut index),
                &mut checkout,
            )?;
            drop(index);
            let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
            self.progress
                .emit(OperationEvent::Finished(crate::ui_i18n::format(domain_ui_resources::CONFLICT_RESOLVED_USING_THE, &[("label", &format!("{}", label))])));
            return Ok(snapshot);
        }

        remove_worktree_path(repo, path)?;
        index.conflict_remove(path)?;
        let _ = index.remove_path(path);
        index.write()?;
        drop(index);
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::format(domain_ui_resources::CONFLICT_RESOLVED_USING_THE, &[("label", &format!("{}", label))])));
        self.snapshot_after_operation(repo)
    }

    pub fn mark_conflict_resolved(
        &self,
        repo: &mut Repository,
        path: &Path,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, crate::ui_i18n::text(domain_ui_resources::CANNOT_MARK_THE_CONFLICT_AS_RESOLVED))?;
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::text(domain_ui_resources::MARKING_CONFLICT_AS_RESOLVED).into()));
        let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::text(domain_ui_resources::CONFLICT_MARKED_AS_RESOLVED).into()));
        Ok(snapshot)
    }

    fn mark_conflict_resolved_inner(
        &self,
        repo: &mut Repository,
        path: &Path,
    ) -> Result<RepositorySnapshot> {
        let mut index = repo.index()?;
        conflict_for_path(&index, path)?;

        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_MARK_CONFLICTS_AS_RESOLVED, &[])))?;
        let full_path = workdir.join(path);
        match fs::symlink_metadata(&full_path) {
            Ok(metadata) => {
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_CONFLICT_PATH_IS_A_DIRECTORY_IT_CANNOT_BE_MARKED_AS_RESOLVED, &[])));
                }
                index.conflict_remove(path)?;
                index.add_path(path)?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                index.conflict_remove(path)?;
                let _ = index.remove_path(path);
            }
            Err(err) => return Err(GitError::Io(err)),
        }
        index.write()?;
        drop(index);
        self.snapshot_after_operation(repo)
    }
}

fn conflict_for_path(index: &git2::Index, path: &Path) -> Result<git2::IndexConflict> {
    index.conflict_get(path).map_err(|err| {
        if err.code() == ErrorCode::NotFound {
            GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THIS_FILE_HAS_NO_CONFLICTS_2, &[("value0", &format!("{}", path_to_git(path)))]))
        } else {
            GitError::Git(err)
        }
    })
}

/// `diff3_merge_text` 的加载结果：三方文本可用 / 缺三方条目 / 二进制。
enum ConflictMergeText {
    /// 带 OURS/BASE/THEIRS 标记的合并原始字节（UTF-8 判定留给调用方，
    /// 便于按场景给出不同文案）。
    Text(Vec<u8>),
    Unsupported,
    Binary,
}

/// 守卫并生成冲突文件的 diff3 合并文本：
/// 三方 index 条目齐全且均为文本 blob 时，返回带
/// `<<<<<<< OURS / ||||||| BASE / ======= / >>>>>>> THEIRS` 标记的合并文本。
fn diff3_merge_text(repo: &Repository, path: &Path) -> Result<ConflictMergeText> {
    let index = repo.index()?;
    let conflict = conflict_for_path(&index, path)?;

    let (Some(ancestor), Some(ours), Some(theirs)) = (
        conflict.ancestor.as_ref(),
        conflict.our.as_ref(),
        conflict.their.as_ref(),
    ) else {
        return Ok(ConflictMergeText::Unsupported);
    };

    if [ancestor, ours, theirs]
        .into_iter()
        .any(|entry| entry.mode == 0 || blob_is_binary(repo, entry).unwrap_or(true))
    {
        return Ok(ConflictMergeText::Binary);
    }

    let mut options = MergeFileOptions::new();
    options
        .style_diff3(true)
        .ancestor_label("BASE")
        .our_label("OURS")
        .their_label("THEIRS");
    let merged = repo.merge_file_from_index(ancestor, ours, theirs, Some(&mut options))?;
    Ok(ConflictMergeText::Text(merged.content().to_vec()))
}

fn write_conflict_draft(repo: &Repository, path: &Path, draft: &str) -> Result<()> {
    write_conflict_result_bytes(repo, path, draft.as_bytes())
}

fn write_conflict_result_bytes(repo: &Repository, path: &Path, result: &[u8]) -> Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_WRITE_CONFLICT_RESULTS, &[])))?;
    let full_path = workdir.join(path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(full_path, result)?;
    Ok(())
}

fn blob_is_binary(repo: &Repository, entry: &git2::IndexEntry) -> Result<bool> {
    // 先读 ODB 对象头取体积：超过全文视图阈值的 blob 直接按二进制处理
    // （无法用文本合并编辑器），避免大对象为嗅探 NUL 先整体 inflate 到堆；
    // 小对象才加载内容并只检查前 8KB。
    if let Ok(odb) = repo.odb()
        && let Ok((size, _)) = odb.read_header(entry.id)
        && size as u64 > super::FULL_FILE_MAX_BYTES
    {
        return Ok(true);
    }
    let blob = repo.find_blob(entry.id)?;
    let sample = &blob.content()[..blob.content().len().min(8 * 1024)];
    Ok(sample.contains(&0))
}

/// 去掉行尾 `\n` / `\r\n`，得到标记行比较用的纯文本。
fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

/// 判断是否为 libgit2 生成的冲突分隔标记行。
///
/// 标记是整行的固定文本（`<<<<<<< OURS` 等），必须整行精确匹配而不是
/// `starts_with`——正常内容行完全可能以 `=======` 开头（RST/Markdown
/// 标题下划线、分隔线），前缀匹配会把它误判为块边界，导致三栏解析错位。
fn is_marker_line(line: &str, marker: &str) -> bool {
    trim_line_ending(line) == marker
}

fn parse_diff3_conflict_text(
    content: &str,
) -> Result<(String, String, String, Vec<ConflictBlock>)> {
    let lines = split_lines_preserve_endings(content);
    let mut index = 0;
    let mut draft = String::new();
    let mut ours_text = String::new();
    let mut theirs_text = String::new();
    let mut blocks = Vec::new();

    while index < lines.len() {
        let line = lines[index];
        if !is_marker_line(line, "<<<<<<< OURS") {
            draft.push_str(line);
            ours_text.push_str(line);
            theirs_text.push_str(line);
            index += 1;
            continue;
        }

        index += 1;
        let ours_start = index;
        while index < lines.len() && !is_marker_line(lines[index], "||||||| BASE") {
            index += 1;
        }
        if index >= lines.len() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CONFLICT_TEXT_IS_MISSING_THE_BASE_SEPARATOR, &[])));
        }
        let ours = lines[ours_start..index].concat();

        index += 1;
        let base_start = index;
        while index < lines.len() && !is_marker_line(lines[index], "=======") {
            index += 1;
        }
        if index >= lines.len() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CONFLICT_TEXT_IS_MISSING_THE_MIDDLE_SEPARATOR, &[])));
        }
        let base = lines[base_start..index].concat();

        index += 1;
        let theirs_start = index;
        while index < lines.len() && !is_marker_line(lines[index], ">>>>>>> THEIRS") {
            index += 1;
        }
        if index >= lines.len() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CONFLICT_TEXT_IS_MISSING_THE_THEIRS_END_MARKER, &[])));
        }
        let theirs = lines[theirs_start..index].concat();
        index += 1;

        let start = draft.len();
        let ours_start = ours_text.len();
        let theirs_start = theirs_text.len();
        // IDEA 式结果区以祖先冲突片段起步，无冲突内容仍采用 libgit2 的合并结果。
        draft.push_str(&base);
        ours_text.push_str(&ours);
        theirs_text.push_str(&theirs);
        let end = draft.len();
        blocks.push(ConflictBlock {
            base: Some(base),
            ours,
            theirs,
            start,
            end,
            ours_start,
            ours_end: ours_text.len(),
            theirs_start,
            theirs_end: theirs_text.len(),
            status: ConflictBlockStatus::Unresolved,
            has_manual_edits: false,
        });
    }

    Ok((draft, ours_text, theirs_text, blocks))
}

fn split_lines_preserve_endings(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start = 0;
    for (index, ch) in content.char_indices() {
        if ch == '\n' {
            lines.push(&content[start..index + 1]);
            start = index + 1;
        }
    }
    if start < content.len() {
        lines.push(&content[start..]);
    }
    lines
}

#[cfg(test)]
#[path = "../tests/git/conflicts.rs"]
mod tests;
