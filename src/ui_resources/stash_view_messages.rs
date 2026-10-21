//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const STASH_DOES_NOT_EXIST_STASH_INDEX: UiMessage = UiMessage::new("views.stash_view.stash_does_not_exist_stash_index", "贮藏不存在：{reference}", "Stash does not exist: {reference}");
pub const DELETE_STASH_INDEX: UiMessage = UiMessage::new("views.stash_view.delete_stash_index", "确认删除 {reference}？", "Delete {reference}?");
pub const POP_STASH_INDEX: UiMessage = UiMessage::new("views.stash_view.pop_stash_index", "确认弹出 {reference}？", "Pop {reference}?");
pub const CREATE_STASH: UiMessage = UiMessage::new("views.stash_view.create_stash", "创建贮藏", "Create stash");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.stash_view.open_a_repository_first", "请先打开一个仓库", "Open a repository first");
pub const LOADING_STASH_FILES: UiMessage = UiMessage::new("views.stash_view.loading_stash_files", "正在加载贮藏文件", "Loading stash files");
pub const THIS_STASH_NO_LONGER_EXISTS: UiMessage = UiMessage::new("views.stash_view.this_stash_no_longer_exists", "当前贮藏已不存在", "This stash no longer exists");
pub const LOADING_STASH_DIFFERENCES: UiMessage = UiMessage::new("views.stash_view.loading_stash_differences", "正在加载贮藏差异", "Loading stash differences");
pub const STASH_DIFFERENCES_LOADED: UiMessage = UiMessage::new("views.stash_view.stash_differences_loaded", "贮藏差异已加载", "Stash differences loaded");
pub const NO_STASH_SELECTED: UiMessage = UiMessage::new("views.stash_view.no_stash_selected", "未选择贮藏", "No stash selected");
pub const RIGHT_CLICK_A_STASH_ON_THE_LEFT_AND_CHOOSE_VIEW: UiMessage = UiMessage::new("views.stash_view.right_click_a_stash_on_the_left_and_choose_view", "请在左侧贮藏区右键选择“查看贮藏”", "Right-click a stash on the left and choose “View stash”");
pub const STASH_DETAILS: UiMessage = UiMessage::new("views.stash_view.stash_details", "贮藏详情", "Stash details");
pub const FILES_AND_DIFF_PREVIEW: UiMessage = UiMessage::new("views.stash_view.files_and_diff_preview", "文件与差异预览", "Files and diff preview");
pub const LOADING_STASH_FILES_2: UiMessage = UiMessage::new("views.stash_view.loading_stash_files_2", "贮藏文件加载中...", "Loading stash files…");
pub const THIS_STASH_HAS_NO_FILE_CHANGES: UiMessage = UiMessage::new("views.stash_view.this_stash_has_no_file_changes", "该贮藏没有文件变更", "This stash has no file changes");
pub const SELECT_A_STASH: UiMessage = UiMessage::new("views.stash_view.select_a_stash", "请选择一个贮藏", "Select a stash");
pub const STASH_FILES: UiMessage = UiMessage::new("views.stash_view.stash_files", "贮藏文件", "Stash files");
pub const STASH_DIFF: UiMessage = UiMessage::new("views.stash_view.stash_diff", "贮藏差异", "Stash diff");
pub const LOADING_STASH_DIFFERENCES_2: UiMessage = UiMessage::new("views.stash_view.loading_stash_differences_2", "贮藏差异加载中...", "Loading stash differences…");
pub const SELECT_A_STASH_FILE_TO_VIEW_ITS_DIFFERENCES: UiMessage = UiMessage::new("views.stash_view.select_a_stash_file_to_view_its_differences", "请选择一个贮藏文件查看差异", "Select a stash file to view its differences");
pub const STASH_CURRENT_CHANGES: UiMessage = UiMessage::new("views.stash_view.stash_current_changes", "贮藏当前修改", "Stash current changes");
pub const CREATING_A_STASH_CLEANS_THE_WORKING_TREE_APPLY_OR_POP: UiMessage = UiMessage::new("views.stash_view.creating_a_stash_cleans_the_working_tree_apply_or_pop", "创建贮藏后，当前工作区会回到干净状态，后续可从左侧贮藏区应用或弹出。", "Creating a stash cleans the working tree. Apply or pop the stash later from Stashes on the left.");
pub const INCLUDE_UNTRACKED_FILES: UiMessage = UiMessage::new("views.stash_view.include_untracked_files", "包含未跟踪文件", "Include untracked files");
pub const KEEP_STAGED_CHANGES: UiMessage = UiMessage::new("views.stash_view.keep_staged_changes", "保留已暂存内容", "Keep staged changes");
pub const CANCEL: UiMessage = UiMessage::new("views.stash_view.cancel", "取消", "Cancel");
pub const DELETE_STASH: UiMessage = UiMessage::new("views.stash_view.delete_stash", "删除贮藏", "Delete stash");
pub const DELETED_STASHES_CANNOT_BE_RESTORED_FROM_THE_STASH_LIST: UiMessage = UiMessage::new("views.stash_view.deleted_stashes_cannot_be_restored_from_the_stash_list", "删除后无法从贮藏列表恢复。", "Deleted stashes cannot be restored from the stash list.");
pub const CONFIRM_DELETION: UiMessage = UiMessage::new("views.stash_view.confirm_deletion", "确认删除", "Confirm deletion");
pub const POP_STASH: UiMessage = UiMessage::new("views.stash_view.pop_stash", "弹出贮藏", "Pop stash");
pub const THIS_APPLIES_THE_STASH_TO_THE_WORKING_TREE_AND_REMOVES: UiMessage = UiMessage::new("views.stash_view.this_applies_the_stash_to_the_working_tree_and_removes", "弹出会把贮藏的改动应用到工作区，成功后从贮藏列表移除该条目。", "This applies the stash to the working tree and removes it from the stash list on success.");
pub const CONFIRM_POP: UiMessage = UiMessage::new("views.stash_view.confirm_pop", "确认弹出", "Confirm pop");
pub const VIEW_STASH: UiMessage = UiMessage::new("views.stash_view.view_stash", "查看贮藏", "View stash");
pub const APPLY_STASH: UiMessage = UiMessage::new("views.stash_view.apply_stash", "应用贮藏", "Apply stash");
pub const COPY_STASH_REFERENCE: UiMessage = UiMessage::new("views.stash_view.copy_stash_reference", "复制贮藏引用名", "Copy stash reference");
pub const DELETE_STASH_2: UiMessage = UiMessage::new("views.stash_view.delete_stash_2", "删除贮藏...", "Delete stash…");

