//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const THE_TEST_WILL_CONNECT_TO_THE_CURRENT_REMOTE_REMOTE: UiMessage = UiMessage::new("views.proxy_view.the_test_will_connect_to_the_current_remote_remote", "测试将连接当前远端：{remote}", "The test will connect to the current remote: {remote}");
pub const OPEN_A_REPOSITORY_WITH_A_REMOTE_BEFORE_TESTING_THE_PROXY: UiMessage = UiMessage::new("views.proxy_view.open_a_repository_with_a_remote_before_testing_the_proxy", "测试代理需要先打开带远端的仓库", "Open a repository with a remote before testing the proxy");
pub const PROXY_MODE: UiMessage = UiMessage::new("views.proxy_view.proxy_mode", "代理模式", "Proxy mode");
pub const MODE: UiMessage = UiMessage::new("views.proxy_view.mode", "模式", "Mode");
pub const NO_PROXY: UiMessage = UiMessage::new("views.proxy_view.no_proxy", "不使用代理", "No proxy");
pub const USE_SYSTEM_PROXY: UiMessage = UiMessage::new("views.proxy_view.use_system_proxy", "使用系统代理", "Use system proxy");
pub const CUSTOM_PROXY: UiMessage = UiMessage::new("views.proxy_view.custom_proxy", "自定义代理", "Custom proxy");
pub const CUSTOM_ADDRESSES: UiMessage = UiMessage::new("views.proxy_view.custom_addresses", "自定义地址", "Custom addresses");
pub const FOR_NOW_INCLUDE_PROXY_AUTHENTICATION_IN_THE_URL_FOR_EXAMPLE: UiMessage = UiMessage::new("views.proxy_view.for_now_include_proxy_authentication_in_the_url_for_example", "代理认证第一版请写在 URL 中，例如 http://user:pass@127.0.0.1:7890。", "For now, include proxy authentication in the URL, for example http://user:pass@127.0.0.1:7890.");
pub const HTTP_PROXY: UiMessage = UiMessage::new("views.proxy_view.http_proxy", "HTTP 代理", "HTTP proxy");
pub const HTTPS_PROXY: UiMessage = UiMessage::new("views.proxy_view.https_proxy", "HTTPS 代理", "HTTPS proxy");
pub const SOCKS5_PROXY: UiMessage = UiMessage::new("views.proxy_view.socks5_proxy", "SOCKS5 代理", "SOCKS5 proxy");
pub const CONNECTION_TEST: UiMessage = UiMessage::new("views.proxy_view.connection_test", "连接测试", "Connection test");
pub const TEST_THE_PROXY_CONNECTION_BEFORE_SAVING: UiMessage = UiMessage::new("views.proxy_view.test_the_proxy_connection_before_saving", "在保存前可以先测试一次代理连通性。", "Test the proxy connection before saving.");
pub const CURRENT_REMOTE: UiMessage = UiMessage::new("views.proxy_view.current_remote", "当前远端", "Current remote");
pub const TEST_PROXY: UiMessage = UiMessage::new("views.proxy_view.test_proxy", "测试代理", "Test proxy");
pub const SAVE: UiMessage = UiMessage::new("views.proxy_view.save", "保存", "Save");
pub const PROXY_SETTINGS_SAVED: UiMessage = UiMessage::new("views.proxy_view.proxy_settings_saved", "代理设置已保存", "Proxy settings saved");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.proxy_view.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");
pub const GIT_NETWORK_OPERATIONS_CONNECT_DIRECTLY_IGNORING_GIT_CONFIGURATION_AND_ENVIRONMENT: UiMessage = UiMessage::new("views.proxy_view.git_network_operations_connect_directly_ignoring_git_configuration_and_environment", "Git 网络操作将显式直连，不使用 Git 配置或环境变量代理。", "Git network operations connect directly, ignoring Git configuration and environment proxies.");
pub const USE_LIBGIT2_AUTOMATIC_PROXY_SELECTION_GIT_PROXY_CONFIGURATION_FIRST_THEN: UiMessage = UiMessage::new("views.proxy_view.use_libgit2_automatic_proxy_selection_git_proxy_configuration_first_then", "使用 libgit2 自动代理：优先读取 Git 代理配置，其次读取 http_proxy / https_proxy 环境变量；不读取系统 UI 代理或 PAC。", "Use libgit2 automatic proxy selection: Git proxy configuration first, then http_proxy / https_proxy environment variables. System UI proxy settings and PAC are not read.");
pub const SELECT_A_CUSTOM_PROXY_BY_REMOTE_PROTOCOL_HTTP_HTTPS_REMOTES: UiMessage = UiMessage::new("views.proxy_view.select_a_custom_proxy_by_remote_protocol_http_https_remotes", "按远端协议选择自定义代理；HTTP/HTTPS 远端可回退 SOCKS5，SSH 远端仅尝试自定义 SOCKS5。", "Select a custom proxy by remote protocol. HTTP/HTTPS remotes may fall back to SOCKS5; SSH remotes only try the custom SOCKS5 proxy.");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    THE_TEST_WILL_CONNECT_TO_THE_CURRENT_REMOTE_REMOTE,
    OPEN_A_REPOSITORY_WITH_A_REMOTE_BEFORE_TESTING_THE_PROXY,
    PROXY_MODE,
    MODE,
    NO_PROXY,
    USE_SYSTEM_PROXY,
    CUSTOM_PROXY,
    CUSTOM_ADDRESSES,
    FOR_NOW_INCLUDE_PROXY_AUTHENTICATION_IN_THE_URL_FOR_EXAMPLE,
    HTTP_PROXY,
    HTTPS_PROXY,
    SOCKS5_PROXY,
    CONNECTION_TEST,
    TEST_THE_PROXY_CONNECTION_BEFORE_SAVING,
    CURRENT_REMOTE,
    TEST_PROXY,
    SAVE,
    PROXY_SETTINGS_SAVED,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    GIT_NETWORK_OPERATIONS_CONNECT_DIRECTLY_IGNORING_GIT_CONFIGURATION_AND_ENVIRONMENT,
    USE_LIBGIT2_AUTOMATIC_PROXY_SELECTION_GIT_PROXY_CONFIGURATION_FIRST_THEN,
    SELECT_A_CUSTOM_PROXY_BY_REMOTE_PROTOCOL_HTTP_HTTPS_REMOTES,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
