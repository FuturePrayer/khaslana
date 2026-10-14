use super::*;
use serde_json::json;

#[test]
fn snapshots_are_confined_and_removed_without_deleting_exports() {
    let dir = tempfile::tempdir().unwrap();
    let warnings = Arc::new(Mutex::new(Vec::new()));
    let mut artifacts = BrowserArtifacts::new(dir.path(), warnings.clone()).unwrap();
    let root = artifacts.root.clone();
    let mut arguments = json!({"filename":"../../outside.png"}).as_object().unwrap().clone();
    artifacts.prepare("browser_take_screenshot", &mut arguments).unwrap();
    let screenshot = PathBuf::from(arguments["filename"].as_str().unwrap());
    assert!(screenshot.starts_with(root.join("snapshots")));
    fs::write(&screenshot, b"image").unwrap();
    let automatic = artifacts.output_dir().join("page-2026-10-12T08-00-00-000Z.yml");
    fs::write(&automatic, b"tree").unwrap();
    let download = artifacts.output_dir().join("download.png");
    fs::write(&download, b"download").unwrap();
    let log = artifacts.output_dir().join("console.log");
    fs::write(&log, b"log").unwrap();
    drop(artifacts);
    assert!(!screenshot.exists());
    assert!(!automatic.exists());
    assert!(download.exists());
    assert!(log.exists());
    assert!(warnings.lock().unwrap().is_empty());
}

#[test]
fn inline_snapshot_stays_inline_and_explicit_snapshot_is_owned() {
    let dir = tempfile::tempdir().unwrap();
    let mut artifacts = BrowserArtifacts::new(dir.path(), Arc::new(Mutex::new(Vec::new()))).unwrap();
    let mut arguments = Map::new();
    artifacts.prepare("browser_snapshot", &mut arguments).unwrap();
    assert!(arguments.is_empty());
    arguments.insert("filename".into(), json!("tree.txt"));
    artifacts.prepare("browser_snapshot", &mut arguments).unwrap();
    let path = PathBuf::from(arguments["filename"].as_str().unwrap());
    fs::write(&path, b"tree").unwrap();
    drop(artifacts);
    assert!(!path.exists());
}

#[test]
fn active_sessions_are_not_cleaned_by_retry_and_sessions_are_independent() {
    let dir = tempfile::tempdir().unwrap();
    let report = Arc::new(Mutex::new(Vec::new()));
    let first = BrowserArtifacts::new(dir.path(), report.clone()).unwrap();
    let second = BrowserArtifacts::new(dir.path(), report).unwrap();
    let one = first.output_dir().join("page-2026-10-12T08-00-00-000Z.yml");
    let two = second.output_dir().join("page-2026-10-12T08-00-00-000Z.yml");
    fs::write(&one, b"one").unwrap();
    fs::write(&two, b"two").unwrap();
    assert!(retry_browser_cleanup(dir.path()).is_empty());
    assert!(one.exists() && two.exists());
    drop(first);
    assert!(!one.exists() && two.exists());
    drop(second);
    assert!(!two.exists());
}

#[test]
fn interrupted_session_manifest_is_retried_and_invalid_paths_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("workflow-browser-sessions/stale");
    fs::create_dir_all(root.join("snapshots")).unwrap();
    fs::create_dir(root.join("outputs")).unwrap();
    File::create(root.join("owner.lock")).unwrap();
    let path = root.join("snapshots/tree.yml");
    fs::write(&path, b"tree").unwrap();
    let mut manifest = Manifest::default();
    manifest.snapshots.insert(PathBuf::from("snapshots/tree.yml"));
    fs::write(root.join(MANIFEST), serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(retry_browser_cleanup(dir.path()).is_empty());
    assert!(!path.exists());
    let outside = dir.path().join("keep.txt");
    fs::write(&outside, b"keep").unwrap();
    manifest.snapshots.insert(PathBuf::from("../../keep.txt"));
    assert!(cleanup(&root, &manifest).is_err());
    assert!(outside.exists());
}

#[test]
fn failure_and_panic_unwind_cleanup_owned_files() {
    let dir = tempfile::tempdir().unwrap();
    let report = Arc::new(Mutex::new(Vec::new()));
    let mut paths = Vec::new();
    let outcome = (|| -> Result<()> {
        let mut artifacts = BrowserArtifacts::new(dir.path(), report.clone())?;
        let mut args = json!({"filename":"failure.yml"}).as_object().unwrap().clone();
        artifacts.prepare("browser_snapshot", &mut args)?;
        let path = PathBuf::from(args["filename"].as_str().unwrap());
        fs::write(&path, b"tree")?;
        paths.push(path);
        Err(GitError::WorkflowCancelled)
    })();
    assert!(outcome.is_err());
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut artifacts = BrowserArtifacts::new(dir.path(), report.clone()).unwrap();
        let mut args = json!({"filename":"panic.png"}).as_object().unwrap().clone();
        artifacts.prepare("browser_take_screenshot", &mut args).unwrap();
        let path = PathBuf::from(args["filename"].as_str().unwrap());
        fs::write(&path, b"image").unwrap();
        paths.push(path);
        panic!("测试会话异常退出");
    }));
    assert!(outcome.is_err());
    assert!(paths.iter().all(|path| !path.exists()));
    assert!(report.lock().unwrap().is_empty());
}

#[cfg(windows)]
#[test]
fn locked_screenshot_reports_warning_and_next_start_retries() {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = tempfile::tempdir().unwrap();
    let report = Arc::new(Mutex::new(Vec::new()));
    let mut artifacts = BrowserArtifacts::new(dir.path(), report.clone()).unwrap();
    let root = artifacts.root.clone();
    let mut args = json!({"filename":"locked.png"}).as_object().unwrap().clone();
    artifacts.prepare("browser_take_screenshot", &mut args).unwrap();
    let path = PathBuf::from(args["filename"].as_str().unwrap());
    let locked = fs::OpenOptions::new().write(true).create(true).truncate(true).share_mode(0).open(&path).unwrap();
    drop(artifacts);
    assert!(root.join(MANIFEST).exists());
    assert!(!report.lock().unwrap().is_empty());
    drop(locked);
    assert!(retry_browser_cleanup(dir.path()).is_empty());
    assert!(!path.exists());
    assert!(!root.join(MANIFEST).exists());
}

#[test]
fn returned_artifact_paths_are_recorded_and_downloads_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let mut artifacts = BrowserArtifacts::new(dir.path(), Arc::new(Mutex::new(Vec::new()))).unwrap();
    let snapshot = artifacts.output_dir().join("page-2026-10-12T08-00-00-000Z.yml");
    let download = artifacts.output_dir().join("page-2026-10-12T08-00-01-000Z.yml");
    fs::write(&snapshot, b"tree").unwrap();
    fs::write(&download, b"download").unwrap();
    artifacts.record_result(&json!([{ "type":"text", "text":format!("### Snapshot\n- [Snapshot]({})\n### Events\n- Downloaded file page to \"{}\"", snapshot.display(), download.display()) }])).unwrap();
    assert!(artifacts.manifest.snapshots.contains(&PathBuf::from("outputs").join(snapshot.file_name().unwrap())));
    drop(artifacts);
    assert!(!snapshot.exists());
    assert!(download.exists());
}
