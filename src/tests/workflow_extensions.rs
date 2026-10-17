#![cfg(windows)]

use std::time::Duration;

use serde_json::json;

use super::*;
use super::extensions::{WorkflowExternalGrant, WorkflowMcpConfig, discover_skill_packages, register_external_actions,
    register_external_actions_with_ai};
use super::extensions::{preview_skill_folder, install_skill_folder, remove_skill_package,
    load_user_mcp_config, load_mcp_config, upsert_user_mcp_server, remove_user_mcp_server,
    inspect_mcp_server_tools};
use crate::git::test_support::git_test_support as git_support;

#[path = "workflow_branch_browser.rs"]
mod branch_browser;

fn test_config(marker: &Path) -> WorkflowMcpConfig {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/fixtures/workflow_mcp_server.ps1");
    serde_json::from_value(json!({
        "servers": {
            "fixture": {
                "command": "powershell.exe",
                "args": ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                    "-File", script, marker],
                "tools": {
                    "echo": {"access": "read"},
                    "write_file": {"access": "write"},
                    "set_state": {"access": "write"},
                    "get_state": {"access": "read"},
                    "stall": {"access": "read"}
                }
            }
        }
    })).unwrap()
}

fn grant_for(steps: &str) -> WorkflowExternalGrant {
    let content = format!("{{version:2, defaults:{{requireCleanWorktree:false}}, steps:[{steps}]}}");
    let definition = parse_workflow_json5(&content).unwrap();
    WorkflowExternalGrant::for_definition(&definition).unwrap().unwrap()
}

#[test]
fn builtin_edge_exposes_all_25_default_tools_without_extra_capabilities() {
    let expected = ["browser_click", "browser_close", "browser_console_messages", "browser_drag", "browser_drop",
        "browser_emulate_media", "browser_evaluate", "browser_file_upload", "browser_fill_form", "browser_find",
        "browser_handle_dialog", "browser_hover", "browser_navigate", "browser_navigate_back", "browser_network_request",
        "browser_network_requests", "browser_press_key", "browser_resize", "browser_run_code_unsafe", "browser_select_option",
        "browser_snapshot", "browser_take_screenshot", "browser_type", "browser_wait_for", "browser_tabs"];
    let server = extensions::builtin_browser_server();
    assert!(server.enabled);
    assert_eq!(server.tools.len(), expected.len());
    for tool in expected { assert!(server.tools.contains_key(tool), "{tool}"); }
    assert!(!server.args.iter().any(|arg| arg == "--caps"));
}

#[test]
fn only_pure_js_and_builtin_edge_workflows_skip_confirmation() {
    let data = tempfile::tempdir().unwrap();
    let mut config = load_mcp_config(data.path()).unwrap();
    let pure = grant_for("{op:'invoke',id:'js',uses:'js.run',with:{script:'return 1;'}}");
    assert!(!pure.requires_confirmation(&config).unwrap());
    for action in ["js.run", "skill.run"] {
        let parameters = if action == "js.run" { "script:'return 1;'" } else { "skill:'demo',task:'填表'" };
        let grant = grant_for(&format!("{{op:'invoke',id:'browser',uses:'{action}',with:{{{parameters},tools:[{{server:'browser.edge',tool:'browser_fill_form'}}]}}}}"));
        assert!(!grant.requires_confirmation(&config).unwrap());
    }
    config.servers.extend(test_config(&data.path().join("unused")).servers);
    let mixed = grant_for("{op:'invoke',id:'js',uses:'js.run',with:{script:'return 1;',tools:[{server:'browser.edge',tool:'browser_snapshot'},{server:'fixture',tool:'echo'}]}}");
    assert!(mixed.requires_confirmation(&config).unwrap());
    let named = grant_for("{op:'invoke',id:'read',uses:'mcp.call',with:{server:'browser.edge',tool:'browser_snapshot'}}");
    config.servers.get_mut("browser.edge").unwrap().command = "custom-server.exe".into();
    assert!(named.requires_confirmation(&config).unwrap());
}