pub const DIFF_TITLE: UiMessage = UiMessage::new("views.stash_view.diff_title", "贮藏差异：{path}", "Stash diff: {path}");

pub const FULL_FILE_TITLE: UiMessage = UiMessage::new("views.stash_view.full_file_title", "贮藏全文：{path}", "Stash full file: {path}");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    STASH_DOES_NOT_EXIST_STASH_INDEX,
    DELETE_STASH_INDEX,
    POP_STASH_INDEX,
    CREATE_STASH,
    OPEN_A_REPOSITORY_FIRST,
    LOADING_STASH_FILES,
    THIS_STASH_NO_LONGER_EXISTS,
    LOADING_STASH_DIFFERENCES,
    STASH_DIFFERENCES_LOADED,
    NO_STASH_SELECTED,
    RIGHT_CLICK_A_STASH_ON_THE_LEFT_AND_CHOOSE_VIEW,
    STASH_DETAILS,
    FILES_AND_DIFF_PREVIEW,
    LOADING_STASH_FILES_2,
    THIS_STASH_HAS_NO_FILE_CHANGES,
    SELECT_A_STASH,
    STASH_FILES,
    STASH_DIFF,
    LOADING_STASH_DIFFERENCES_2,
    SELECT_A_STASH_FILE_TO_VIEW_ITS_DIFFERENCES,
    STASH_CURRENT_CHANGES,
    CREATING_A_STASH_CLEANS_THE_WORKING_TREE_APPLY_OR_POP,
    INCLUDE_UNTRACKED_FILES,
    KEEP_STAGED_CHANGES,
    CANCEL,
    DELETE_STASH,
    DELETED_STASHES_CANNOT_BE_RESTORED_FROM_THE_STASH_LIST,
    CONFIRM_DELETION,
    POP_STASH,
    THIS_APPLIES_THE_STASH_TO_THE_WORKING_TREE_AND_REMOVES,
    CONFIRM_POP,
    VIEW_STASH,
    APPLY_STASH,
    COPY_STASH_REFERENCE,
    DELETE_STASH_2,
    DIFF_TITLE,
    FULL_FILE_TITLE,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
