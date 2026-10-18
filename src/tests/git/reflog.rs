use std::collections::BTreeMap;
use std::io::Cursor;

use git2::{Repository, ResetType, WorktreeAddOptions};

use super::*;
use crate::git::test_support::git_test_support as git_support;

const ZERO: &str = "0000000000000000000000000000000000000000";
const OLD: &str = "0123456789abcdef0123456789abcdef01234567";
const NEW: &str = "abcdef0123456789abcdef0123456789abcdef01";

fn record(timestamp: i64, message: &str) -> Vec<u8> {
    format!("{OLD} {NEW} Test Actor <actor@example.invalid> {timestamp} +0800\t{message}\n")
        .into_bytes()
}

fn put_log(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn head_log(repo: &Repository) -> PathBuf {
    repo.path().join("logs/HEAD")
}

fn snapshot(repo: &Repository, scope: ReflogScope) -> ReflogSnapshot {
    git_support::service()
        .read_reflog_snapshot(repo, &scope, &AtomicBool::new(false))
        .unwrap()
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, directory: &Path, result: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if entry.file_type().unwrap().is_dir() {
                result.insert(relative, None);
                visit(root, &path, result);
            } else {
                result.insert(relative, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

#[test]
fn missing_logs_are_empty_and_leave_repository_byte_for_byte_unchanged() {
    let (_dir, repo, service) = git_support::init_repo();
    assert!(!head_log(&repo).exists());
    let before = inventory(repo.path());
    let sources = service.reflog_sources(&repo).unwrap();
    assert!(sources.branches.is_empty());
    assert_eq!(sources.current_branch.as_deref(), Some("main"));
    for scope in [
        ReflogScope::Head,
        ReflogScope::CurrentBranch,
        ReflogScope::All,
    ] {
        let result = snapshot(&repo, scope);
        assert!(result.entries.is_empty());
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.sources_read, 0);
        assert_eq!(result.sources_total, 1);
    }
    assert_eq!(before, inventory(repo.path()));
    assert!(!head_log(&repo).exists());
}

#[test]
fn disabled_logging_does_not_create_logs_for_existing_branch() {
    let (dir, repo, service) = git_support::init_repo();
    repo.config()
        .unwrap()
        .set_bool("core.logAllRefUpdates", false)
        .unwrap();
    git_support::write_file(dir.path(), "data.txt", "one");
    git_support::commit_all(&repo, "without logs");
    assert!(!head_log(&repo).exists());
    let before = inventory(repo.path());
    assert_eq!(service.reflog_sources(&repo).unwrap().branches, ["main"]);
    let result = snapshot(&repo, ReflogScope::All);
    assert!(result.entries.is_empty());
    assert_eq!((result.sources_read, result.sources_total), (0, 2));
    assert!(result.diagnostics.iter().any(|line| line.contains("0/2")));
    assert_eq!(before, inventory(repo.path()));
}

#[test]
fn real_commits_reset_and_local_branch_are_preserved_as_separate_sources() {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    let first = git_support::commit_all(&repo, "first");
    git_support::write_file(dir.path(), "data.txt", "two");
    let second = git_support::commit_all(&repo, "second");
    repo.branch("feature/topic", &repo.find_commit(first).unwrap(), false)
        .unwrap();
    repo.reference("refs/remotes/origin/main", second, false, "remote fixture")
        .unwrap();
    // 所有写操作都限于临时仓库；Soft reset 产生真实的引用移动日志。
    repo.reset(
        &repo.find_object(first, None).unwrap(),
        ResetType::Soft,
        None,
    )
    .unwrap();
    put_log(
        &repo.path().join("logs/refs/heads/deleted"),
        &record(10, "orphan log"),
    );
    put_log(
        &repo.path().join("logs/refs/remotes/origin/main"),
        &record(10, "remote log"),
    );
    let sources = service.reflog_sources(&repo).unwrap();
    assert_eq!(sources.branches, ["feature/topic", "main"]);
    assert_eq!(sources.current_branch.as_deref(), Some("main"));
    let before = inventory(repo.path());
    let head = snapshot(&repo, ReflogScope::Head);
    assert!(head.entries.iter().all(|entry| entry.ref_name == "HEAD"));
    assert!(head.entries.iter().any(|entry| {
        entry.old_oid == second.to_string() && entry.new_oid == first.to_string()
    }));
    let all = snapshot(&repo, ReflogScope::All);
    assert_eq!((all.sources_read, all.sources_total), (3, 3));
    let reset_refs = all
        .entries
        .iter()
        .filter(|entry| entry.old_oid == second.to_string() && entry.new_oid == first.to_string())
        .map(|entry| entry.ref_name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(reset_refs, ["HEAD", "refs/heads/main"]);
    assert!(
        all.entries
            .iter()
            .any(|entry| entry.ref_name == "refs/heads/feature/topic")
    );
    assert!(
        !all.entries
            .iter()
            .any(|entry| entry.message == "orphan log" || entry.message == "remote log")
    );
    let current = snapshot(&repo, ReflogScope::CurrentBranch);
    assert!(
        current
            .entries
            .iter()
            .all(|entry| entry.ref_name == "refs/heads/main")
    );
    let branch = snapshot(&repo, ReflogScope::LocalBranch("feature/topic".into()));
    assert!(
        branch
            .entries
            .iter()
            .all(|entry| entry.ref_name == "refs/heads/feature/topic")
    );
    assert_eq!(before, inventory(repo.path()));
}

#[test]
fn detached_head_has_head_log_but_no_current_branch_source() {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    let oid = git_support::commit_all(&repo, "first");
    repo.set_head_detached(oid).unwrap();
    assert_eq!(service.reflog_sources(&repo).unwrap().current_branch, None);
    let current = snapshot(&repo, ReflogScope::CurrentBranch);
    assert_eq!(current.sources_total, 0);
    assert!(current.entries.is_empty());
    assert!(!current.diagnostics.is_empty());
    assert!(!snapshot(&repo, ReflogScope::Head).entries.is_empty());
}

#[test]
fn pages_at_100_and_101_and_limit_clamping_use_snapshot_offsets() {
    let (_dir, repo, _service) = git_support::init_repo();
    let mut bytes = Vec::new();
    for timestamp in 0..100 {
        bytes.extend(record(timestamp, "item"));
    }
    put_log(&head_log(&repo), &bytes);
    let hundred = snapshot(&repo, ReflogScope::Head);
    let query = ReflogQuery::default();
    let page = hundred.query(&query, 0, 100);
    assert_eq!(
        (page.entries.len(), page.total, page.next_offset),
        (100, 100, None)
    );
    bytes.extend(record(100, "item"));
    put_log(&head_log(&repo), &bytes);
    let hundred_one = snapshot(&repo, ReflogScope::Head);
    let first = hundred_one.query(&query, 0, usize::MAX);
    assert_eq!(
        (first.entries.len(), first.total, first.next_offset),
        (100, 101, Some(100))
    );
    assert_eq!(first.entries[0].timestamp, 100);
    let last = hundred_one.query(&query, first.next_offset.unwrap(), 100);
    assert_eq!(
        (last.entries.len(), last.total, last.next_offset),
        (1, 101, None)
    );
    assert_eq!(last.entries[0].timestamp, 0);
    assert!(
        hundred_one
            .query(&query, usize::MAX, 100)
            .entries
            .is_empty()
    );
    let zero = hundred_one.query(&query, 0, 0);
    assert_eq!(
        (zero.entries.len(), zero.total, zero.next_offset),
        (0, 101, None)
    );
    assert_eq!(hundred.entries[0].timestamp, 99);
}

#[test]
fn search_and_inclusive_since_filter_before_paging() {
    let (_dir, repo, _service) = git_support::init_repo();
    let mut bytes = Vec::new();
    for timestamp in 0..=120 {
        let message = if timestamp % 2 == 0 {
            "Needle"
        } else {
            "other"
        };
        bytes.extend(record(timestamp, message));
    }
    put_log(&head_log(&repo), &bytes);
    let result = snapshot(&repo, ReflogScope::Head);
    let query = ReflogQuery {
        search: "nEeDlE".into(),
        since: Some(20),
    };
    let page = result.query(&query, 10, 7);
    assert_eq!((page.total, page.next_offset), (51, Some(17)));
    assert_eq!(
        page.entries
            .iter()
            .map(|entry| entry.timestamp)
            .collect::<Vec<_>>(),
        [100, 98, 96, 94, 92, 90, 88]
    );
    for search in [
        "HEAD",
        "TEST ACTOR",
        "ACTOR@EXAMPLE.INVALID",
        "0123456789ABCDEF",
        "ABCDEF0123456789",
    ] {
        let page = result.query(
            &ReflogQuery {
                search: search.into(),
                since: None,
            },
            120,
            20,
        );
        assert_eq!(
            (page.entries.len(), page.total, page.next_offset),
            (1, 121, None)
        );
    }
    let empty = result.query(
        &ReflogQuery {
            search: "absent".into(),
            since: None,
        },
        0,
        100,
    );
    assert_eq!((empty.total, empty.next_offset), (0, None));
}

#[test]
fn parser_accepts_zero_oids_missing_messages_and_signed_times() {
    let line = format!("{ZERO} {NEW} 姓名 With Spaces <mail@example.invalid> -1 -0330");
    let entry = parse_record(line.as_bytes(), "HEAD", 12).unwrap();
    assert_eq!(entry.id, "HEAD:12");
    assert_eq!(entry.old_oid, ZERO);
    assert_eq!(entry.new_oid, NEW);
    assert_eq!(entry.actor, "姓名 With Spaces");
    assert_eq!(entry.actor_email, "mail@example.invalid");
    assert_eq!(entry.timestamp, -1);
    assert_eq!(entry.timezone_offset, -210);
    assert!(entry.message.is_empty());
    assert!(!entry.lossy);
    assert_eq!(parse_timezone(b"+0545").unwrap(), 345);
    assert_eq!(parse_timezone(b"-0000").unwrap(), 0);
    let entry = parse_record(format!("{line}\t").as_bytes(), "HEAD", 13).unwrap();
    assert!(entry.message.is_empty());
}

#[test]
fn lossy_actor_email_and_message_are_marked_and_diagnosed() {
    let (_dir, repo, _service) = git_support::init_repo();
    let mut bytes = format!("{ZERO} {NEW} ").into_bytes();
    bytes.extend_from_slice(b"Name\xff <email\xfe@example.invalid> 2 +0000\tmessage\xff\n");
    put_log(&head_log(&repo), &bytes);
    let result = snapshot(&repo, ReflogScope::Head);
    assert_eq!(result.entries.len(), 1);
    let entry = &result.entries[0];
    assert!(entry.lossy);
    assert!(entry.actor.contains('\u{fffd}'));
    assert!(entry.actor_email.contains('\u{fffd}'));
    assert!(entry.message.contains('\u{fffd}'));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|line| line.contains("非 UTF-8"))
    );
}

#[test]
fn malformed_oids_times_and_incomplete_lines_are_skipped_without_forged_entries() {
    let (_dir, repo, _service) = git_support::init_repo();
    let mut bytes = b"bad\n\n".to_vec();
    for line in [
        format!("short {NEW} Actor <a@b> 1 +0000"),
        format!("{} {NEW} Actor <a@b> 1 +0000", "g".repeat(40)),
        format!("{OLD} {NEW} Actor <a@b> invalid +0000"),
        format!("{OLD} {NEW} Actor <a@b> 9223372036854775808 +0000"),
        format!("{OLD} {NEW} Actor <a@b> 1 +2460"),
        format!("{OLD} {NEW} Actor <a@b> 1 0800"),
        format!("{OLD} {NEW} Actor a@b 1 +0000"),
    ] {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    bytes.extend(record(10, "valid"));
    let mut incomplete = record(11, "no newline");
    incomplete.pop();
    bytes.extend(incomplete);
    put_log(&head_log(&repo), &bytes);
    let result = snapshot(&repo, ReflogScope::Head);
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].message, "valid");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|line| line.contains("不完整"))
    );
    assert_eq!(result.diagnostics.len(), 10);
    for timezone in [b"+2400", b"+0060", b"+0x00", b"?0000"] {
        assert!(parse_timezone(timezone).is_err());
    }
}

