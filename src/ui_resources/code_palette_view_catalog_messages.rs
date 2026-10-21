//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const REPO_NAME_BROWSE_TAGGED_FILES: UiMessage = UiMessage::new("views.code_palette_view.catalog.repo_name_browse_tagged_files", "{repo_name} · 浏览标签文件", "{repo_name} · Browse tagged files");
pub const REPO_NAME_VIEW_STASH_DIFFERENCES: UiMessage = UiMessage::new("views.code_palette_view.catalog.repo_name_view_stash_differences", "{repo_name} · 查看贮藏差异", "{repo_name} · View stash differences");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_a_repository_first", "请先打开仓库", "Open a repository first");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");
pub const THIS_REPOSITORY_HAS_NO_AVAILABLE_REMOTE: UiMessage = UiMessage::new("views.code_palette_view.catalog.this_repository_has_no_available_remote", "当前仓库没有可用远端", "This repository has no available remote");
pub const COMPLETE_OR_ABORT_THE_MERGE_FIRST: UiMessage = UiMessage::new("views.code_palette_view.catalog.complete_or_abort_the_merge_first", "请先完成或中止合并", "Complete or abort the merge first");
pub const OPEN_REPOSITORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_repository", "打开仓库", "Open repository");
pub const CHOOSE_A_LOCAL_GIT_REPOSITORY_FOLDER: UiMessage = UiMessage::new("views.code_palette_view.catalog.choose_a_local_git_repository_folder", "选择本地 Git 仓库文件夹", "Choose a local Git repository folder");
pub const COMMANDS: UiMessage = UiMessage::new("views.code_palette_view.catalog.commands", "功能", "Commands");
pub const OPEN_PROJECT_OPEN_REPOSITORY_REPO: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_project_open_repository_repo", "打开 项目 open repository repo", "Open project open repository repo");
pub const CLONE_REPOSITORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.clone_repository", "克隆仓库", "Clone repository");
pub const ENTER_A_REMOTE_ADDRESS_AND_LOCAL_DIRECTORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.enter_a_remote_address_and_local_directory", "填写远端地址和本地目录", "Enter a remote address and local directory");
pub const DOWNLOAD_PROJECT_CLONE: UiMessage = UiMessage::new("views.code_palette_view.catalog.download_project_clone", "下载 项目 clone", "Download project clone");
pub const WORKING_TREE: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree", "工作区", "Working tree");
pub const VIEW_CHANGES_STAGE_FILES_AND_COMMIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_changes_stage_files_and_commit", "查看修改、暂存文件和提交", "View changes, stage files, and commit");
pub const CHANGES_COMMIT_STAGE_UNSTAGE_AMEND_COMMIT_STAGE_UNSTAGE_DIFF_REFRESH: UiMessage = UiMessage::new("views.code_palette_view.catalog.changes_commit_stage_unstage_amend_commit_stage_unstage_diff_refresh", "修改 提交 暂存 取消暂存 修补 amend commit stage unstage diff refresh 刷新", "Changes commit stage unstage amend commit stage unstage diff refresh");
pub const HISTORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.history", "提交记录", "History");
pub const VIEW_HISTORY_DIFFERENCES_AND_COMMIT_OPERATIONS: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_history_differences_and_commit_operations", "查看历史、差异及提交操作", "View history, differences, and commit operations");
pub const HISTORY_LOG_UNDO_RESET_REVERT_CHERRY_PICK_RESET_REVERT_LOG: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_log_undo_reset_revert_cherry_pick_reset_revert_log", "历史 日志 撤销 重置 回退 拣选 cherry-pick reset revert log history", "History log undo reset revert cherry-pick reset revert log history");
pub const COMMIT_GRAPH: UiMessage = UiMessage::new("views.code_palette_view.catalog.commit_graph", "提交图谱", "Commit graph");
pub const VIEW_BRANCH_LINEAGE_AND_COMMIT_RELATIONSHIPS: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_branch_lineage_and_commit_relationships", "查看分支谱系和提交关系", "View branch lineage and commit relationships");
pub const GRAPH_BRANCHES_GRAPH: UiMessage = UiMessage::new("views.code_palette_view.catalog.graph_branches_graph", "图谱 分支 graph", "Graph branches graph");
pub const WORKFLOWS: UiMessage = UiMessage::new("views.code_palette_view.catalog.workflows", "工作流", "Workflows");
pub const CHOOSE_TEMPLATES_CONFIGURE_PARAMETERS_AND_VIEW_EXECUTION_LOGS: UiMessage = UiMessage::new("views.code_palette_view.catalog.choose_templates_configure_parameters_and_view_execution_logs", "选择模板、配置参数和查看执行日志", "Choose templates, configure parameters, and view execution logs");
pub const AUTOMATION_TEMPLATES_WORKFLOW: UiMessage = UiMessage::new("views.code_palette_view.catalog.automation_templates_workflow", "自动化 模板 workflow", "Automation templates workflow");
pub const CONFLICTS: UiMessage = UiMessage::new("views.code_palette_view.catalog.conflicts", "冲突处理", "Conflicts");
pub const VIEW_CONFLICTS_AND_ACCEPT_BLOCKS_OR_MERGE_WITH_AI: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_conflicts_and_accept_blocks_or_merge_with_ai", "查看冲突并按块接受或使用 AI 合并", "View conflicts and accept blocks or merge with AI");
pub const MERGE_RESOLVE_MERGE_CONFLICT_AI: UiMessage = UiMessage::new("views.code_palette_view.catalog.merge_resolve_merge_conflict_ai", "合并 解决冲突 merge conflict AI", "Merge resolve merge conflict AI");
pub const STASH_LIST: UiMessage = UiMessage::new("views.code_palette_view.catalog.stash_list", "贮藏列表", "Stash list");
pub const VIEW_SAVED_STASHES_AND_FILE_DIFFERENCES: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_saved_stashes_and_file_differences", "查看已保存的贮藏和文件差异", "View saved stashes and file differences");
pub const STASH_STASH: UiMessage = UiMessage::new("views.code_palette_view.catalog.stash_stash", "储藏 stash", "Stash stash");
pub const BROWSE_FILES: UiMessage = UiMessage::new("views.code_palette_view.catalog.browse_files", "浏览文件", "Browse files");
pub const BROWSE_CURRENT_BRANCH_FILES_AND_COMPARE_DIFFERENCES: UiMessage = UiMessage::new("views.code_palette_view.catalog.browse_current_branch_files_and_compare_differences", "浏览当前分支文件内容和比较差异", "Browse current branch files and compare differences");
pub const FILES_VIEW_BRANCH_COMPARISON_COMPARE_BROWSE: UiMessage = UiMessage::new("views.code_palette_view.catalog.files_view_branch_comparison_compare_browse", "文件 查看 分支比较 compare browse", "Files view branch comparison compare browse");
pub const ATTRIBUTION: UiMessage = UiMessage::new("views.code_palette_view.catalog.attribution", "追溯", "Attribution");
pub const VIEW_THE_ORIGINATING_COMMIT_FOR_EACH_FILE_LINE: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_the_originating_commit_for_each_file_line", "查看文件每一行的提交来源", "View the originating commit for each file line");
pub const BLAME_AUTHOR_ATTRIBUTION: UiMessage = UiMessage::new("views.code_palette_view.catalog.blame_author_attribution", "blame 作者 代码追溯", "blame author attribution");
pub const THIS_REPOSITORY_HAS_NO_CONFLICTS: UiMessage = UiMessage::new("views.code_palette_view.catalog.this_repository_has_no_conflicts", "当前仓库没有冲突", "This repository has no conflicts");
pub const COMMIT_CHANGES: UiMessage = UiMessage::new("views.code_palette_view.catalog.commit_changes", "提交修改", "Commit changes");
pub const WORKING_TREE_STAGE_CHANGES_AND_ENTER_A_COMMIT_MESSAGE_BELOW: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_stage_changes_and_enter_a_commit_message_below", "工作区 · 暂存修改后在差异下方填写提交信息", "Working tree · Stage changes and enter a commit message below the diff");
pub const COMMIT_COMMIT_STAGE_STAGE: UiMessage = UiMessage::new("views.code_palette_view.catalog.commit_commit_stage_stage", "commit 提交 暂存 stage", "commit commit stage stage");
pub const AI_COMMIT_MESSAGE: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_commit_message", "AI 生成提交信息", "AI commit message");
pub const WORKING_TREE_AI_GENERATION_BUTTON_IN_THE_COMMIT_AREA: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_ai_generation_button_in_the_commit_area", "工作区 · 提交区的 AI 生成按钮", "Working tree · AI generation button in the commit area");
pub const COMMIT_MESSAGE_COMMIT_MESSAGE: UiMessage = UiMessage::new("views.code_palette_view.catalog.commit_message_commit_message", "提交说明 commit message", "Commit message commit message");
pub const AI_CODE_REVIEW: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_code_review", "AI 代码评审", "AI code review");
pub const WORKING_TREE_SELECT_A_DIFF_AND_OPEN_AI_REVIEW: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_select_a_diff_and_open_ai_review", "工作区 · 选择差异后打开 AI 评审", "Working tree · Select a diff and open AI review");
pub const REVIEW_REVIEW: UiMessage = UiMessage::new("views.code_palette_view.catalog.review_review", "审查 review", "Review review");
pub const REFRESH_REPOSITORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.refresh_repository", "刷新仓库", "Refresh repository");
pub const TITLE_BAR_REFRESH_BUTTON: UiMessage = UiMessage::new("views.code_palette_view.catalog.title_bar_refresh_button", "顶栏 · 刷新按钮", "Title bar · Refresh button");
pub const REFRESH_REFRESH: UiMessage = UiMessage::new("views.code_palette_view.catalog.refresh_refresh", "refresh 刷新", "refresh refresh");
pub const FETCH_REMOTE_UPDATES: UiMessage = UiMessage::new("views.code_palette_view.catalog.fetch_remote_updates", "获取远端更新", "Fetch remote updates");
pub const TITLE_BAR_FETCH_BUTTON: UiMessage = UiMessage::new("views.code_palette_view.catalog.title_bar_fetch_button", "顶栏 · 获取按钮", "Title bar · Fetch button");
pub const FETCH_FETCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.fetch_fetch", "fetch 获取", "fetch fetch");
pub const SWITCH_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.switch_branch", "切换分支", "Switch branch");
pub const WORKING_TREE_SELECT_A_CHECKOUT_TARGET_IN_THE_BRANCH_LIST: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_select_a_checkout_target_in_the_branch_list", "工作区 · 在分支列表中选择检出目标", "Working tree · Select a checkout target in the branch list");
pub const CHECKOUT_CHECKOUT: UiMessage = UiMessage::new("views.code_palette_view.catalog.checkout_checkout", "checkout 检出", "checkout checkout");
pub const MERGE_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.merge_branch", "合并分支", "Merge branch");
pub const WORKING_TREE_MERGE_ACTION_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_merge_action_in_the_branch_context_menu", "工作区 · 分支右键菜单中的合并操作", "Working tree · Merge action in the branch context menu");
pub const REBASE: UiMessage = UiMessage::new("views.code_palette_view.catalog.rebase", "变基", "Rebase");
pub const WORKING_TREE_REBASE_ACTION_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_rebase_action_in_the_branch_context_menu", "工作区 · 分支右键菜单中的变基操作", "Working tree · Rebase action in the branch context menu");
pub const RENAME_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.rename_branch", "重命名分支", "Rename branch");
pub const WORKING_TREE_RENAME_ACTION_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_rename_action_in_the_branch_context_menu", "工作区 · 分支右键菜单中的重命名操作", "Working tree · Rename action in the branch context menu");
pub const DELETE_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.delete_branch", "删除分支", "Delete branch");
pub const WORKING_TREE_DELETE_CONFIRMATION_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_delete_confirmation_in_the_branch_context_menu", "工作区 · 分支右键菜单中的删除确认入口", "Working tree · Delete confirmation in the branch context menu");
pub const COMPARE_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.catalog.compare_branches", "分支比较", "Compare branches");
pub const WORKING_TREE_COMPARE_ACTION_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_compare_action_in_the_branch_context_menu", "工作区 · 分支右键菜单中的比较操作", "Working tree · Compare action in the branch context menu");
pub const SET_UPSTREAM_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.set_upstream_branch", "设置上游分支", "Set upstream branch");
pub const WORKING_TREE_UPSTREAM_SETTING_IN_THE_BRANCH_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_upstream_setting_in_the_branch_context_menu", "工作区 · 分支右键菜单中的上游设置", "Working tree · Upstream setting in the branch context menu");
pub const UPSTREAM_TRACKING: UiMessage = UiMessage::new("views.code_palette_view.catalog.upstream_tracking", "upstream 跟踪", "upstream tracking");
pub const CHERRY_PICK_COMMIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.cherry_pick_commit", "拣选提交", "Cherry-pick commit");
pub const HISTORY_SELECT_A_COMMIT_AND_CHERRY_PICK_FROM_ITS_CONTEXT: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_select_a_commit_and_cherry_pick_from_its_context", "提交记录 · 选择提交后在右键菜单中拣选", "History · Select a commit and cherry-pick from its context menu");
pub const RESET_TO_COMMIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.reset_to_commit", "重置到提交", "Reset to commit");
pub const HISTORY_SELECT_A_COMMIT_AND_RESET_FROM_ITS_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_select_a_commit_and_reset_from_its_context_menu", "提交记录 · 选择提交后在右键菜单中重置", "History · Select a commit and reset from its context menu");
pub const REVERT_COMMIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.revert_commit", "还原提交", "Revert commit");
pub const HISTORY_SELECT_A_COMMIT_AND_REVERT_FROM_ITS_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_select_a_commit_and_revert_from_its_context_menu", "提交记录 · 选择提交后在右键菜单中还原", "History · Select a commit and revert from its context menu");
pub const REVERT_REVERT: UiMessage = UiMessage::new("views.code_palette_view.catalog.revert_revert", "revert 回退", "revert revert");
pub const UNDO_COMMIT: UiMessage = UiMessage::new("views.code_palette_view.catalog.undo_commit", "撤销提交", "Undo commit");
pub const HISTORY_SELECT_A_COMMIT_AND_UNDO_FROM_ITS_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_select_a_commit_and_undo_from_its_context_menu", "提交记录 · 选择提交后在右键菜单中撤销", "History · Select a commit and undo from its context menu");
pub const FILE_HISTORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.file_history", "文件历史", "File history");
pub const WORKING_TREE_HISTORY_ACTION_IN_THE_FILE_CONTEXT_MENU: UiMessage = UiMessage::new("views.code_palette_view.catalog.working_tree_history_action_in_the_file_context_menu", "工作区 · 文件右键菜单中的历史入口", "Working tree · History action in the file context menu");
pub const HISTORY_HISTORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.history_history", "历史 history", "History history");
pub const SETTINGS_CENTER: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_center", "设置中心", "Settings center");
pub const OPEN_APPLICATION_SETTINGS_AND_PREFERENCES: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_application_settings_and_preferences", "打开应用设置和偏好", "Open application settings and preferences");
pub const SETTINGS: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings", "设置", "Settings");
pub const SETTINGS_KEYS_TOKENS_ACCOUNTS_SSH_PASSWORDS_CREDENTIALS_TOKEN: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_keys_tokens_accounts_ssh_passwords_credentials_token", "设置 密钥 令牌 账号 SSH 密码 credentials token", "Settings keys tokens accounts SSH passwords credentials token");
pub const SETTINGS_PROXY_SOCKS_HTTP_HTTPS_PROXY: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_proxy_socks_http_https_proxy", "设置 代理 socks http https proxy", "Settings proxy socks http https proxy");
pub const SETTINGS_MODELS_PROVIDERS_APIS_SERVICES_PROVIDER_API_AI: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_models_providers_apis_services_provider_api_ai", "设置 模型 大模型 接口 服务 商 provider API AI", "Settings models providers APIs services provider API AI");
pub const SETTINGS_EXTERNAL_MERGE_INTELLIJ_IDEA_MERGE_TOOL: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_external_merge_intellij_idea_merge_tool", "设置 外部合并 IntelliJ IDEA merge tool", "Settings external merge IntelliJ IDEA merge tool");
pub const SETTINGS_CODE_INDEX_KNOWLEDGE_GRAPH_REBUILD_CODE_INDEX: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_code_index_knowledge_graph_rebuild_code_index", "设置 代码索引 知识图谱 重建 code index", "Settings code index knowledge graph rebuild code index");
pub const SETTINGS_THEME_DARK_LIGHT_COLORS_FONTS_THEME_APPEARANCE: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_theme_dark_light_colors_fonts_theme_appearance", "设置 主题 深色 浅色 配色 字体 theme appearance", "Settings theme dark light colors fonts theme appearance");
pub const SETTINGS_VERSION_UPDATE_CHECKS_DATA_DIRECTORY_MIGRATION_UPDATE: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_version_update_checks_data_directory_migration_update", "设置 版本 检查更新 数据目录 迁移 update", "Settings version update checks data directory migration update");
pub const SETTINGS_KEYS_KEYBOARD_SHORTCUTS_KEYBOARD_SHORTCUT: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_keys_keyboard_shortcuts_keyboard_shortcut", "设置 按键 键盘 快捷键 keyboard shortcut", "Settings keys keyboard shortcuts keyboard shortcut");
pub const SETTINGS_ABOUT_VERSION_INFORMATION_ABOUT: UiMessage = UiMessage::new("views.code_palette_view.catalog.settings_about_version_information_about", "设置 关于 版本 说明 about", "Settings about version information about");
pub const MANAGE_SKILLS: UiMessage = UiMessage::new("views.code_palette_view.catalog.manage_skills", "Skill 管理", "Manage Skills");
pub const AI_SETTINGS_IMPORT_VIEW_AND_MANAGE_SKILLS: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_settings_import_view_and_manage_skills", "AI 设置 · 导入、查看和管理技能", "AI settings · Import, view, and manage Skills");
pub const SKILLS_EXTENSIONS_SKILLS: UiMessage = UiMessage::new("views.code_palette_view.catalog.skills_extensions_skills", "技能 扩展 skills", "Skills extensions skills");
pub const MCP_SERVICES: UiMessage = UiMessage::new("views.code_palette_view.catalog.mcp_services", "MCP 服务", "MCP services");
pub const AI_SETTINGS_CONFIGURE_SERVERS_AND_TOOL_PERMISSIONS: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_settings_configure_servers_and_tool_permissions", "AI 设置 · 配置服务器和工具权限", "AI settings · Configure servers and tool permissions");
pub const TOOLS_EXTENSIONS_MCP_SERVER: UiMessage = UiMessage::new("views.code_palette_view.catalog.tools_extensions_mcp_server", "工具 扩展 mcp server", "Tools extensions mcp server");
pub const BROWSER_RUNTIME: UiMessage = UiMessage::new("views.code_palette_view.catalog.browser_runtime", "浏览器运行环境", "Browser runtime");
pub const AI_SETTINGS_CONFIGURE_THE_WORKFLOW_BROWSER_RUNTIME: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_settings_configure_the_workflow_browser_runtime", "AI 设置 · 配置工作流浏览器运行环境", "AI settings · Configure the workflow browser runtime");
pub const ENABLE_AI_IN_AI_SETTINGS_FIRST: UiMessage = UiMessage::new("views.code_palette_view.catalog.enable_ai_in_ai_settings_first", "请先在 AI 设置中启用 AI 功能", "Enable AI in AI settings first");
pub const CREATE_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_branch", "创建分支", "Create branch");
pub const OPEN_THE_NEW_BRANCH_FORM: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_the_new_branch_form", "打开新分支表单", "Open the new branch form");
pub const CREATE_BRANCH_2: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_branch_2", "新建 branch", "Create branch");
pub const CREATE_TAG: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_tag", "创建标签", "Create tag");
pub const CREATE_A_TAG_FOR_THE_CURRENT_HEAD: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_a_tag_for_the_current_head", "为当前 HEAD 创建标签", "Create a tag for the current HEAD");
pub const CREATE_TAG_2: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_tag_2", "新建 tag", "Create tag");
pub const CREATE_STASH: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_stash", "创建贮藏", "Create stash");
pub const OPEN_THE_STASH_FORM_TO_SAVE_CURRENT_CHANGES: UiMessage = UiMessage::new("views.code_palette_view.catalog.open_the_stash_form_to_save_current_changes", "打开贮藏表单，保存当前修改", "Open the stash form to save current changes");
pub const SUBMODULES: UiMessage = UiMessage::new("views.code_palette_view.catalog.submodules", "子模块", "Submodules");
pub const MANAGE_INITIALIZE_AND_UPDATE_SUBMODULES: UiMessage = UiMessage::new("views.code_palette_view.catalog.manage_initialize_and_update_submodules", "管理、初始化和更新子模块", "Manage, initialize, and update submodules");
pub const MANAGE_REMOTES: UiMessage = UiMessage::new("views.code_palette_view.catalog.manage_remotes", "远端管理", "Manage remotes");
pub const ADD_AND_EDIT_REMOTES_AND_CONFIGURE_CREDENTIALS: UiMessage = UiMessage::new("views.code_palette_view.catalog.add_and_edit_remotes_and_configure_credentials", "添加、编辑远端并配置凭据", "Add and edit remotes and configure credentials");
pub const REMOTE_REMOTE_ADDRESS_FETCH_FETCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.remote_remote_address_fetch_fetch", "remote 远程 地址 获取 fetch", "remote remote address fetch fetch");
pub const PULL: UiMessage = UiMessage::new("views.code_palette_view.catalog.pull", "拉取", "Pull");
pub const SELECT_A_REMOTE_AND_BRANCH_TO_PULL: UiMessage = UiMessage::new("views.code_palette_view.catalog.select_a_remote_and_branch_to_pull", "选择远端和分支后拉取", "Select a remote and branch to pull");
pub const PULL_DOWNLOAD: UiMessage = UiMessage::new("views.code_palette_view.catalog.pull_download", "pull 下载", "pull download");
pub const PUSH: UiMessage = UiMessage::new("views.code_palette_view.catalog.push", "推送", "Push");
pub const SELECT_A_REMOTE_AND_BRANCH_TO_PUSH: UiMessage = UiMessage::new("views.code_palette_view.catalog.select_a_remote_and_branch_to_push", "选择远端和分支后推送", "Select a remote and branch to push");
pub const PUSH_UPLOAD: UiMessage = UiMessage::new("views.code_palette_view.catalog.push_upload", "push 上传", "push upload");
pub const WORKFLOW_EDITOR: UiMessage = UiMessage::new("views.code_palette_view.catalog.workflow_editor", "工作流编辑器", "Workflow editor");
pub const CREATE_OR_EDIT_A_WORKFLOW_TEMPLATE: UiMessage = UiMessage::new("views.code_palette_view.catalog.create_or_edit_a_workflow_template", "创建或编辑工作流模板", "Create or edit a workflow template");
pub const TEMPLATE_CREATE_MODIFY_EDIT_WORKFLOW_EDITOR: UiMessage = UiMessage::new("views.code_palette_view.catalog.template_create_modify_edit_workflow_editor", "模板 新建 修改 编辑 workflow editor", "Template create modify edit workflow editor");
pub const AI_REVIEW_HISTORY: UiMessage = UiMessage::new("views.code_palette_view.catalog.ai_review_history", "AI 评审历史", "AI review history");
pub const VIEW_SAVED_CODE_REVIEW_REPORTS: UiMessage = UiMessage::new("views.code_palette_view.catalog.view_saved_code_review_reports", "查看已保存的代码评审报告", "View saved code review reports");
pub const LOCAL_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.catalog.local_branches", "本地分支", "Local branches");
pub const REMOTE_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.catalog.remote_branches", "远端分支", "Remote branches");
pub const BRANCH_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.catalog.branch_branch", "branch 分支", "branch branch");
pub const TAGS: UiMessage = UiMessage::new("views.code_palette_view.catalog.tags", "标签", "Tags");
pub const REMOTES: UiMessage = UiMessage::new("views.code_palette_view.catalog.remotes", "远端", "Remotes");
pub const STASHES: UiMessage = UiMessage::new("views.code_palette_view.catalog.stashes", "贮藏", "Stashes");
pub const STASH_STASH_2: UiMessage = UiMessage::new("views.code_palette_view.catalog.stash_stash_2", "stash 储藏", "stash stash");

