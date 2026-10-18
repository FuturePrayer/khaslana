use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;

use git2::{IndexEntry, IndexTime, ObjectType, Oid, Repository};
use tempfile::TempDir;

use super::*;
use crate::git::test_support::git_test_support as git_support;

fn recovery_fixture() -> (TempDir, Repository, GitService, Oid, Oid) {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "tracked.txt", "before\n");
    let before = git_support::commit_all(&repo, "恢复前提交");
    git_support::write_file(dir.path(), "tracked.txt", "after\n");
    let after = git_support::commit_all(&repo, "恢复后提交");
    (dir, repo, service, before, after)
}

fn request(
    service: &GitService,
    repo: &Repository,
    oid: Oid,
    branch_name: &str,
) -> CreateRecoveryBranchRequest {
    CreateRecoveryBranchRequest {
        identity: service.recovery_repo_identity(repo).unwrap(),
        target_oid: oid.to_string(),
        branch_name: branch_name.to_string(),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PreservedState {
    head_file: Vec<u8>,
    head_oid: Option<Oid>,
    index_file: Option<Vec<u8>>,
    files: BTreeMap<PathBuf, Option<Vec<u8>>>,
}

fn optional_bytes(path: &Path) -> Option<Vec<u8>> {
    match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("读取测试文件失败（{}）：{error}", path.display()),
    }
}

fn preserved_state(repo: &Repository) -> PreservedState {
    let root = repo.workdir().unwrap();
    let files = ["tracked.txt", "untracked.txt"]
        .into_iter()
        .map(|name| (PathBuf::from(name), optional_bytes(&root.join(name))))
        .collect();
    PreservedState {
        head_file: fs::read(repo.path().join("HEAD")).unwrap(),
        head_oid: repo.head().ok().and_then(|head| head.target()),
        index_file: optional_bytes(&repo.path().join("index")),
        files,
    }
}

fn ref_target(repo: &Repository, name: &str) -> Option<Oid> {
    repo.find_reference(&format!("refs/heads/{name}"))
        .unwrap()
        .target()
}

#[test]
fn identity_canonicalizes_all_paths_and_rejects_bare_repository() {
    let (dir, repo, service) = git_support::init_repo();
    let identity = service.recovery_repo_identity(&repo).unwrap();
    assert_eq!(
        identity.worktree_path,
        fs::canonicalize(dir.path()).unwrap()
    );
    assert_eq!(identity.git_dir, fs::canonicalize(repo.path()).unwrap());
    assert_eq!(
        identity.common_dir,
        fs::canonicalize(repo.commondir()).unwrap()
    );

    let bare_dir = TempDir::new().unwrap();
    let bare = Repository::init_bare(bare_dir.path()).unwrap();
    let error = service.recovery_repo_identity(&bare).unwrap_err();
    assert!(error.to_string().contains("不支持裸仓库"));
}

#[test]
fn details_read_both_reflog_sides_and_unreachable_commit_directly() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    let log = repo.reflog("HEAD").unwrap();
    let entry = log.get(0).unwrap();
    assert_eq!(entry.id_old(), before);
    assert_eq!(entry.id_new(), after);
    // 只改变临时 fixture 的引用，让新侧提交不再能从任何当前分支到达。
    repo.reference("refs/heads/main", before, true, "测试不可达提交")
        .unwrap();
    let state = preserved_state(&repo);
    for (oid, summary) in [
        (entry.id_old(), "恢复前提交"),
        (entry.id_new(), "恢复后提交"),
    ] {
        let details = service
            .recovery_commit_details(&repo, &oid.to_string().to_uppercase())
            .unwrap();
        assert_eq!(details.oid, oid.to_string());
        assert_eq!(details.state, RecoveryObjectState::Available);
        let commit = details.commit.unwrap();
        assert_eq!(commit.oid, oid.to_string());
        assert_eq!(commit.summary, summary);
        assert_eq!(commit.author, "Test User");
    }
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn details_distinguish_zero_missing_blob_tree_tag_and_unreadable_commit() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let missing = Oid::from_str("1234567890123456789012345678901234567890").unwrap();
    let blob = repo.blob(b"not a commit").unwrap();
    let tree = repo.find_commit(after).unwrap().tree_id();
    let object = repo.find_object(after, None).unwrap();
    let tag = repo
        .tag(
            "recovery-test",
            &object,
            &repo.signature().unwrap(),
            "tag",
            false,
        )
        .unwrap();
    let malformed = repo
        .odb()
        .unwrap()
        .write(ObjectType::Commit, b"broken commit\n")
        .unwrap();
    let state = preserved_state(&repo);
    for (oid, expected) in [
        (Oid::ZERO_SHA1, RecoveryObjectState::Zero),
        (missing, RecoveryObjectState::Missing),
        (blob, RecoveryObjectState::NonCommit),
        (tree, RecoveryObjectState::NonCommit),
        (tag, RecoveryObjectState::NonCommit),
    ] {
        let details = service
            .recovery_commit_details(&repo, &oid.to_string())
            .unwrap();
        assert_eq!(details.state, expected);
        assert!(details.commit.is_none());
    }
    let details = service
        .recovery_commit_details(&repo, &malformed.to_string())
        .unwrap();
    assert!(matches!(details.state, RecoveryObjectState::Unreadable(_)));
    assert!(details.commit.is_none());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn details_and_creation_reject_prefixes_invalid_oids_and_revision_expressions() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let complete = after.to_string();
    let invalid = vec![
        String::new(),
        complete[..8].to_string(),
        "HEAD".into(),
        "HEAD~1".into(),
        "refs/heads/main".into(),
        format!("{complete}^0"),
        format!(" {complete}"),
        format!("{complete}\n"),
        "g".repeat(40),
        "a".repeat(64),
    ];
    let state = preserved_state(&repo);
    for oid in invalid {
        assert!(
            service.recovery_commit_details(&repo, &oid).is_err(),
            "{oid:?}"
        );
        let mut request = request(&service, &repo, after, "invalid-target");
        request.target_oid = oid.clone();
        assert!(
            service.create_recovery_branch(&repo, &request).is_err(),
            "{oid:?}"
        );
    }
    assert!(repo.find_reference("refs/heads/invalid-target").is_err());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn creation_rejects_zero_missing_noncommit_and_unreadable_targets() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let missing = Oid::from_str("1234567890123456789012345678901234567890").unwrap();
    let blob = repo.blob(b"blob target").unwrap();
    let tree = repo.find_commit(after).unwrap().tree_id();
    let malformed = repo
        .odb()
        .unwrap()
        .write(ObjectType::Commit, b"broken commit\n")
        .unwrap();
    let state = preserved_state(&repo);
    for (index, oid) in [Oid::ZERO_SHA1, missing, blob, tree, malformed]
        .into_iter()
        .enumerate()
    {
        let name = format!("invalid-kind-{index}");
        assert!(
            service
                .create_recovery_branch(&repo, &request(&service, &repo, oid, &name))
                .is_err()
        );
        assert!(repo.find_reference(&format!("refs/heads/{name}")).is_err());
    }
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn creating_old_and_new_targets_preserves_staged_dirty_and_untracked_bytes() {
    let (dir, repo, service, before, after) = recovery_fixture();
    git_support::write_file(dir.path(), "tracked.txt", "staged\n");
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("tracked.txt")).unwrap();
    index.write().unwrap();
    drop(index);
    git_support::write_bytes(dir.path(), "tracked.txt", b"dirty\r\n\0tail");
    git_support::write_bytes(dir.path(), "untracked.txt", b"untracked\0\xff\r\n");
    let state = preserved_state(&repo);
    for (name, oid) in [("recover-before", before), ("recover-after", after)] {
        let mut request = request(&service, &repo, oid, name);
        request.target_oid = request.target_oid.to_uppercase();
        let outcome = service.create_recovery_branch(&repo, &request).unwrap();
        assert_eq!(outcome.branch_name, name);
        assert_eq!(outcome.target_oid, oid.to_string());
        assert_eq!(outcome.verification_warning, None);
        assert_eq!(ref_target(&repo, name), Some(oid));
        assert_eq!(preserved_state(&repo), state);
    }
}