#[test]
fn all_sorting_uses_timestamp_then_reference_then_original_source_order() {
    let (dir, repo, _service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    let oid = git_support::commit_all(&repo, "seed");
    repo.branch("feature", &repo.find_commit(oid).unwrap(), false)
        .unwrap();
    let mut head = record(20, "head first tie");
    head.extend(record(30, "newest"));
    head.extend(record(20, "head second tie"));
    head.extend(record(-1, "oldest"));
    put_log(&head_log(&repo), &head);
    put_log(
        &repo.path().join("logs/refs/heads/main"),
        &record(20, "main"),
    );
    put_log(
        &repo.path().join("logs/refs/heads/feature"),
        &record(20, "feature"),
    );
    let result = snapshot(&repo, ReflogScope::All);
    assert_eq!(
        result
            .entries
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>(),
        [
            "newest",
            "head first tie",
            "head second tie",
            "feature",
            "main",
            "oldest"
        ]
    );
    assert_eq!(result.entries.len(), 6);
    assert_eq!(result.entries[1].id, "HEAD:1");
    assert_eq!(result.entries[2].id, "HEAD:3");
    assert!(result.entries.iter().all(|entry| entry.new_oid == NEW));
}

#[test]
fn line_limit_accepts_the_boundary_and_skips_an_overlong_line() {
    let (_dir, repo, _service) = git_support::init_repo();
    let prefix = format!("{OLD} {NEW} Actor <a@b> 1 +0000\t");
    let mut bytes = prefix.as_bytes().to_vec();
    bytes.resize(LINE_BYTE_LIMIT, b'x');
    bytes.push(b'\n');
    bytes.extend_from_slice(prefix.as_bytes());
    bytes.extend(std::iter::repeat_n(
        b'x',
        LINE_BYTE_LIMIT + 1 - prefix.len(),
    ));
    bytes.push(b'\n');
    bytes.extend(record(3, "after long line"));
    put_log(&head_log(&repo), &bytes);
    let result = snapshot(&repo, ReflogScope::Head);
    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].message, "after long line");
    assert_eq!(
        result.entries[1].message.len(),
        LINE_BYTE_LIMIT - prefix.len()
    );
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].contains("256 KiB"));
}