#[test]
fn release_form_verifies_before_single_submit_and_never_retries_uncertain_results() {
    for (name, expected_submits, succeeds) in [("success", 1, true), ("wrong-values", 0, false), ("uncertain-submit", 1, false), ("existing-branch", 0, false)] {
        let marker_dir = tempfile::tempdir().unwrap();
        let marker = marker_dir.path().join(name);
        let mut config = test_config(&marker);
        for tool in ["browser_navigate", "browser_evaluate", "browser_type", "browser_select_option", "browser_take_screenshot", "browser_click"] {
            config.servers.get_mut("fixture").unwrap().tools.insert(tool.into(), extensions::WorkflowMcpTool { access: extensions::WorkflowToolAccess::Write });
        }
        let content = include_str!("../../docs/examples/release-form/create-release-submit.json5").replace("browser.edge", "fixture");
        let definition = parse_workflow_json5(&content).unwrap();
        let registry = make_registry(config, WorkflowExternalGrant::for_definition(&definition).unwrap());
        let (dir, mut repo, service) = git_support::init_repo();
        git_support::write_file(dir.path(), "README.md", "initial");
        git_support::commit_all(&repo, "initial");
        if name == "existing-branch" { repo.branch("release/test", &repo.head().unwrap().peel_to_commit().unwrap(), false).unwrap(); }
        let options = WorkflowRunOptions { input_vars: BTreeMap::from([
            ("base".into(), "main".into()), ("target".into(), "release/test".into()),
            ("summary".into(), "第一行\n第二行".into()), ("environment".into(), "2".into()),
            ("completionDate".into(), "2026-10-12".into()),
        ]), ..Default::default() };
        let outcome = WorkflowExecutor::with_actions(&service, &registry).run(&mut repo, &definition, options, |_| {});
        assert_eq!(outcome.is_ok(), succeeds, "{name}: {outcome:?}");
        let writes = std::fs::read_to_string(&marker).unwrap_or_default();
        assert_eq!(writes.lines().filter(|line| *line == "submit").count(), expected_submits, "{name}");
        assert!(repo.find_branch("release/test", git2::BranchType::Local).is_ok());
    }
}

#[test]
#[ignore = "需要本机 Edge、已安装的浏览器组件和联网；只提交 Selenium 公开测试表单"]
fn release_form_real_edge_creates_submits_and_cleans_snapshots() {
    let data_dir = std::path::PathBuf::from(std::env::var("KHASLANA_BROWSER_TEST_DATA_DIR").expect("指定已有浏览器组件的数据目录"));
    assert!(browser_runtime::ready(&data_dir));
    let config_dir = tempfile::tempdir().unwrap();
    let config = load_mcp_config(config_dir.path()).unwrap();
    let definition = parse_workflow_json5(include_str!("../../docs/examples/release-form/create-release-submit.json5")).unwrap();
    let grant = WorkflowExternalGrant::for_definition(&definition).unwrap();
    assert!(!grant.as_ref().unwrap().requires_confirmation(&config).unwrap());
    let (dir, mut repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "README.md", "initial");
    git_support::commit_all(&repo, "initial");
    let options = WorkflowRunOptions { input_vars: BTreeMap::from([
        ("base".into(), "main".into()), ("target".into(), "release/browser-test".into()),
        ("summary".into(), "第一行测试\n第二行说明".into()), ("environment".into(), "2".into()),
        ("completionDate".into(), "2026-10-12".into()),
    ]), ..Default::default() };
    let mut registry = WorkflowActionRegistry::default();
    register_external_actions_with_ai(&mut registry, config, grant, None, None, Some(&data_dir)).unwrap();
    WorkflowExecutor::with_actions(&service, &registry).run(&mut repo, &definition, options, |event| {
        if let WorkflowProgressEvent::StepStarted { index, .. } = event { println!("开始步骤 {index}"); }
    }).unwrap();
    assert_eq!(repo.head().unwrap().shorthand().unwrap(), "release/browser-test");
    assert!(registry.finish().is_empty());
    let sessions = data_dir.join("workflow-browser-sessions");
    for entry in std::fs::read_dir(sessions).unwrap().flatten() {
        if !entry.path().join("snapshots.json").exists() {
            assert!(std::fs::read_dir(entry.path().join("snapshots")).unwrap().next().is_none());
            for output in std::fs::read_dir(entry.path().join("outputs")).unwrap().flatten() {
                assert!(!output.file_name().to_string_lossy().ends_with(".yml"));
            }
        }
    }
}

fn make_registry(config: WorkflowMcpConfig, grant: Option<WorkflowExternalGrant>) -> WorkflowActionRegistry {
    let mut registry = WorkflowActionRegistry::default();
    register_external_actions(&mut registry, config, grant).unwrap();
    registry
}

fn test_skill(dir: &Path) {
    let package = dir.join("workflow-skills/fixture-skill");
    std::fs::create_dir_all(package.join("references")).unwrap();
    std::fs::write(package.join("SKILL.md"),
        "---\nname: fixture-skill\ndescription: 测试工具循环\n---\n读取 fixture 工具。\n").unwrap();
    std::fs::write(package.join("references/guide.md"), "只读取 hello。\n").unwrap();
}

