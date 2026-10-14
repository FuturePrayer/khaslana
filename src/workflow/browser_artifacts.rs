//! 每个 Edge 会话独占输出目录；只回收快照与截图，不删除下载和其它导出。
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{GitError, Result};

pub type BrowserCleanupWarnings = Arc<Mutex<Vec<String>>>;
const MANIFEST: &str = "snapshots.json";

#[derive(Default, Deserialize, Serialize)]
struct Manifest {
    snapshots: BTreeSet<PathBuf>,
    #[serde(default)]
    exports: BTreeSet<PathBuf>,
}

pub(super) struct BrowserArtifacts {
    root: PathBuf,
    manifest: Manifest,
    _lock: File,
    warnings: BrowserCleanupWarnings,
}

impl BrowserArtifacts {
    pub(super) fn new(data_dir: &Path, warnings: BrowserCleanupWarnings) -> Result<Self> {
        static SERIAL: AtomicU64 = AtomicU64::new(1);
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let base = data_dir.join("workflow-browser-sessions");
        fs::create_dir_all(&base)?;
        let root = base.join(format!("{}-{stamp}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&root)?;
        fs::create_dir_all(root.join("snapshots"))?;
        fs::create_dir_all(root.join("outputs"))?;
        let lock = File::create(root.join("owner.lock"))?;
        lock.lock()?;
        let artifacts = Self { root, manifest: Manifest::default(), _lock: lock, warnings };
        artifacts.save()?;
        Ok(artifacts)
    }

    pub(super) fn output_dir(&self) -> PathBuf { self.root.join("outputs") }

    pub(super) fn prepare(&mut self, tool: &str, arguments: &mut Map<String, Value>) -> Result<()> {
        if !matches!(tool, "browser_snapshot" | "browser_take_screenshot") { return Ok(()); }
        let filename = arguments.get("filename");
        if filename.is_some_and(|value| !value.is_string()) { return Ok(()); }
        // 无 filename 的结构快照继续返回正文；自动落盘的 page-*.yml 在收尾回收。
        if tool == "browser_snapshot" && filename.is_none() { return Ok(()); }
        let suffix = if tool == "browser_snapshot" { "yml" }
            else if arguments.get("type").and_then(Value::as_str) == Some("jpeg") { "jpeg" } else { "png" };
        let leaf = filename.and_then(Value::as_str).and_then(|name| Path::new(name).file_name())
            .filter(|name| !name.is_empty()).map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("snapshot.{suffix}"));
        let relative = PathBuf::from("snapshots").join(format!("{}-{leaf}", self.manifest.snapshots.len()));
        self.manifest.snapshots.insert(relative.clone());
        // 先持久化归属，再允许工具写入；中断也能在下次启动清理。
        self.save()?;
        arguments.insert("filename".into(), Value::String(self.root.join(relative).to_string_lossy().into_owned()));
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let temporary = self.root.join(".snapshots.pending.json");
        fs::write(&temporary, serde_json::to_vec(&self.manifest)
            .map_err(|error| GitError::Message(format!("记录 Edge 快照失败：{error}")))?)?;
        fs::rename(temporary, self.root.join(MANIFEST))?;
        Ok(())
    }

    pub(super) fn record_result(&mut self, content: &Value) -> Result<()> {
        let Some(parts) = content.as_array() else { return Ok(()) };
        for text in parts.iter().filter_map(|part| part.get("text").and_then(Value::as_str)) {
            for captures in regex::Regex::new(r"(?m)^-?\s*\[(?:Snapshot|Screenshot[^\]]*)\]\(([^)\r\n]+)\)")
                .expect("快照链接正则固定").captures_iter(text) {
                if let Some(relative) = self.owned_output_path(&captures[1]) {
                    self.manifest.snapshots.insert(relative);
                }
            }
            for captures in regex::Regex::new(r#"(?m)^- Downloaded file .* to "([^"\r\n]+)""#)
                .expect("下载结果正则固定").captures_iter(text) {
                if let Some(relative) = self.owned_output_path(&captures[1]) {
                    self.manifest.exports.insert(relative);
                }
            }
        }
        self.save()
    }

    fn owned_output_path(&self, name: &str) -> Option<PathBuf> {
        let path = PathBuf::from(name).canonicalize().ok()?;
        let root = self.root.canonicalize().ok()?;
        let relative = path.strip_prefix(root).ok()?.to_path_buf();
        (relative.starts_with("outputs") || relative.starts_with("snapshots")).then_some(relative)
    }
}

impl Drop for BrowserArtifacts {
    fn drop(&mut self) {
        if let Err(error) = cleanup(&self.root, &self.manifest) {
            self.warnings.lock().unwrap_or_else(|p| p.into_inner())
                .push(format!("Edge 快照和截图清理未完成，下次启动将重试：{error}"));
        }
    }
}

fn cleanup(root: &Path, manifest: &Manifest) -> Result<()> {
    let mut files = manifest.snapshots.clone();
    // 固定版本自动快照采用 page-<ISO 时间>.yml，不能按图片／YAML 扩展名扫仓库。
    let output = root.join("outputs");
    if output.is_dir() {
        for entry in fs::read_dir(&output)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if regex::Regex::new(r"^page-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}-\d{3}Z\.yml$")
                .expect("自动快照文件名正则固定").is_match(&name) {
                files.insert(PathBuf::from("outputs").join(name));
            }
        }
    }
    for relative in files {
        if manifest.exports.contains(&relative) { continue; }
        if relative.components().any(|part| !matches!(part, Component::Normal(_)))
            || !(relative.starts_with("snapshots") || relative.starts_with("outputs")) {
            return Err(GitError::Message("Edge 快照清单包含越界路径".into()));
        }
        let path = root.join(&relative);
        // 不跟随目录链接；即使清单被修改也不能删除会话目录之外的文件。
        let canonical_root = root.canonicalize()?;
        if !path.parent().ok_or_else(|| GitError::Message("快照路径无效".into()))?
            .canonicalize()?.starts_with(&canonical_root) {
            return Err(GitError::Message("Edge 快照目录指向会话范围之外".into()));
        }
        match fs::remove_file(path) {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(error) => return Err(error.into()),
        }
    }
    // 导出文件仍在 outputs；只移除完成清理的归属标记。
    match fs::remove_file(root.join(MANIFEST)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn retry_browser_cleanup(data_dir: &Path) -> Vec<String> {
    let root = data_dir.join("workflow-browser-sessions");
    let mut warnings = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return warnings,
        Err(error) => return vec![format!("读取 Edge 残留快照目录失败：{error}")],
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => { warnings.push(format!("读取 Edge 残留会话失败：{error}")); continue; }
        };
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) && path.join(MANIFEST).is_file() {
            let result = (|| -> Result<()> {
                let lock = fs::OpenOptions::new().read(true).write(true).open(path.join("owner.lock"))?;
                match lock.try_lock() {
                    Ok(()) => {},
                    Err(std::fs::TryLockError::WouldBlock) => return Ok(()),
                    Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
                }
                let manifest: Manifest = serde_json::from_slice(&fs::read(path.join(MANIFEST))?)
                    .map_err(|_| GitError::Message("Edge 快照归属记录无效".into()))?;
                cleanup(&path, &manifest)
            })();
            if let Err(error) = result { warnings.push(format!("Edge 残留快照清理未完成：{error}")); }
        }
    }
    warnings
}

#[cfg(test)]
#[path = "../tests/browser_artifacts.rs"]
mod tests;