#[test]
fn whole_snapshot_budgets_apply_across_sources_and_fail_in_all_scope() {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    git_support::commit_all(&repo, "seed");
    let bytes = record(1, "one");
    put_log(&head_log(&repo), &bytes);
    put_log(&repo.path().join("logs/refs/heads/main"), &bytes);
    let identity = service.recovery_repo_identity(&repo).unwrap();
    let cancelled = AtomicBool::new(false);
    let limits = ReadLimits {
        raw_bytes: (bytes.len() * 2 - 1) as u64,
        ..ReadLimits::default()
    };
    let error =
        read_snapshot_attempt(&repo, &identity, &ReflogScope::All, &cancelled, limits).unwrap_err();
    assert!(error.to_string().contains("64 MiB"));
    let limits = ReadLimits {
        entries: 1,
        ..ReadLimits::default()
    };
    let error =
        read_snapshot_attempt(&repo, &identity, &ReflogScope::All, &cancelled, limits).unwrap_err();
    assert!(error.to_string().contains("200000"));
    assert_eq!(ReadLimits::default().entries, 200_000);
    assert_eq!(ReadLimits::default().raw_bytes, 64 * 1024 * 1024);
}

#[test]
fn oversized_actual_file_is_rejected_before_reading_or_returning_partial_data() {
    let (_dir, repo, service) = git_support::init_repo();
    put_log(&head_log(&repo), b"");
    File::options()
        .write(true)
        .open(head_log(&repo))
        .unwrap()
        .set_len(RAW_BYTE_LIMIT + 1)
        .unwrap();
    let error = service
        .read_reflog_snapshot(&repo, &ReflogScope::All, &AtomicBool::new(false))
        .unwrap_err();
    assert!(error.to_string().contains("64 MiB"));
}