#[test]
fn skill_import_requires_confirmed_content_and_can_be_removed() {
    let source_dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir().unwrap();
    test_skill(source_dir.path());
    let source = source_dir.path().join("workflow-skills/fixture-skill");
    let preview = preview_skill_folder(&source).unwrap();
    std::fs::write(source.join("references/guide.md"), "内容已改变\n").unwrap();
    assert!(install_skill_folder(data_dir.path(), &source, &preview.content_sha256)
        .unwrap_err().to_string().contains("内容已变化"));
    assert!(discover_skill_packages(data_dir.path()).unwrap().is_empty());

    let preview = preview_skill_folder(&source).unwrap();
    install_skill_folder(data_dir.path(), &source, &preview.content_sha256).unwrap();
    assert_eq!(discover_skill_packages(data_dir.path()).unwrap().len(), 1);
    remove_skill_package(data_dir.path(), "fixture-skill").unwrap();
    assert!(discover_skill_packages(data_dir.path()).unwrap().is_empty());
}

#[test]
fn visual_mcp_config_roundtrip_keeps_builtin_out_of_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut server = test_config(&dir.path().join("unused"))
        .servers.remove("fixture").unwrap();
    server.tools.retain(|name, _| name == "echo");
    upsert_user_mcp_server(dir.path(), None, "fixture", server).unwrap();
    assert!(load_user_mcp_config(dir.path()).unwrap().servers.contains_key("fixture"));
    assert!(load_mcp_config(dir.path()).unwrap().servers.contains_key("browser.edge"));
    let saved = std::fs::read_to_string(dir.path().join("workflow-mcp.json5")).unwrap();
    assert!(!saved.contains("browser.edge"));
    remove_user_mcp_server(dir.path(), "fixture").unwrap();
    assert!(load_user_mcp_config(dir.path()).unwrap().servers.is_empty());
}

#[test]
fn visual_mcp_config_rejects_builtin_name_and_empty_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let mut server = test_config(&dir.path().join("unused"))
        .servers.remove("fixture").unwrap();
    assert!(upsert_user_mcp_server(dir.path(), None, "browser.edge", server.clone()).is_err());
    server.tools.clear();
    assert!(upsert_user_mcp_server(dir.path(), None, "fixture", server).is_err());
    assert!(!dir.path().join("workflow-mcp.json5").exists());
}

#[cfg(windows)]
#[test]
fn mcp_connection_test_lists_tools_without_invoking_them() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("should-not-write.txt");
    let server = test_config(&marker).servers.remove("fixture").unwrap();
    let tools = inspect_mcp_server_tools(&server.command, &server.args).unwrap();
    assert!(tools.contains(&"echo".to_owned()));
    assert!(tools.contains(&"write_file".to_owned()));
    assert!(!marker.exists());
}

#[test]
#[ignore = "需本机 Node.js/npm 与网络；只握手和读取工具列表，不启动浏览器"]
fn npx_chrome_devtools_mcp_lists_tools() {
    let tools = inspect_mcp_server_tools("npx", &[
        "-y".into(), "chrome-devtools-mcp@latest".into(),
    ]).unwrap();
    assert!(tools.contains(&"navigate_page".to_owned()));
    assert!(tools.contains(&"take_snapshot".to_owned()));
}

#[test]
fn cmd_mcp_entry_supports_connection_test_and_workflow_call() {
    let directory = tempfile::tempdir().unwrap();
    let directory = directory.path().join("MCP service with spaces");
    std::fs::create_dir(&directory).unwrap();
    let entry = directory.join("stdio-server.cmd");
    std::fs::write(&entry, "@echo off\r\npowershell.exe %*\r\n").unwrap();
    let marker = directory.join("not written.txt");
    let mut config = test_config(&marker);
    let server = config.servers.get_mut("fixture").unwrap();
    server.command = entry.to_string_lossy().into_owned();
    let tools = inspect_mcp_server_tools(&server.command, &server.args).unwrap();
    assert!(tools.contains(&"echo".to_owned()));
    assert!(!marker.exists());

    let grant = grant_for("{op:'invoke',id:'read',uses:'mcp.call',with:{server:'fixture',tool:'echo'}}");
    let registry = make_registry(config, Some(grant));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let result = registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"server":"fixture","tool":"echo","arguments":{"value":"hello"}}),
        &WorkflowRunControl::new(),
    ).unwrap();
    assert_eq!(result.output, Some(json!({"value":"hello"})));
    assert!(!marker.exists());
}

fn skill_settings(base_url: String) -> crate::ai::config::AiProviderSettings {
    let mut settings = crate::ai::config::AiProviderSettings::default();
    settings.enabled = true;
    settings.base_url = base_url;
    settings.model = "test-model".into();
    settings
}

