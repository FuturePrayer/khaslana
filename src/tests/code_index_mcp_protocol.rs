use super::*;
use crate::git::test_support::git_test_support::{init_repo, write_file};

fn response(server: &McpServer, input: Value) -> Value {
    serde_json::from_str(&server.handle_message(&input.to_string()).expect("请求必须有响应")).unwrap()
}

#[test]
fn invalid_envelopes_have_null_id_and_do_not_disappear_as_notifications() {
    let data = tempfile::tempdir().unwrap();
    let server = McpServer::for_multi_test(data.path().into());
    for input in [json!(null), json!([]), json!({}), json!({"jsonrpc":"1.0","method":"ping","id":1}),
        json!({"jsonrpc":"2.0","method":"ping","id":{}})] {
        let reply = response(&server, input);
        assert_eq!(reply["error"]["code"], -32600);
        assert!(reply["id"].is_null());
    }
    let reply: Value = serde_json::from_str(&server.handle_message("{").unwrap()).unwrap();
    assert_eq!(reply["error"]["code"], -32700);
    assert!(reply["id"].is_null());
    assert!(server.handle_message(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
}

#[test]
fn malformed_tool_arguments_fail_before_resolving_or_refreshing_a_repository() {
    let data = tempfile::tempdir().unwrap();
    let server = McpServer::for_multi_test(data.path().into());
    for (name, arguments) in [
        ("index_status", json!({"repo":42})),
        ("search_graph", json!({"query":"target","offset":-1})),
        ("search_graph", json!({"query":"target","offset":u64::MAX})),
        ("search_graph", json!({"query":"target","generation":false})),
        ("search_graph", json!({"query":"target","detail":"unknown"})),
        ("trace_path", json!({"function_name":"target","direction":"sideways"})),
        ("check_index_coverage", json!({"paths":[1]})),
        ("refresh_index", json!({"mode":"unknown"})),
        ("list_projects", json!([])),
        ("nonexistent_tool", json!({})),
    ] {
        let reply = response(&server, json!({"jsonrpc":"2.0","id":"bad","method":"tools/call",
            "params":{"name":name,"arguments":arguments}}));
        assert_eq!(reply["error"]["code"], -32602, "{reply}");
        assert_eq!(reply["id"], "bad");
    }
    assert!(!data.path().join("code-index").exists());
}

#[test]
fn git_directory_and_worktree_root_share_the_same_index_identity() {
    let (dir, _repo, _service) = init_repo();
    let data = tempfile::tempdir().unwrap();
    let root = McpServer::context_from_root(data.path(), dir.path()).unwrap();
    let git_dir = McpServer::context_from_root(data.path(), &dir.path().join(".git")).unwrap();
    assert_eq!(git_dir.repo_root, root.repo_root);
    assert_eq!(git_dir.repo_key, root.repo_key);
    assert_eq!(git_dir.db_path, root.db_path);
}

#[test]
fn bare_repository_is_rejected_instead_of_indexing_git_internal_files() {
    let dir = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    git2::Repository::init_bare(dir.path()).unwrap();
    assert!(McpServer::context_from_root(data.path(), dir.path()).is_err());
}

#[test]
fn missing_index_is_not_reported_as_a_successful_empty_search() {
    let (dir, _repo, _service) = init_repo();
    let data = tempfile::tempdir().unwrap();
    write_file(dir.path(), "lib.rs", "fn existing_symbol() {}\n");
    let server = McpServer::for_test(dir.path(), data.path().join("index.db"));
    let reply = response(&server, json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":"search_graph","arguments":{"query":"existing_symbol"}}}));
    assert_eq!(reply["result"]["isError"], true, "{reply}");
}

#[test]
fn compact_queries_keep_identity_pagination_and_direct_call_evidence() {
    let (dir, _repo, _service) = init_repo();
    let data = tempfile::tempdir().unwrap();
    write_file(dir.path(), "lib.rs", "/// Target documentation.\nfn target() {}\nfn caller() { target(); target(); }\n");
    write_file(dir.path(), "other.rs", "fn cross() { target(); }\n");
    let db = data.path().join("index.db");
    let mut options = super::super::pipeline::PipelineOptions::new(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)), Box::new(|_| {}));
    super::super::run_index(dir.path(), &db, true, &mut options).unwrap();
    let server = McpServer::for_test(dir.path(), db);
    let call = |name: &str, arguments: Value| {
        let reply = response(&server, json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":name,"arguments":arguments}}));
        assert_ne!(reply["result"]["isError"], true, "{reply}");
        let payload: Value = serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(payload, reply["result"]["structuredContent"]);
        payload
    };
    let compact = call("search_graph", json!({"name_pattern":"^target$"}));
    let full = call("search_graph", json!({"name_pattern":"^target$", "detail":"full"}));
    assert!(compact["results"][0]["docstring"].is_null());
    assert_eq!(full["results"][0]["docstring"], "Target documentation.");
    assert_eq!(compact["results"][0]["qualified_name"], full["results"][0]["qualified_name"]);
    assert_eq!(compact["generation"], full["generation"]);
    let inbound = call("trace_path", json!({"function_name":"target","direction":"inbound","depth":1}));
    let original = call("trace_path", json!({"function_name":"target","direction":"inbound","depth":1,"detail":"full"}));
    assert_eq!(inbound["callers_total"], original["callers_total"]);
    assert_eq!(inbound["callers"][0]["call_sites"]["count"], 2);
    assert_eq!(inbound["callers"][0]["call_sites"]["lines"], json!([3,3]));
    assert!(inbound["callers"][0]["call_sites"]["file_path"].is_null());
    assert_eq!(inbound["callers"][0]["file_path"], original["callers"][0]["call_sites"]["file_path"]);
    assert!(inbound["coverage_note"].is_null());
    assert!(inbound.to_string().len() < original.to_string().len());
    let outbound = call("trace_path", json!({"function_name":"cross","direction":"outbound","depth":1}));
    assert_eq!(outbound["callees"][0]["file_path"], "lib.rs");
    assert_eq!(outbound["callees"][0]["call_sites"]["file_path"], "other.rs");
}