#[test]
fn all_returns_other_sources_with_explicit_coverage_when_one_is_unreadable() {
    let (dir, repo, _service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    git_support::commit_all(&repo, "seed");
    let branch_log = repo.path().join("logs/refs/heads/main");
    fs::remove_file(&branch_log).unwrap();
    fs::create_dir(&branch_log).unwrap();
    let result = snapshot(&repo, ReflogScope::All);
    assert_eq!((result.sources_read, result.sources_total), (1, 2));
    assert!(!result.entries.is_empty());
    assert!(result.entries.iter().all(|entry| entry.ref_name == "HEAD"));
    assert!(
        result
            .diagnostics
            .iter()
            .any(|line| line.contains("refs/heads/main") && line.contains("未覆盖"))
    );
    assert!(result.diagnostics.iter().any(|line| line.contains("1/2")));
    let error = git_support::service()
        .read_reflog_snapshot(&repo, &ReflogScope::CurrentBranch, &AtomicBool::new(false))
        .unwrap_err();
    assert!(error.to_string().contains("不可读"));
}

#[test]
fn cancellation_before_and_during_read_returns_an_error() {
    let (_dir, repo, service) = git_support::init_repo();
    let before = inventory(repo.path());
    let cancelled = AtomicBool::new(true);
    let error = service
        .read_reflog_snapshot(&repo, &ReflogScope::Head, &cancelled)
        .unwrap_err();
    assert!(error.to_string().contains("取消"));
    assert_eq!(before, inventory(repo.path()));

    struct CancelDuringRead<'a> {
        bytes: Cursor<Vec<u8>>,
        cancelled: &'a AtomicBool,
    }
    impl Read for CancelDuringRead<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let count = self.bytes.read(buffer)?;
            self.cancelled.store(true, Ordering::Relaxed);
            Ok(count)
        }
    }
    cancelled.store(false, Ordering::Relaxed);
    let mut reader = CancelDuringRead {
        bytes: Cursor::new(record(1, "one")),
        cancelled: &cancelled,
    };
    let error = read_records(
        &mut reader,
        "HEAD",
        &cancelled,
        ReadLimits::default(),
        &mut ReadBudget::default(),
        &mut Diagnostics::default(),
    )
    .unwrap_err();
    assert!(matches!(error, ReadFailure::Fatal(error) if error.to_string().contains("取消")));
}