#[test]
fn branch_validation_is_shared_and_creation_rechecks_names() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let state = preserved_state(&repo);
    for name in [
        "", " ", "-option", "a..b", "bad.lock", "a:b", "a\\b", "a//b", "a@{b", "a\nb",
    ] {
        assert!(
            matches!(
                service.validate_recovery_branch_name(name),
                Err(GitError::InvalidBranchName(_))
            ),
            "{name:?}"
        );
        assert!(
            service
                .create_recovery_branch(&repo, &request(&service, &repo, after, name))
                .is_err(),
            "{name:?}"
        );
    }
    assert!(
        service
            .validate_recovery_branch_name("recovery/合法名称")
            .is_ok()
    );
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn existing_loose_and_symbolic_refs_are_never_overwritten_even_for_same_oid() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    repo.reference("refs/heads/occupied", before, false, "fixture")
        .unwrap();
    repo.reference_symbolic(
        "refs/heads/occupied-symbolic",
        "refs/heads/missing",
        false,
        "fixture",
    )
    .unwrap();
    let state = preserved_state(&repo);
    for oid in [before, after] {
        for name in ["occupied", "occupied-symbolic"] {
            assert!(
                service
                    .create_recovery_branch(&repo, &request(&service, &repo, oid, name))
                    .is_err()
            );
        }
        assert_eq!(ref_target(&repo, "occupied"), Some(before));
        assert_eq!(
            repo.find_reference("refs/heads/occupied-symbolic")
                .unwrap()
                .symbolic_target()
                .unwrap(),
            Some("refs/heads/missing")
        );
    }
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn loose_directory_file_conflicts_are_rejected_in_both_directions() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    repo.reference("refs/heads/parent", before, false, "fixture")
        .unwrap();
    repo.reference("refs/heads/container/child", before, false, "fixture")
        .unwrap();
    let state = preserved_state(&repo);
    for name in ["parent/child", "container"] {
        assert!(
            service
                .create_recovery_branch(&repo, &request(&service, &repo, after, name))
                .is_err()
        );
        assert!(repo.find_reference(&format!("refs/heads/{name}")).is_err());
    }
    assert_eq!(ref_target(&repo, "parent"), Some(before));
    assert_eq!(ref_target(&repo, "container/child"), Some(before));
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn packed_refs_and_packed_directory_file_conflicts_are_rejected() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    let packed_path = repo.commondir().join("packed-refs");
    let packed_bytes = format!(
        "# pack-refs with: peeled fully-peeled sorted\n{before} refs/heads/container/child\n{before} refs/heads/packed\n"
    ).into_bytes();
    fs::write(&packed_path, &packed_bytes).unwrap();
    let state = preserved_state(&repo);
    for name in ["packed", "packed/child", "container"] {
        assert!(
            service
                .create_recovery_branch(&repo, &request(&service, &repo, after, name))
                .is_err()
        );
    }
    assert_eq!(fs::read(&packed_path).unwrap(), packed_bytes);
    assert_eq!(ref_target(&repo, "packed"), Some(before));
    assert_eq!(ref_target(&repo, "container/child"), Some(before));
    assert!(!repo.commondir().join("refs/heads/packed").exists());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn an_existing_ref_lock_is_reported_without_removing_it() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let lock_path = repo.commondir().join("refs/heads/locked.lock");
    fs::write(&lock_path, b"another writer owns this lock\n").unwrap();
    let state = preserved_state(&repo);
    assert!(
        service
            .create_recovery_branch(&repo, &request(&service, &repo, after, "locked"))
            .is_err()
    );
    assert_eq!(
        fs::read(&lock_path).unwrap(),
        b"another writer owns this lock\n"
    );
    assert!(repo.find_reference("refs/heads/locked").is_err());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn detached_head_does_not_block_creation_or_move() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    repo.set_head_detached(after).unwrap();
    let state = preserved_state(&repo);
    service
        .create_recovery_branch(&repo, &request(&service, &repo, before, "from-detached"))
        .unwrap();
    assert_eq!(ref_target(&repo, "from-detached"), Some(before));
    assert!(repo.head_detached().unwrap());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn unborn_head_target_is_rejected_but_other_names_are_allowed() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    repo.set_head("refs/heads/unborn").unwrap();
    let state = preserved_state(&repo);
    let error = service
        .create_recovery_branch(&repo, &request(&service, &repo, before, "unborn"))
        .unwrap_err();
    assert!(error.to_string().contains("HEAD"));
    assert!(repo.find_reference("refs/heads/unborn").is_err());
    assert_eq!(preserved_state(&repo), state);
    service
        .create_recovery_branch(
            &repo,
            &request(&service, &repo, before, "other-unborn-recovery"),
        )
        .unwrap();
    assert_eq!(ref_target(&repo, "other-unborn-recovery"), Some(before));
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn unborn_head_symbolic_chain_is_followed_without_mutating_it() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    repo.reference_symbolic(
        "refs/heads/head-alias",
        "refs/heads/chain-unborn",
        false,
        "fixture",
    )
    .unwrap();
    fs::write(repo.path().join("HEAD"), b"ref: refs/heads/head-alias\n").unwrap();
    let state = preserved_state(&repo);
    assert!(
        service
            .create_recovery_branch(&repo, &request(&service, &repo, before, "chain-unborn"))
            .is_err()
    );
    assert!(repo.find_reference("refs/heads/chain-unborn").is_err());
    service
        .create_recovery_branch(&repo, &request(&service, &repo, before, "chain-other"))
        .unwrap();
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn cyclic_or_excessively_long_head_symbolic_chains_fail_closed() {
    for cycle in [true, false] {
        let (_dir, repo, service, before, _after) = recovery_fixture();
        let count = if cycle {
            2
        } else {
            HEAD_SYMBOLIC_DEPTH_LIMIT + 2
        };
        for index in 0..count {
            let target = if cycle && index + 1 == count {
                0
            } else {
                index + 1
            };
            repo.reference_symbolic(
                &format!("refs/heads/chain-{index}"),
                &format!("refs/heads/chain-{target}"),
                false,
                "fixture",
            )
            .unwrap();
        }
        fs::write(repo.path().join("HEAD"), b"ref: refs/heads/chain-0\n").unwrap();
        let state = preserved_state(&repo);
        let error = service
            .create_recovery_branch(&repo, &request(&service, &repo, before, "head-guard"))
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains(if cycle { "循环" } else { "过长" })
        );
        assert!(repo.find_reference("refs/heads/head-guard").is_err());
        assert_eq!(preserved_state(&repo), state);
    }
}