// 此 fixture 独立于 client 层，验证已授权 Skill 的完整业务入口不会重写配置。
fn skill_http_fixture() -> (String, std::thread::JoinHandle<serde_json::Value>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Instant;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "未收到 Skill HTTP 请求");
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(err) => panic!("Skill fixture 接收失败：{err}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut bytes = Vec::new();
        let header_end = loop {
            let mut buffer = [0; 4096];
            let count = stream.read(&mut buffer).unwrap();
            assert_ne!(count, 0, "Skill 请求头提前结束");
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                break end + 4;
            }
            assert!(bytes.len() < 64 * 1024, "Skill 请求头过大");
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
        assert!(headers.starts_with("POST /chat/completions HTTP/1.1\r\n"));
        let length = headers.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().unwrap())
        }).expect("Skill 请求应发送 Content-Length");
        assert!(length < 1024 * 1024);
        while bytes.len() < header_end + length {
            let mut buffer = [0; 4096];
            let count = stream.read(&mut buffer).unwrap();
            assert_ne!(count, 0, "Skill 请求正文提前结束");
            bytes.extend_from_slice(&buffer[..count]);
        }
        let request = serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
        let response = format!("data: {}\n\ndata: [DONE]\n\n", json!({
            "choices":[{"index":0,"delta":{"content":"已按 Skill 指令完成。"},"finish_reason":"stop"}]
        }));
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
        request
    });
    (base_url, server)
}

#[test]
fn skill_without_tools_preserves_custom_budget_above_4096_and_completes() {
    let data_dir = tempfile::tempdir().unwrap();
    test_skill(data_dir.path());
    let mut grant = grant_for("{op:'invoke',id:'skill',uses:'skill.run',with:{skill:'fixture-skill',task:'只输出完成结果'}}");
    grant.prepare_skills(data_dir.path()).unwrap();
    let (base_url, server) = skill_http_fixture();
    let mut settings = skill_settings(base_url);
    settings.max_tokens = 20_123;
    settings.temperature = 0.625;
    settings.reasoning_effort = Some("high".into());
    settings.request_timeout_secs = 5;
    let mut registry = WorkflowActionRegistry::default();
    register_external_actions_with_ai(
        &mut registry, WorkflowMcpConfig::default(), Some(grant),
        Some(settings), None, Some(data_dir.path()),
    ).unwrap();
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let mut progress = Vec::new();
    let result = registry.get("skill.run").unwrap().execute_with_control_and_progress(
        &service, &mut repo,
        &json!({"skill":"fixture-skill","task":"只输出完成结果"}),
        &WorkflowRunControl::new(), &mut |line| progress.push(line),
    ).unwrap();
    assert_eq!(result.output, Some(json!("已按 Skill 指令完成。")));
    assert!(result.details.iter().any(|line| line.contains("fixture-skill 已完成")));
    assert!(progress.iter().any(|line| line.contains("允许 0 个工具")));
    let request = server.join().unwrap();
    assert_eq!(request["max_tokens"], 20_123);
    assert_eq!(request["temperature"], 0.625);
    assert_eq!(request["reasoning_effort"], "high");
    assert_eq!(request["model"], "test-model");
    assert_eq!(request["stream"], true);
    assert!(request.get("tools").is_none());
    assert!(request["messages"][0]["content"].as_str().unwrap().contains("读取 fixture 工具"));
    assert_eq!(request["messages"][1]["content"], "只输出完成结果");
    assert!(registry.finish().is_empty());
}

#[test]
fn skill_package_content_is_required_before_authorization() {
    let dir = tempfile::tempdir().unwrap();
    test_skill(dir.path());
    assert_eq!(discover_skill_packages(dir.path()).unwrap(),
        vec![("fixture-skill".into(), "测试工具循环".into())]);
    let step = "{op:'invoke',id:'skill',uses:'skill.run',with:{skill:'fixture-skill',task:'读取',tools:[{server:'fixture',tool:'echo'}]}}";
    let mut grant = grant_for(step);
    assert!(grant.permission_lines(&test_config(&dir.path().join("unused"))).is_err());
    grant.prepare_skills(dir.path()).unwrap();
    let lines = grant.permission_lines(&test_config(&dir.path().join("unused"))).unwrap();
    assert!(lines.iter().any(|line| line.contains("AI Skill：fixture-skill")));
}

#[test]
fn mcp_command_only_config_saves_without_starting_or_installing_server() {
    let dir = tempfile::tempdir().unwrap();
    let server: super::extensions::WorkflowMcpServer = serde_json::from_value(json!({
        "command": "khaslana-mcp-not-installed", "args": ["--stdio"]
    })).unwrap();
    assert!(server.auto_discover);
    assert!(server.enabled);
    upsert_user_mcp_server(dir.path(), None, "uninstalled", server).unwrap();
    let config = load_user_mcp_config(dir.path()).unwrap();
    assert!(config.servers["uninstalled"].auto_discover);
    let grant = grant_for("{op:'invoke',id:'read',uses:'mcp.call',with:{server:'uninstalled',tool:'new_tool'}}");
    assert_eq!(grant.permission_lines(&config).unwrap(),
        ["写入 MCP 工具：uninstalled / new_tool"]);
}