#[test]
fn invalid_or_missing_local_branch_names_cannot_escape_the_log_root() {
    let (_dir, repo, service) = git_support::init_repo();
    let before = inventory(repo.path());
    for name in [
        "",
        "../outside",
        "/absolute",
        "a/../../outside",
        "a\\outside",
        "C:/outside",
        "a//b",
        "a@{b",
        "not-present",
    ] {
        assert!(
            service
                .read_reflog_snapshot(
                    &repo,
                    &ReflogScope::LocalBranch(name.into()),
                    &AtomicBool::new(false),
                )
                .is_err(),
            "unexpectedly accepted {name:?}"
        );
    }
    assert_eq!(before, inventory(repo.path()));
}

#[test]
fn unsupported_ref_backend_is_explicitly_rejected() {
    let (_dir, repo, service) = git_support::init_repo();
    repo.config()
        .unwrap()
        .set_str("extensions.refStorage", "reftable")
        .unwrap();
    let before = inventory(repo.path());
    let error = service.reflog_sources(&repo).unwrap_err();
    assert!(error.to_string().contains("reftable"));
    let error = service
        .read_reflog_snapshot(&repo, &ReflogScope::Head, &AtomicBool::new(false))
        .unwrap_err();
    assert!(error.to_string().contains("reftable"));
    assert_eq!(before, inventory(repo.path()));
}

#[test]
fn changed_metadata_discards_the_source_instead_of_accepting_mixed_data() {
    let (_dir, repo, _service) = git_support::init_repo();
    let path = head_log(&repo);
    put_log(&path, &record(1, "before"));
    let source = LogSource {
        ref_name: "HEAD".into(),
        path: path.clone(),
        root: repo.path().into(),
    };
    let before = observe_source(&source).unwrap();
    let Observation::Ready {
        path: resolved,
        stamp,
    } = &before
    else {
        panic!("missing fixture")
    };
    put_log(&path, &record(2, "after with changed length"));
    let mut budget = ReadBudget::default();
    let result = read_source(
        resolved,
        &source,
        stamp,
        &AtomicBool::new(false),
        ReadLimits::default(),
        &mut budget,
        &mut Diagnostics::default(),
    )
    .unwrap();
    assert!(result.is_none());
    assert_eq!(budget.raw_bytes, 0);
    assert_ne!(observe_source(&source).unwrap(), before);
    let result = snapshot(&repo, ReflogScope::Head);
    assert_eq!(result.entries[0].message, "after with changed length");
}