#[test]
fn existing_index_conflicts_do_not_block_or_change_recovery() {
    let (dir, repo, service, before, _after) = recovery_fixture();
    let mut index = repo.index().unwrap();
    index.remove(Path::new("tracked.txt"), 0).unwrap();
    for stage in 1u16..=3 {
        let entry = IndexEntry {
            ctime: IndexTime::new(0, 0),
            mtime: IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: 0,
            id: repo.blob(format!("stage {stage}\n").as_bytes()).unwrap(),
            flags: stage << 12,
            flags_extended: 0,
            path: b"tracked.txt".to_vec(),
        };
        index.add(&entry).unwrap();
    }
    assert!(index.has_conflicts());
    index.write().unwrap();
    drop(index);
    git_support::write_file(
        dir.path(),
        "tracked.txt",
        "<<<<<<< ours\nconflict\n=======\ntheirs\n>>>>>>> theirs\n",
    );
    git_support::write_file(dir.path(), "untracked.txt", "keep conflict fixture\n");
    let state = preserved_state(&repo);
    service
        .create_recovery_branch(&repo, &request(&service, &repo, before, "with-conflicts"))
        .unwrap();
    assert_eq!(ref_target(&repo, "with-conflicts"), Some(before));
    assert!(repo.index().unwrap().has_conflicts());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn identity_is_rechecked_for_each_of_its_components() {
    let (_dir, repo, service, _before, after) = recovery_fixture();
    let state = preserved_state(&repo);
    for component in 0..3 {
        let mut request = request(&service, &repo, after, "wrong-identity");
        match component {
            0 => request.identity.worktree_path.push("other"),
            1 => request.identity.git_dir.push("other"),
            _ => request.identity.common_dir.push("other"),
        }
        let error = service.create_recovery_branch(&repo, &request).unwrap_err();
        assert!(error.to_string().contains("仓库身份"));
    }
    assert!(repo.find_reference("refs/heads/wrong-identity").is_err());
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn concurrent_independent_repositories_create_exactly_one_branch() {
    let (dir, repo, service, before, _after) = recovery_fixture();
    let state = preserved_state(&repo);
    let request = request(&service, &repo, before, "racing-recovery");
    let barrier = Arc::new(Barrier::new(8));
    let workers = (0..8)
        .map(|_| {
            let path = dir.path().to_path_buf();
            let request = request.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let repo = Repository::open(path).unwrap();
                let service = git_support::service();
                barrier.wait();
                service.create_recovery_branch(&repo, &request).is_ok()
            })
        })
        .collect::<Vec<_>>();
    let successes = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .filter(|success| *success)
        .count();
    assert_eq!(successes, 1);
    assert_eq!(ref_target(&repo, "racing-recovery"), Some(before));
    assert_eq!(preserved_state(&repo), state);
}

