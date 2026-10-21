//! 仓库操作的表单守卫、加载与剪贴板反馈。
use khaslana::ui_i18n::UiMessage;
pub const OPEN_FAILED: UiMessage = UiMessage::new("repository.open.failed", "打开仓库失败", "Could not open repository");
pub const NOT_REPOSITORY: UiMessage = UiMessage::new("repository.not_repository", "该目录不是 Git 仓库", "This folder is not a Git repository");
pub const OPENING: UiMessage = UiMessage::new("repository.open.started", "正在打开仓库", "Opening repository");
pub const OPENED: UiMessage = UiMessage::new("repository.open.completed", "仓库已打开", "Repository opened");
pub const CLONE_REQUIRED: UiMessage = UiMessage::new("repository.clone.required", "需要填写远程仓库 URL 和克隆到父文件夹", "Enter the remote repository URL and destination parent folder");
pub const CLONE_NAME: UiMessage = UiMessage::new("repository.clone.folder_name", "无法从远程仓库 URL 推导仓库文件夹名", "Could not determine the repository folder name from the remote URL");
pub const TARGET_EXISTS: UiMessage = UiMessage::new("repository.clone.target_exists", "目标仓库文件夹已存在", "The destination repository folder already exists");
pub const ALREADY_OPEN: UiMessage = UiMessage::new("repository.already_open", "该仓库已经打开", "This repository is already open");
pub const CLONING: UiMessage = UiMessage::new("operation.cloned.started", "正在克隆仓库", "Cloning repository");
pub const OPEN_REQUIRED: UiMessage = UiMessage::new("repository.open_required", "请先打开一个仓库", "Open a repository first");
pub const REFRESHING: UiMessage = crate::operation_ui_resources::REFRESH_STARTED;
pub const WAITING: UiMessage = UiMessage::new("repository.load.waiting", "等待加载仓库", "Waiting to load repository");
pub const BRANCHES_LOADED: UiMessage = UiMessage::new("repository.load.branches", "本地分支已加载，正在加载仓库详情", "Local branches loaded; loading repository details");
pub const METADATA_LOADED: UiMessage = UiMessage::new("repository.load.metadata", "仓库信息已加载", "Repository information loaded");
pub const FAST_LOADED: UiMessage = UiMessage::new("repository.load.fast", "快速变更已加载，正在补全未跟踪文件", "Changes loaded; loading untracked files");
pub const BUSY: UiMessage = UiMessage::new("repository.busy", "已有操作正在运行", "An operation is already running");
pub const NO_REMOTE: UiMessage = UiMessage::new("repository.no_remote", "当前仓库没有远端", "This repository has no remotes");
pub const PULL_PUSH: UiMessage = UiMessage::new("repository.action.pull_push", "拉取或推送", "pull or push");
pub const BRANCH_MISSING: UiMessage = UiMessage::new("repository.branch_missing", "本地分支不存在：{branch}", "Local branch does not exist: {branch}");
pub const NOT_LOCAL_BRANCH: UiMessage = UiMessage::new("repository.not_local_branch", "当前不是本地分支，无法拉取、推送或设置 upstream", "The current reference is not a local branch; pull, push, and upstream configuration are unavailable");
pub const REMOTE_BRANCH_REQUIRED: UiMessage = UiMessage::new("repository.remote_branch_required", "需要填写远程分支", "Enter a remote branch");
pub const REMOTE_BRANCH_MISSING: UiMessage = UiMessage::new("repository.remote_branch_missing", "远端分支不存在，请点击刷新或选择已有分支", "The remote branch does not exist. Refresh or choose an existing branch.");
pub const CHECKOUT: UiMessage = UiMessage::new("repository.action.checkout", "切换分支", "switch branches");
pub const BRANCH_REQUIRED: UiMessage = UiMessage::new("repository.branch_required", "需要填写分支名称", "Enter a branch name");
pub const NEW_BRANCH_REQUIRED: UiMessage = UiMessage::new("repository.new_branch_required", "需要填写新的分支名称", "Enter a new branch name");
pub const MERGE_AGAIN: UiMessage = UiMessage::new("repository.action.merge_again", "再次合并", "merge again");
pub const CHECKOUT_REMOTE: UiMessage = UiMessage::new("repository.action.checkout_remote", "切换远端分支", "check out a remote branch");
pub const CHECKOUT_TAG: UiMessage = UiMessage::new("repository.action.checkout_tag", "检出标签", "check out a tag");
pub const TAG_REQUIRED: UiMessage = UiMessage::new("repository.tag_required", "请填写标签名称", "Enter a tag name");
pub const NO_REMOTE_TAG: UiMessage = UiMessage::new("repository.tag.no_remote", "当前仓库没有远端，无法推送标签", "This repository has no remotes to push the tag to");
pub const APPLY_STASH: UiMessage = UiMessage::new("repository.action.apply_stash", "应用贮藏", "apply a stash");
pub const POP_STASH: UiMessage = UiMessage::new("repository.action.pop_stash", "弹出贮藏", "pop a stash");
pub const NO_DISCARD_FILES: UiMessage = UiMessage::new("repository.discard.no_files", "没有可回滚的文件", "No files to discard");
pub const RESET: UiMessage = UiMessage::new("repository.action.reset", "重置提交", "reset commits");
pub const REVERT: UiMessage = UiMessage::new("repository.action.revert", "回滚提交", "revert a commit");
pub const REVERT_MERGE: UiMessage = UiMessage::new("repository.action.revert_merge", "撤销合并提交", "revert a merge commit");
pub const UNDO_COMMIT: UiMessage = UiMessage::new("repository.action.undo_commit", "还原提交到暂存区", "restore a commit to the staging area");
pub const DISCARD: UiMessage = UiMessage::new("repository.action.discard", "回滚工作区更改", "discard worktree changes");
pub const DISCARDED_FILE: UiMessage = UiMessage::new("operation.discarded_file", "已回滚文件全部更改", "All changes to the file discarded");
pub const DISCARDED_FILES: UiMessage = UiMessage::new("operation.discarded_files", "已回滚选定文件全部更改", "All changes to selected files discarded");
pub const DISCARDED_STAGED: UiMessage = UiMessage::new("operation.discarded_staged", "已回滚暂存区全部更改", "All staged changes discarded");
pub const DISCARDED_UNSTAGED: UiMessage = UiMessage::new("operation.discarded_unstaged", "已回滚未暂存更改", "Unstaged changes discarded");
pub const DISCARDED_SELECTED_UNSTAGED: UiMessage = UiMessage::new("operation.discarded_selected_unstaged", "已回滚选定未暂存更改", "Selected unstaged changes discarded");
pub const DISCARDED_ALL_UNSTAGED: UiMessage = UiMessage::new("operation.discarded_all_unstaged", "已回滚修改区全部更改", "All unstaged changes discarded");
pub const DISCARDING: UiMessage = UiMessage::new("operation.discard.started", "正在回滚文件更改", "Discarding file changes");
pub const SHA_COPIED: UiMessage = UiMessage::new("clipboard.sha.copied", "已复制提交 SHA", "Commit SHA copied");
pub const NO_OPEN_REPO: UiMessage = UiMessage::new("repository.no_open_repository", "当前没有打开的仓库", "No repository is open");
pub const PATH_COPIED: UiMessage = UiMessage::new("clipboard.path.copied", "已复制文件绝对路径", "Absolute file path copied");
pub const NO_PARENT: UiMessage = UiMessage::new("repository.file.no_parent", "无法确定文件所在目录", "Could not determine the containing folder");
pub const PARENT_MISSING: UiMessage = UiMessage::new("repository.file.parent_missing", "文件所在目录不存在：{path}", "The containing folder does not exist: {path}");
pub const PARENT_OPENED: UiMessage = UiMessage::new("repository.parent.opened", "已打开文件所在目录", "Containing folder opened");
pub const PARENT_FAILED: UiMessage = UiMessage::new("repository.parent.failed", "打开文件所在目录失败：{error}", "Could not open the containing folder: {error}");
pub const EXPLORER_OPENED: UiMessage = UiMessage::new("repository.explorer.opened", "已在资源管理器中打开仓库", "Repository opened in the file manager");
pub const EXPLORER_FAILED: UiMessage = UiMessage::new("repository.explorer.failed", "打开仓库目录失败：{error}", "Could not open the repository folder: {error}");
pub const REMOTE_URL_MISSING: UiMessage = UiMessage::new("repository.remote_url_missing", "远端 URL 为空或未找到", "Remote URL is empty or unavailable");
pub const BROWSER_OPENED: UiMessage = UiMessage::new("repository.browser.opened", "已在浏览器中打开 {url}", "Opened {url} in the browser");
pub const BRANCH_COPIED: UiMessage = UiMessage::new("clipboard.branch.copied", "已复制分支名称", "Branch name copied");
pub const CHECKOUT_COPIED: UiMessage = UiMessage::new("clipboard.checkout.copied", "已复制 checkout 命令", "Checkout command copied");
pub const REMOTE_COPIED: UiMessage = UiMessage::new("clipboard.remote.copied", "已复制远端名称", "Remote name copied");
pub const TAG_COPIED: UiMessage = UiMessage::new("clipboard.tag.copied", "已复制标签名称", "Tag name copied");
pub const STASH_COPIED: UiMessage = UiMessage::new("clipboard.stash.copied", "已复制贮藏引用名", "Stash reference copied");