#[test]
fn mcp_legacy_whitelist_and_disabled_service_keep_permission_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let server = test_config(&dir.path().join("unused")).servers.remove("fixture").unwrap();
    assert!(!server.auto_discover);
    upsert_user_mcp_server(dir.path(), None, "fixture", server).unwrap();
    let config = load_user_mcp_config(dir.path()).unwrap();
    let unknown = grant_for("{op:'invoke',id:'read',uses:'mcp.call',with:{server:'fixture',tool:'unknown'}}");
    assert!(unknown.permission_lines(&config).unwrap_err().to_string().contains("未在本地配置中允许"));
    super::extensions::set_user_mcp_server_enabled(dir.path(), "fixture", false).unwrap();
    let config = load_user_mcp_config(dir.path()).unwrap();
    let read = grant_for("{op:'invoke',id:'read',uses:'mcp.call',with:{server:'fixture',tool:'echo'}}");
    assert!(read.permission_lines(&config).unwrap_err().to_string().contains("已禁用"));
    assert!(!config.servers["fixture"].auto_discover);
    assert_eq!(config.servers["fixture"].tools["echo"].access,
        super::extensions::WorkflowToolAccess::Read);
    super::extensions::set_user_mcp_server_enabled(dir.path(), "fixture", true).unwrap();
    assert!(read.permission_lines(&load_user_mcp_config(dir.path()).unwrap()).is_ok());
}

#[test]
fn mcp_auto_discovery_still_requires_grant_and_live_schema() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("not-written.txt");
    let mut config = test_config(&marker);
    let server = config.servers.get_mut("fixture").unwrap();
    server.tools.clear();
    server.auto_discover = true;
    // 缓存只是展示记录，不能代替服务实际返回的工具与 schema。
    server.cached_tools = vec!["echo".into(), "nonexistent".into()];
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let control = WorkflowRunControl::new();
    let denied = make_registry(config.clone(), None);
    assert!(denied.get("mcp.call").unwrap().execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"write_file","arguments":{"value":"denied"}}),
        &control).unwrap_err().to_string().contains("尚未授权"));
    assert!(!marker.exists());
    let grant = grant_for(r#"
        {op:'invoke',id:'read',uses:'mcp.call',with:{server:'fixture',tool:'echo'}},
        {op:'invoke',id:'missing',uses:'mcp.call',with:{server:'fixture',tool:'nonexistent'}}
    "#);
    let registry = make_registry(config, Some(grant));
    let action = registry.get("mcp.call").unwrap();
    assert!(action.execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"echo","arguments":{"value":3}}),
        &control).unwrap_err().to_string().contains("schema"));
    assert_eq!(action.execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"echo","arguments":{"value":"discovered"}}),
        &control).unwrap().output, Some(json!({"value":"discovered"})));
    assert!(action.execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"nonexistent"}), &control)
        .unwrap_err().to_string().contains("未提供工具"));
}

