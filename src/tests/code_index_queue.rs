use super::*;

#[test]
fn fifo_remove_reopen_moves_to_tail_and_merge_upgrades_in_place() {
    let mut queue = CodeIndexQueue::default();
    queue.push("a", false, false, true);
    queue.push("b", false, false, true);
    queue.push("c", false, true, true);
    queue.push("b", true, true, true);
    assert_eq!(queue.position("b"), Some(2));
    queue.remove("b");
    assert_eq!(queue.position("c"), Some(2));
    queue.push("b", false, false, true);
    queue.push("b", true, true, true);
    assert_eq!(queue.pop().unwrap().repo_key, "a");
    assert_eq!(queue.pop().unwrap().repo_key, "c");
    assert_eq!(queue.pop().unwrap(), IndexRequest { repo_key: "b".into(), force_full: true, user_initiated: true, require_enabled: true });
    assert!(queue.pop().is_none());
}

#[test]
fn dequeue_requires_enabled_only_for_auto_and_enable_requests() {
    let mut queue = CodeIndexQueue::default();
    queue.push("auto", false, false, true);
    queue.push("enable", true, true, true);
    queue.push("manual-disabled", true, true, false);
    let auto = queue.pop().unwrap();
    assert!(!auto.eligible(false));
    assert!(auto.eligible(true));
    let enable = queue.pop().unwrap();
    assert!(!enable.eligible(false));
    assert!(enable.eligible(true));
    assert!(queue.pop().unwrap().eligible(false));
}

#[test]
fn merged_manual_request_keeps_disabled_eligibility_in_both_orders() {
    for manual_first in [false, true] {
        let mut queue = CodeIndexQueue::default();
        queue.push("a", false, manual_first, !manual_first);
        queue.push("b", true, true, true);
        queue.push("a", true, !manual_first, manual_first);
        let merged = queue.pop().unwrap();
        assert_eq!(merged.repo_key, "a");
        assert!(merged.force_full);
        assert!(merged.user_initiated);
        assert!(merged.eligible(false));
        assert_eq!(queue.pop().unwrap().repo_key, "b");
    }
}

#[test]
fn repeated_auto_refresh_keeps_only_one_followup() {
    let mut queue = CodeIndexQueue::default();
    for _ in 0..10 { queue.push("active", false, false, true); }
    queue.push("other", true, true, true);
    assert_eq!(queue.pop().unwrap().repo_key, "active");
    assert_eq!(queue.pop().unwrap().repo_key, "other");
    assert!(queue.pop().is_none());
}