#[cfg(test)]
#[test]
fn resources_have_unique_keys_and_matching_parameters() {
    khaslana::ui_i18n::validate(&[OPEN_FAILED, NOT_REPOSITORY, OPENING, OPENED, CLONE_REQUIRED, CLONE_NAME, TARGET_EXISTS, ALREADY_OPEN, CLONING, OPEN_REQUIRED, REFRESHING, WAITING, BRANCHES_LOADED, METADATA_LOADED, FAST_LOADED, BUSY, NO_REMOTE, PULL_PUSH, BRANCH_MISSING, NOT_LOCAL_BRANCH, REMOTE_BRANCH_REQUIRED, REMOTE_BRANCH_MISSING, CHECKOUT, BRANCH_REQUIRED, NEW_BRANCH_REQUIRED, MERGE_AGAIN, CHECKOUT_REMOTE, CHECKOUT_TAG, TAG_REQUIRED, NO_REMOTE_TAG, APPLY_STASH, POP_STASH, NO_DISCARD_FILES, RESET, REVERT, REVERT_MERGE, UNDO_COMMIT, DISCARD, DISCARDED_FILE, DISCARDED_FILES, DISCARDED_STAGED, DISCARDED_UNSTAGED, DISCARDED_SELECTED_UNSTAGED, DISCARDED_ALL_UNSTAGED, DISCARDING, SHA_COPIED, NO_OPEN_REPO, PATH_COPIED, NO_PARENT, PARENT_MISSING, PARENT_OPENED, PARENT_FAILED, EXPLORER_OPENED, EXPLORER_FAILED, REMOTE_URL_MISSING, BROWSER_OPENED, BRANCH_COPIED, CHECKOUT_COPIED, REMOTE_COPIED, TAG_COPIED, STASH_COPIED]).unwrap();
}