fn linked_fixture(repo: &Repository) -> (TempDir, Repository) {
    let dir = TempDir::new().unwrap();
    let linked_path = dir.path().join("linked");
    let worktree = repo.worktree("linked-fixture", &linked_path, None).unwrap();
    let linked = Repository::open_from_worktree(&worktree).unwrap();
    (dir, linked)
}

#[test]
fn linked_worktrees_share_the_write_mutex_and_created_ref() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    let (_linked_dir, linked) = linked_fixture(&repo);
    let main_identity = service.recovery_repo_identity(&repo).unwrap();
    let linked_identity = service.recovery_repo_identity(&linked).unwrap();
    assert_ne!(main_identity.worktree_path, linked_identity.worktree_path);
    assert_ne!(main_identity.git_dir, linked_identity.git_dir);
    assert_eq!(main_identity.common_dir, linked_identity.common_dir);
    assert!(Arc::ptr_eq(
        &recovery_write_lock(&main_identity.common_dir),
        &recovery_write_lock(&linked_identity.common_dir)
    ));
    let main_state = preserved_state(&repo);
    let linked_state = preserved_state(&linked);
    service
        .create_recovery_branch(
            &linked,
            &request(&service, &linked, before, "shared-recovery"),
        )
        .unwrap();
    assert_eq!(ref_target(&repo, "shared-recovery"), Some(before));
    assert_eq!(ref_target(&linked, "shared-recovery"), Some(before));
    let wrong_request = CreateRecoveryBranchRequest {
        identity: main_identity,
        target_oid: before.to_string(),
        branch_name: "wrong-worktree".into(),
    };
    assert!(
        service
            .create_recovery_branch(&linked, &wrong_request)
            .is_err()
    );
    assert_eq!(preserved_state(&repo), main_state);
    assert_eq!(preserved_state(&linked), linked_state);
}