#[test]
#[ignore = "需要下载 Playwright MCP 依赖"]
fn browser_runtime_downloads_and_starts_without_npx() {
    let data = tempfile::tempdir().unwrap();
    super::browser_runtime::install(data.path(), &crate::proxy::NetworkProxySettings::default(),
        |message| eprintln!("{message}")).unwrap();
    let (node, mut args) = super::browser_runtime::launch_command(data.path()).unwrap();
    args.push("--help".into());
    let output = std::process::Command::new(node).args(args).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
#[ignore = "需要下载 Node 与 Playwright MCP 依赖"]
fn browser_runtime_downloads_node_when_system_node_is_unavailable() {
    let data = tempfile::tempdir().unwrap();
    super::browser_runtime::install_without_system_node(data.path(),
        &crate::proxy::NetworkProxySettings::default(),
        |message| eprintln!("{message}")).unwrap();
    let (node, mut args) = super::browser_runtime::launch_command(data.path()).unwrap();
    assert!(node.starts_with(data.path()));
    args.push("--help".into());
    let output = std::process::Command::new(node).args(args).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn builtin_edge_mcp_requires_no_user_config_and_legacy_sample_is_migrated_in_memory() {
    let data = tempfile::tempdir().unwrap();
    let config = super::extensions::load_mcp_config(data.path()).unwrap();
    let builtin = config.servers.get(super::browser_runtime::SERVER_ID).unwrap();
    assert_eq!(builtin.command, super::browser_runtime::COMMAND_MARKER);
    assert!(builtin.tools.contains_key("browser_snapshot"));
    assert!(!super::browser_runtime::ready(data.path()));
    let mut config = config;
    config.configure_browser_proxy(&crate::proxy::NetworkProxySettings::default()).unwrap();
    assert!(config.servers[super::browser_runtime::SERVER_ID].args.is_empty());
    let different_proxies: crate::proxy::NetworkProxySettings = serde_json::from_value(json!({
        "mode": "Custom",
        "custom": {
            "http_proxy": "http://127.0.0.1:8001",
            "https_proxy": "http://127.0.0.1:8002"
        }
    })).unwrap();
    assert!(config.configure_browser_proxy(&different_proxies).is_err());

    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/examples/workflow-edge-demo/workflow-mcp.json5");
    std::fs::copy(sample, data.path().join("workflow-mcp.json5")).unwrap();
    let config = super::extensions::load_mcp_config(data.path()).unwrap();
    assert_eq!(config.servers[super::browser_runtime::LEGACY_SERVER_ID].command,
        super::browser_runtime::COMMAND_MARKER);
    assert!(std::fs::read_to_string(data.path().join("workflow-mcp.json5")).unwrap()
        .contains("npx.cmd"));
}

#[test]
fn edge_demo_templates_parse_and_allow_only_read_fill_tools() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/examples/workflow-edge-demo");
    let (_repo_dir, repo, service) = git_support::init_repo();
    for file in ["edge-web-form.json5", "edge-web-form-skill.json5"] {
        let definition = parse_workflow_json5(&std::fs::read_to_string(root.join(file)).unwrap()).unwrap();
        let mut grant = WorkflowExternalGrant::for_definition(&definition).unwrap().unwrap();
        grant.prepare_skills(&root).unwrap();
        let config = super::extensions::load_mcp_config(&root).unwrap();
        let lines = grant.permission_lines(&config).unwrap();
        assert!(lines.iter().any(|line| line.contains("browser_type")));
        assert!(!lines.iter().any(|line| line.contains("browser_click")));
        let mut registry = WorkflowActionRegistry::default();
        register_external_actions_with_ai(&mut registry, config, None,
            Some(skill_settings("http://127.0.0.1".into())), None, Some(&root)).unwrap();
        let mut options = WorkflowRunOptions::default();
        options.input_vars.insert("demoText".into(), "preview".into());
        assert_eq!(WorkflowExecutor::with_actions(&service, &registry)
            .preview(&repo, &definition, &options).unwrap().steps.len(), 1);
    }
}

#[test]
fn edge_demo_stops_before_fill_when_page_snapshot_is_wrong() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/examples/workflow-edge-demo");
    let definition = parse_workflow_json5(&std::fs::read_to_string(root.join("edge-web-form.json5")).unwrap()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("wrong-page.txt");
    let mut config = test_config(&marker);
    let mut server = config.servers.remove("fixture").unwrap();
    for (name, access) in [("browser_navigate", "write"), ("browser_snapshot", "read"), ("browser_type", "write")] {
        server.tools.insert(name.into(), serde_json::from_value(json!({"access":access})).unwrap());
    }
    config.servers.insert("browser.edge".into(), server);
    let grant = WorkflowExternalGrant::for_definition(&definition).unwrap();
    let registry = make_registry(config, grant);
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let mut options = WorkflowRunOptions::default();
    options.input_vars.insert("demoText".into(), "must-not-appear".into());
    let result = WorkflowExecutor::with_actions(&service, &registry).run(&mut repo, &definition,
        options, |_| {});
    assert!(result.unwrap_err().to_string().contains("JavaScript 执行失败"));
    assert!(!marker.exists());
}

#[test]
#[ignore = "需要本机 Edge、网络和可用的 Selenium 演示页"]
fn edge_demo_reads_and_fills_real_public_page() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/examples/workflow-edge-demo");
    let definition = parse_workflow_json5(&std::fs::read_to_string(root.join("edge-web-form.json5")).unwrap()).unwrap();
    let runtime = tempfile::tempdir().unwrap();
    super::browser_runtime::install(runtime.path(), &crate::proxy::NetworkProxySettings::default(), |_| {}).unwrap();
    let mut config = super::extensions::load_mcp_config(runtime.path()).unwrap();
    config.configure_browser_proxy(&crate::proxy::NetworkProxySettings::default()).unwrap();
    let grant = WorkflowExternalGrant::for_definition(&definition).unwrap();
    let mut registry = WorkflowActionRegistry::default();
    register_external_actions_with_ai(&mut registry, config, grant, None, None,
        Some(runtime.path())).unwrap();
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let mut options = WorkflowRunOptions::default();
    options.input_vars.insert("demoText".into(), "Khaslana Edge demo".into());
    let result = WorkflowExecutor::with_actions(&service, &registry).run(&mut repo, &definition, options, |_| {});
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn mcp_read_write_schema_and_permission_are_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("written.txt");
    let config = test_config(&marker);
    let grant = grant_for(r#"
        {op:'invoke', id:'read', uses:'mcp.call', with:{server:'fixture',tool:'echo'}},
        {op:'invoke', id:'write', uses:'mcp.call', with:{server:'fixture',tool:'write_file'}}
    "#);
    let permissions = grant.permission_lines(&config).unwrap();
    assert!(permissions.iter().any(|line| line.contains("读取 MCP 工具：fixture / echo")));
    assert!(permissions.iter().any(|line| line.contains("写入 MCP 工具：fixture / write_file")));
    let registry = make_registry(config.clone(), Some(grant));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let control = WorkflowRunControl::new();
    let read = registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"server":"fixture","tool":"echo","arguments":{"value":"hello"}}),
        &control,
    ).unwrap();
    assert_eq!(read.output, Some(json!({"value":"hello"})));
    let invalid = registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"server":"fixture","tool":"write_file","arguments":{"value":3}}),
        &control,
    );
    assert!(invalid.unwrap_err().to_string().contains("schema"));
    assert!(!marker.exists());
    let write = registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"server":"fixture","tool":"write_file","arguments":{"value":"saved"}}),
        &control,
    ).unwrap();
    assert_eq!(write.output, Some(json!({"written":true})));
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "saved");

    let denied_registry = make_registry(config, None);
    let denied = denied_registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"server":"fixture","tool":"write_file","arguments":{"value":"changed"}}),
        &control,
    );
    assert!(denied.unwrap_err().to_string().contains("尚未授权"));
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "saved");
}