#[test]
fn malformed_line_diagnostics_are_bounded() {
    let (_dir, repo, _service) = git_support::init_repo();
    put_log(&head_log(&repo), &b"bad\n".repeat(DIAGNOSTIC_LIMIT + 2));
    let result = snapshot(&repo, ReflogScope::Head);
    assert!(result.entries.is_empty());
    assert_eq!(result.diagnostics.len(), DIAGNOSTIC_LIMIT + 1);
    assert!(result.diagnostics.last().unwrap().contains("2 条日志诊断"));
}

#[cfg(unix)]
#[test]
fn symlinked_log_cannot_escape_repository_directory() {
    let (_dir, repo, service) = git_support::init_repo();
    let outside = tempfile::TempDir::new().unwrap();
    let outside_log = outside.path().join("HEAD");
    put_log(&outside_log, &record(1, "outside"));
    fs::create_dir_all(repo.path().join("logs")).unwrap();
    std::os::unix::fs::symlink(&outside_log, head_log(&repo)).unwrap();
    let error = service
        .read_reflog_snapshot(&repo, &ReflogScope::Head, &AtomicBool::new(false))
        .unwrap_err();
    assert!(error.to_string().contains("超出仓库目录"));
}

#[test]
fn linked_worktree_reads_its_own_head_and_shared_local_branch_logs() {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "data.txt", "one");
    let oid = git_support::commit_all(&repo, "seed");
    let linked_branch = repo
        .branch("linked", &repo.find_commit(oid).unwrap(), false)
        .unwrap();
    let parent = tempfile::TempDir::new().unwrap();
    let linked_path = parent.path().join("linked");
    let mut options = WorktreeAddOptions::new();
    options.reference(Some(linked_branch.get()));
    let _worktree = repo
        .worktree("linked-fixture", &linked_path, Some(&options))
        .unwrap();
    let linked_repo = Repository::open(&linked_path).unwrap();
    let identity = service.recovery_repo_identity(&linked_repo).unwrap();
    assert_ne!(identity.git_dir, identity.common_dir);
    assert_eq!(identity.common_dir, fs::canonicalize(repo.path()).unwrap());
    put_log(&head_log(&repo), &record(111, "main worktree HEAD"));
    put_log(
        &identity.git_dir.join("logs/HEAD"),
        &record(222, "linked HEAD"),
    );
    put_log(
        &identity.common_dir.join("logs/refs/heads/main"),
        &record(333, "shared main"),
    );
    put_log(
        &identity.common_dir.join("logs/refs/heads/linked"),
        &record(444, "shared linked"),
    );
    put_log(
        &identity.git_dir.join("logs/refs/heads/main"),
        &record(999, "wrong worktree branch path"),
    );
    let before = inventory(repo.path());
    let sources = service.reflog_sources(&linked_repo).unwrap();
    assert_eq!(sources.current_branch.as_deref(), Some("linked"));
    assert_eq!(sources.branches, ["linked", "main"]);
    let head = snapshot(&linked_repo, ReflogScope::Head);
    assert_eq!(head.identity, identity);
    assert_eq!(head.entries.len(), 1);
    assert_eq!(head.entries[0].message, "linked HEAD");
    assert_eq!(
        snapshot(&repo, ReflogScope::Head).entries[0].message,
        "main worktree HEAD"
    );
    assert_eq!(
        snapshot(&linked_repo, ReflogScope::CurrentBranch).entries[0].message,
        "shared linked"
    );
    assert_eq!(
        snapshot(&linked_repo, ReflogScope::LocalBranch("main".into())).entries[0].message,
        "shared main"
    );
    let all = snapshot(&linked_repo, ReflogScope::All);
    assert_eq!((all.sources_read, all.sources_total), (3, 3));
    assert_eq!(
        all.entries
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>(),
        ["shared linked", "shared main", "linked HEAD"]
    );
    assert_eq!(before, inventory(repo.path()));
}