pub const HIGHLIGHT_CURRENT_BRANCH: UiMessage = UiMessage::new("views.code_palette_view_catalog.highlight_current_branch", "{repo_name} · 定位并高亮分支 · 当前分支", "{repo_name} · Locate and highlight branch · Current branch");

pub const HIGHLIGHT_BRANCH: UiMessage = UiMessage::new("views.code_palette_view_catalog.highlight_branch", "{repo_name} · 定位并高亮分支", "{repo_name} · Locate and highlight branch");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    REPO_NAME_BROWSE_TAGGED_FILES,
    REPO_NAME_VIEW_STASH_DIFFERENCES,
    OPEN_A_REPOSITORY_FIRST,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    THIS_REPOSITORY_HAS_NO_AVAILABLE_REMOTE,
    COMPLETE_OR_ABORT_THE_MERGE_FIRST,
    OPEN_REPOSITORY,
    CHOOSE_A_LOCAL_GIT_REPOSITORY_FOLDER,
    COMMANDS,
    OPEN_PROJECT_OPEN_REPOSITORY_REPO,
    CLONE_REPOSITORY,
    ENTER_A_REMOTE_ADDRESS_AND_LOCAL_DIRECTORY,
    DOWNLOAD_PROJECT_CLONE,
    WORKING_TREE,
    VIEW_CHANGES_STAGE_FILES_AND_COMMIT,
    CHANGES_COMMIT_STAGE_UNSTAGE_AMEND_COMMIT_STAGE_UNSTAGE_DIFF_REFRESH,
    HISTORY,
    VIEW_HISTORY_DIFFERENCES_AND_COMMIT_OPERATIONS,
    HISTORY_LOG_UNDO_RESET_REVERT_CHERRY_PICK_RESET_REVERT_LOG,
    COMMIT_GRAPH,
    VIEW_BRANCH_LINEAGE_AND_COMMIT_RELATIONSHIPS,
    GRAPH_BRANCHES_GRAPH,
    WORKFLOWS,
    CHOOSE_TEMPLATES_CONFIGURE_PARAMETERS_AND_VIEW_EXECUTION_LOGS,
    AUTOMATION_TEMPLATES_WORKFLOW,
    CONFLICTS,
    VIEW_CONFLICTS_AND_ACCEPT_BLOCKS_OR_MERGE_WITH_AI,
    MERGE_RESOLVE_MERGE_CONFLICT_AI,
    STASH_LIST,
    VIEW_SAVED_STASHES_AND_FILE_DIFFERENCES,
    STASH_STASH,
    BROWSE_FILES,
    BROWSE_CURRENT_BRANCH_FILES_AND_COMPARE_DIFFERENCES,
    FILES_VIEW_BRANCH_COMPARISON_COMPARE_BROWSE,
    ATTRIBUTION,
    VIEW_THE_ORIGINATING_COMMIT_FOR_EACH_FILE_LINE,
    BLAME_AUTHOR_ATTRIBUTION,
    THIS_REPOSITORY_HAS_NO_CONFLICTS,
    COMMIT_CHANGES,
    WORKING_TREE_STAGE_CHANGES_AND_ENTER_A_COMMIT_MESSAGE_BELOW,
    COMMIT_COMMIT_STAGE_STAGE,
    AI_COMMIT_MESSAGE,
    WORKING_TREE_AI_GENERATION_BUTTON_IN_THE_COMMIT_AREA,
    COMMIT_MESSAGE_COMMIT_MESSAGE,
    AI_CODE_REVIEW,
    WORKING_TREE_SELECT_A_DIFF_AND_OPEN_AI_REVIEW,
    REVIEW_REVIEW,
    REFRESH_REPOSITORY,
    TITLE_BAR_REFRESH_BUTTON,
    REFRESH_REFRESH,
    FETCH_REMOTE_UPDATES,
    TITLE_BAR_FETCH_BUTTON,
    FETCH_FETCH,
    SWITCH_BRANCH,
    WORKING_TREE_SELECT_A_CHECKOUT_TARGET_IN_THE_BRANCH_LIST,
    CHECKOUT_CHECKOUT,
    MERGE_BRANCH,
    WORKING_TREE_MERGE_ACTION_IN_THE_BRANCH_CONTEXT_MENU,
    REBASE,
    WORKING_TREE_REBASE_ACTION_IN_THE_BRANCH_CONTEXT_MENU,
    RENAME_BRANCH,
    WORKING_TREE_RENAME_ACTION_IN_THE_BRANCH_CONTEXT_MENU,
    DELETE_BRANCH,
    WORKING_TREE_DELETE_CONFIRMATION_IN_THE_BRANCH_CONTEXT_MENU,
    COMPARE_BRANCHES,
    WORKING_TREE_COMPARE_ACTION_IN_THE_BRANCH_CONTEXT_MENU,
    SET_UPSTREAM_BRANCH,
    WORKING_TREE_UPSTREAM_SETTING_IN_THE_BRANCH_CONTEXT_MENU,
    UPSTREAM_TRACKING,
    CHERRY_PICK_COMMIT,
    HISTORY_SELECT_A_COMMIT_AND_CHERRY_PICK_FROM_ITS_CONTEXT,
    RESET_TO_COMMIT,
    HISTORY_SELECT_A_COMMIT_AND_RESET_FROM_ITS_CONTEXT_MENU,
    REVERT_COMMIT,
    HISTORY_SELECT_A_COMMIT_AND_REVERT_FROM_ITS_CONTEXT_MENU,
    REVERT_REVERT,
    UNDO_COMMIT,
    HISTORY_SELECT_A_COMMIT_AND_UNDO_FROM_ITS_CONTEXT_MENU,
    FILE_HISTORY,
    WORKING_TREE_HISTORY_ACTION_IN_THE_FILE_CONTEXT_MENU,
    HISTORY_HISTORY,
    SETTINGS_CENTER,
    OPEN_APPLICATION_SETTINGS_AND_PREFERENCES,
    SETTINGS,
    SETTINGS_KEYS_TOKENS_ACCOUNTS_SSH_PASSWORDS_CREDENTIALS_TOKEN,
    SETTINGS_PROXY_SOCKS_HTTP_HTTPS_PROXY,
    SETTINGS_MODELS_PROVIDERS_APIS_SERVICES_PROVIDER_API_AI,
    SETTINGS_EXTERNAL_MERGE_INTELLIJ_IDEA_MERGE_TOOL,
    SETTINGS_CODE_INDEX_KNOWLEDGE_GRAPH_REBUILD_CODE_INDEX,
    SETTINGS_THEME_DARK_LIGHT_COLORS_FONTS_THEME_APPEARANCE,
    SETTINGS_VERSION_UPDATE_CHECKS_DATA_DIRECTORY_MIGRATION_UPDATE,
    SETTINGS_KEYS_KEYBOARD_SHORTCUTS_KEYBOARD_SHORTCUT,
    SETTINGS_ABOUT_VERSION_INFORMATION_ABOUT,
    MANAGE_SKILLS,
    AI_SETTINGS_IMPORT_VIEW_AND_MANAGE_SKILLS,
    SKILLS_EXTENSIONS_SKILLS,
    MCP_SERVICES,
    AI_SETTINGS_CONFIGURE_SERVERS_AND_TOOL_PERMISSIONS,
    TOOLS_EXTENSIONS_MCP_SERVER,
    BROWSER_RUNTIME,
    AI_SETTINGS_CONFIGURE_THE_WORKFLOW_BROWSER_RUNTIME,
    ENABLE_AI_IN_AI_SETTINGS_FIRST,
    CREATE_BRANCH,
    OPEN_THE_NEW_BRANCH_FORM,
    CREATE_BRANCH_2,
    CREATE_TAG,
    CREATE_A_TAG_FOR_THE_CURRENT_HEAD,
    CREATE_TAG_2,
    CREATE_STASH,
    OPEN_THE_STASH_FORM_TO_SAVE_CURRENT_CHANGES,
    SUBMODULES,
    MANAGE_INITIALIZE_AND_UPDATE_SUBMODULES,
    MANAGE_REMOTES,
    ADD_AND_EDIT_REMOTES_AND_CONFIGURE_CREDENTIALS,
    REMOTE_REMOTE_ADDRESS_FETCH_FETCH,
    PULL,
    SELECT_A_REMOTE_AND_BRANCH_TO_PULL,
    PULL_DOWNLOAD,
    PUSH,
    SELECT_A_REMOTE_AND_BRANCH_TO_PUSH,
    PUSH_UPLOAD,
    WORKFLOW_EDITOR,
    CREATE_OR_EDIT_A_WORKFLOW_TEMPLATE,
    TEMPLATE_CREATE_MODIFY_EDIT_WORKFLOW_EDITOR,
    AI_REVIEW_HISTORY,
    VIEW_SAVED_CODE_REVIEW_REPORTS,
    LOCAL_BRANCHES,
    REMOTE_BRANCHES,
    BRANCH_BRANCH,
    TAGS,
    REMOTES,
    STASHES,
    STASH_STASH_2,
    HIGHLIGHT_CURRENT_BRANCH,
    HIGHLIGHT_BRANCH,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