#[test]
fn js_transforms_input_without_any_mcp_server() {
    let script = "return {upper: input.value.toUpperCase()};";
    let grant = grant_for(&format!(r#"{{op:'invoke',id:'js',uses:'js.run',with:{{script:{}}}}}"#,
        serde_json::to_string(script).unwrap()));
    let registry = make_registry(WorkflowMcpConfig::default(), Some(grant));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let result = registry.get("js.run").unwrap().execute_with_control(
        &service, &mut repo, &json!({"script":script,"input":{"value":"hello"}}),
        &WorkflowRunControl::new(),
    ).unwrap();
    assert_eq!(result.output, Some(json!({"upper":"HELLO"})));
}

#[test]
fn js_transforms_data_and_calls_only_declared_tools() {
    let dir = tempfile::tempdir().unwrap();
    let config = test_config(&dir.path().join("untouched.txt"));
    let script = "const reply = mcp.call('fixture', 'echo', {value: input.value}); return {upper: reply.value.toUpperCase()};";
    let definition = parse_workflow_json5(&format!(r#"{{version:2,steps:[{{op:'invoke',id:'js',uses:'js.run',with:{{script:{},tools:[{{server:'fixture',tool:'echo'}}]}}}}]}}"#,
        serde_json::to_string(script).unwrap())).unwrap();
    let grant = WorkflowExternalGrant::for_definition(&definition).unwrap();
    let registry = make_registry(config, grant);
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let result = registry.get("js.run").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"script":script,"tools":[{"server":"fixture","tool":"echo"}],
            "input":{"value":"hello"}}),
        &WorkflowRunControl::new(),
    ).unwrap();
    assert_eq!(result.output, Some(json!({"upper":"HELLO"})));

    let denied_script = "return mcp.call('fixture', 'write_file', {value:'bad'});";
    let denied_grant = grant_for(&format!(r#"{{op:'invoke',id:'js',uses:'js.run',with:{{script:{},tools:[{{server:'fixture',tool:'echo'}}]}}}}"#,
        serde_json::to_string(denied_script).unwrap()));
    let denied_registry = make_registry(test_config(&dir.path().join("untouched.txt")), Some(denied_grant));
    let denied = denied_registry.get("js.run").unwrap().execute_with_control(
        &service, &mut repo,
        &json!({"script":denied_script,"tools":[{"server":"fixture","tool":"echo"}]}),
        &WorkflowRunControl::new(),
    );
    assert!(denied.is_err());
    assert!(!dir.path().join("untouched.txt").exists());
}

#[test]
fn js_infinite_loop_can_be_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let script = "for (;;) {}";
    let grant = grant_for(&format!(r#"{{op:'invoke',id:'js',uses:'js.run',with:{{script:{}}}}}"#,
        serde_json::to_string(script).unwrap()));
    let registry = make_registry(test_config(&dir.path().join("unused.txt")), Some(grant));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let control = WorkflowRunControl::new();
    let cancel = control.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        cancel.cancel();
    });
    let result = registry.get("js.run").unwrap().execute_with_control(
        &service, &mut repo, &json!({"script":script}), &control);
    thread.join().unwrap();
    assert!(matches!(result, Err(GitError::WorkflowCancelled)));
}