#[test]
fn unborn_targets_of_other_linked_and_main_worktrees_are_protected() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    let (_linked_dir, linked) = linked_fixture(&repo);
    linked.set_head("refs/heads/linked-unborn").unwrap();
    let main_state = preserved_state(&repo);
    let linked_state = preserved_state(&linked);
    assert!(
        service
            .create_recovery_branch(&repo, &request(&service, &repo, before, "linked-unborn"))
            .is_err()
    );
    service
        .create_recovery_branch(&repo, &request(&service, &repo, before, "linked-other"))
        .unwrap();
    assert_eq!(preserved_state(&repo), main_state);
    assert_eq!(preserved_state(&linked), linked_state);

    repo.set_head("refs/heads/main-unborn").unwrap();
    let main_state = preserved_state(&repo);
    let linked_state = preserved_state(&linked);
    assert!(
        service
            .create_recovery_branch(&linked, &request(&service, &linked, before, "main-unborn"))
            .is_err()
    );
    assert!(repo.find_reference("refs/heads/main-unborn").is_err());
    assert!(repo.find_reference("refs/heads/linked-unborn").is_err());
    assert_eq!(preserved_state(&repo), main_state);
    assert_eq!(preserved_state(&linked), linked_state);
}

#[test]
fn post_creation_read_error_is_a_warning_and_keeps_the_created_ref() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    let state = preserved_state(&repo);
    let outcome = service
        .create_recovery_branch_with_verifier(
            &repo,
            &request(&service, &repo, before, "verification-error"),
            |repo, fullname| {
                assert_eq!(
                    repo.find_reference(fullname).unwrap().target(),
                    Some(before)
                );
                Err(git2::Error::from_str("测试注入读取失败"))
            },
        )
        .unwrap();
    assert!(
        outcome
            .verification_warning
            .unwrap()
            .contains("无法读回验证")
    );
    assert_eq!(outcome.target_oid, before.to_string());
    assert_eq!(ref_target(&repo, "verification-error"), Some(before));
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn post_creation_verifier_panic_is_a_warning_and_does_not_poison_the_write_lock() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    let state = preserved_state(&repo);
    let outcome = service
        .create_recovery_branch_with_verifier(
            &repo,
            &request(&service, &repo, before, "verification-panic"),
            |_, _| panic!("测试注入核验 panic"),
        )
        .unwrap();
    assert!(outcome.verification_warning.unwrap().contains("意外中断"));
    assert_eq!(outcome.target_oid, before.to_string());
    assert_eq!(ref_target(&repo, "verification-panic"), Some(before));
    service
        .create_recovery_branch(
            &repo,
            &request(&service, &repo, before, "after-verification-panic"),
        )
        .unwrap();
    assert_eq!(ref_target(&repo, "after-verification-panic"), Some(before));
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn post_creation_mismatch_is_a_warning_without_rollback_or_overwrite() {
    let (_dir, repo, service, before, after) = recovery_fixture();
    let state = preserved_state(&repo);
    let outcome = service
        .create_recovery_branch_with_verifier(
            &repo,
            &request(&service, &repo, before, "verification-mismatch"),
            |repo, fullname| {
                // 模拟创建后其它工具已更新引用；恢复服务必须保留该更新。
                repo.reference(fullname, after, true, "测试并发更新")?;
                read_recovery_reference(repo, fullname)
            },
        )
        .unwrap();
    assert!(outcome.verification_warning.unwrap().contains("不一致"));
    assert_eq!(outcome.target_oid, before.to_string());
    assert_eq!(ref_target(&repo, "verification-mismatch"), Some(after));
    assert_eq!(preserved_state(&repo), state);
}

#[test]
fn post_creation_symbolic_ref_is_a_warning_and_remains_symbolic() {
    let (_dir, repo, service, before, _after) = recovery_fixture();
    let state = preserved_state(&repo);
    let outcome = service
        .create_recovery_branch_with_verifier(
            &repo,
            &request(&service, &repo, before, "verification-symbolic"),
            |repo, fullname| {
                repo.reference_symbolic(fullname, "refs/heads/main", true, "测试并发符号引用")?;
                read_recovery_reference(repo, fullname)
            },
        )
        .unwrap();
    assert!(
        outcome
            .verification_warning
            .unwrap()
            .contains("不是直接引用")
    );
    assert_eq!(
        repo.find_reference("refs/heads/verification-symbolic")
            .unwrap()
            .symbolic_target()
            .unwrap(),
        Some("refs/heads/main")
    );
    assert_eq!(preserved_state(&repo), state);
}
