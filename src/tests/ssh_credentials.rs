use super::{http_remote_to_ssh, ssh_remote_to_https};

#[test]
fn credential_remote_round_trips_between_https_and_ssh() {
    let original = "https://gitee.com/cathaylife/cxlsvr.git";
    let mut remote = original.to_string();
    for _ in 0..5 {
        remote = http_remote_to_ssh(&remote, "git").unwrap();
        assert_eq!(remote, "git@gitee.com:cathaylife/cxlsvr.git");
        assert_eq!(http_remote_to_ssh(&remote, "git"), None);
        remote = ssh_remote_to_https(&remote).unwrap();
        assert_eq!(remote, original);
        assert_eq!(ssh_remote_to_https(&remote), None);
    }
}

#[test]
fn converts_common_ssh_remotes_to_https() {
    for (input, expected) in [
        (
            "git@gitee.com:cathaylife/cxlsvr.git",
            "https://gitee.com/cathaylife/cxlsvr.git",
        ),
        (
            "alice@example.com:team/nested/repo.git",
            "https://example.com/team/nested/repo.git",
        ),
        (
            "ssh://alice@example.com/team/repo.git",
            "https://example.com/team/repo.git",
        ),
        (
            "ssh://example.com/team/repo.git",
            "https://example.com/team/repo.git",
        ),
        (
            "ssh://git@example.com:2222/team/repo.git",
            "https://example.com/team/repo.git",
        ),
        (
            "git@[2001:db8::1]:team/repo.git",
            "https://[2001:db8::1]/team/repo.git",
        ),
        (
            "ssh://git@[2001:db8::1]:2222/team/repo.git",
            "https://[2001:db8::1]/team/repo.git",
        ),
        (
            "  git@example.com:team/repo.git  ",
            "https://example.com/team/repo.git",
        ),
    ] {
        assert_eq!(
            ssh_remote_to_https(input).as_deref(),
            Some(expected),
            "{input}"
        );
    }
}

#[test]
fn refuses_non_ssh_and_malformed_remotes() {
    for input in [
        "",
        "   ",
        "https://example.com/team/repo.git",
        "http://example.com/repo",
        "example.com",
        "example.com/repo",
        "C:/repos/repo",
        "../repo",
        "git@:repo",
        "@example.com:repo",
        "git@example.com:",
        "git@example.com: ",
        "ssh:///repo",
        "ssh://example.com/",
        "ssh://@example.com/repo",
        "ssh://git@@example.com/repo",
        "ssh://git@example.com:bad/repo",
        "ssh://git@example.com:0/repo",
        "ssh://git@example.com:65536/repo",
        "ssh://git@[broken]/repo",
        "git@[2001:db8::1:repo",
        "git@example.com/repo:other",
        "ssh://git@example.com/repo?query",
        "ssh://git@example.com/repo#fragment",
        "ssh://git@example.com/repo\\file",
    ] {
        assert_eq!(ssh_remote_to_https(input), None, "{input}");
    }
}

#[test]
fn converts_current_ssh_value_after_manual_edit() {
    let generated = http_remote_to_ssh("https://gitee.com/old/repo.git", "git").unwrap();
    assert_eq!(generated, "git@gitee.com:old/repo.git");
    assert_eq!(
        ssh_remote_to_https("git@gitee.com:new/edited.git").as_deref(),
        Some("https://gitee.com/new/edited.git")
    );
}
