use super::*;

fn remote(name: &str) -> RemoteInfo {
    RemoteInfo {
        name: name.into(),
        url: format!("https://example.com/{name}.git"),
        credential_record_id: None,
    }
}

/// 无 snapshot 或远端列表不含 origin 时，新增远端默认预填 origin。
#[test]
fn default_new_remote_name_prefills_origin_when_absent() {
    assert_eq!(default_new_remote_name(None), "origin");
    let snapshot = RepositorySnapshot {
        remotes: vec![remote("upstream")],
        ..Default::default()
    };
    assert_eq!(default_new_remote_name(Some(&snapshot)), "origin");
}

/// 已存在 origin 时留空，避免重名。
#[test]
fn default_new_remote_name_empty_when_origin_exists() {
    let snapshot = RepositorySnapshot {
        remotes: vec![remote("origin"), remote("upstream")],
        ..Default::default()
    };
    assert_eq!(default_new_remote_name(Some(&snapshot)), "");
}