#[test]
fn v2_workflow_passes_mcp_output_through_js_to_write_tool() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("workflow-result.txt");
    let definition: WorkflowDefinition = serde_json::from_value(json!({
        "version": 2,
        "defaults": {"requireCleanWorktree": true},
        "steps": [
            {"op":"invoke","id":"read","uses":"mcp.call","saveAs":"readResult",
                "with":{"server":"fixture","tool":"echo","arguments":{"value":"hello"}}},
            {"op":"invoke","id":"transform","uses":"js.run","saveAs":"prepared",
                "with":{"script":"return {value: input.value.toUpperCase()};",
                    "input":{"value":"${readResult.value}"}}},
            {"op":"invoke","id":"write","uses":"mcp.call",
                "with":{"server":"fixture","tool":"write_file",
                    "arguments":{"value":"${prepared.value}"}}}
        ]
    })).unwrap();
    let grant = WorkflowExternalGrant::for_definition(&definition).unwrap();
    let registry = make_registry(test_config(&marker), grant);
    let (repo_dir, mut repo, service) = git_support::init_repo();
    git_support::write_file(repo_dir.path(), "README.md", "initial\n");
    git_support::commit_all(&repo, "initial");
    let executor = WorkflowExecutor::with_actions(&service, &registry);
    let preview = executor.preview(&repo, &definition, &WorkflowRunOptions::default()).unwrap();
    assert_eq!(preview.steps.len(), 3);
    let result = executor.run(&mut repo, &definition, WorkflowRunOptions::default(), |_| {}).unwrap();
    assert_eq!(result.steps_run, 3);
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "HELLO");
}

#[test]
fn mcp_call_can_be_cancelled_while_server_is_waiting() {
    let dir = tempfile::tempdir().unwrap();
    let registry = make_registry(test_config(&dir.path().join("unused.txt")),
        Some(grant_for("{op:'invoke',id:'stall',uses:'mcp.call',with:{server:'fixture',tool:'stall'}}")));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let control = WorkflowRunControl::new();
    let cancel = control.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        cancel.cancel();
    });
    let started = std::time::Instant::now();
    let result = registry.get("mcp.call").unwrap().execute_with_control(
        &service, &mut repo, &json!({"server":"fixture","tool":"stall"}), &control);
    thread.join().unwrap();
    assert!(matches!(result, Err(GitError::WorkflowCancelled)));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn external_wait_stops_at_deadline() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let control = WorkflowRunControl::new();
    let result: Result<()> = runtime.block_on(super::extensions::wait_checked(
        std::future::pending::<std::result::Result<(), ()>>(),
        &control, std::time::Instant::now() + Duration::from_millis(100), "MCP 工具调用",
    ));
    assert!(result.unwrap_err().to_string().contains("超时"));
}

#[test]
fn external_permission_rejects_dynamic_tool_identity() {
    let definition = parse_workflow_json5(r#"{
        version: 2,
        steps: [{op:'invoke',id:'call',uses:'mcp.call',
            with:{server:'fixture',tool:'${selectedTool}',arguments:{}}}]
    }"#).unwrap();
    assert!(WorkflowExternalGrant::for_definition(&definition).unwrap_err()
        .to_string().contains("固定值"));
}

#[test]
fn mcp_session_persists_between_steps_of_one_run() {
    let dir = tempfile::tempdir().unwrap();
    let grant = grant_for(r#"
        {op:'invoke',id:'set',uses:'mcp.call',with:{server:'fixture',tool:'set_state'}},
        {op:'invoke',id:'get',uses:'mcp.call',with:{server:'fixture',tool:'get_state'}}
    "#);
    let mut config = test_config(&dir.path().join("unused.txt"));
    let discoveries = dir.path().join("discoveries.txt");
    config.servers.get_mut("fixture").unwrap().args.push(discoveries.to_string_lossy().into_owned());
    let registry = make_registry(config, Some(grant));
    let (_repo_dir, mut repo, service) = git_support::init_repo();
    let control = WorkflowRunControl::new();
    registry.get("mcp.call").unwrap().execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"set_state","arguments":{"value":"same session"}}),
        &control).unwrap();
    let result = registry.get("mcp.call").unwrap().execute_with_control(&service, &mut repo,
        &json!({"server":"fixture","tool":"get_state"}), &control).unwrap();
    assert_eq!(result.output, Some(json!({"value":"same session"})));
    assert_eq!(std::fs::read_to_string(discoveries).unwrap().lines().count(), 1);
}
