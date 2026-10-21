//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const STAGE_SELECTED_LINES_COUNT: UiMessage = UiMessage::new("views.diff_view.stage_selected_lines_count", "暂存选中行({count})", "Stage selected lines ({count})");
pub const UNSTAGE_SELECTED_LINES_COUNT: UiMessage = UiMessage::new("views.diff_view.unstage_selected_lines_count", "取消暂存选中行({count})", "Unstage selected lines ({count})");
pub const ENCODING_LABEL: UiMessage = UiMessage::new("views.diff_view.encoding_label", "编码：{label}", "Encoding: {label}");
pub const DIFF: UiMessage = UiMessage::new("views.diff_view.diff", "差异", "Diff");
pub const FULL_FILE: UiMessage = UiMessage::new("views.diff_view.full_file", "全文", "Full file");
pub const SWITCHED_TO_FULL_FILE_VIEW: UiMessage = UiMessage::new("views.diff_view.switched_to_full_file_view", "已切换为全文视图", "Switched to full-file view");
pub const SWITCHED_TO_DIFF_VIEW: UiMessage = UiMessage::new("views.diff_view.switched_to_diff_view", "已切换为差异视图", "Switched to diff view");
pub const FILE_IS_TOO_LARGE_SWITCHED_BACK_TO_DIFF_VIEW: UiMessage = UiMessage::new("views.diff_view.file_is_too_large_switched_back_to_diff_view", "文件过大，已回退到差异视图", "File is too large; switched back to diff view");
pub const WORKING_TREE_DIFF_ENCODING: UiMessage = UiMessage::new("views.diff_view.working_tree_diff_encoding", "工作区差异编码", "Working tree diff encoding");
pub const COMMIT_DIFF_ENCODING: UiMessage = UiMessage::new("views.diff_view.commit_diff_encoding", "提交差异编码", "Commit diff encoding");
pub const RECOVERY_DIFF_ENCODING: UiMessage = UiMessage::new("views.diff_view.recovery_diff_encoding", "恢复差异编码", "Recovery diff encoding");
pub const STASH_DIFF_ENCODING: UiMessage = UiMessage::new("views.diff_view.stash_diff_encoding", "贮藏差异编码", "Stash diff encoding");
pub const BROWSE_ENCODING: UiMessage = UiMessage::new("views.diff_view.browse_encoding", "浏览编码", "Browse encoding");
pub const ATTRIBUTION_ENCODING: UiMessage = UiMessage::new("views.diff_view.attribution_encoding", "追溯编码", "Attribution encoding");
pub const DIFF_METADATA_CLICK_TO_COLLAPSE: UiMessage = UiMessage::new("views.diff_view.diff_metadata_click_to_collapse", "Diff 元信息（点击折叠）", "Diff metadata (click to collapse)");
pub const DIFF_METADATA_CLICK_TO_EXPAND: UiMessage = UiMessage::new("views.diff_view.diff_metadata_click_to_expand", "Diff 元信息（点击展开）", "Diff metadata (click to expand)");
pub const STAGE_HUNK: UiMessage = UiMessage::new("views.diff_view.stage_hunk", "暂存此块", "Stage hunk");
pub const UNSTAGE_HUNK: UiMessage = UiMessage::new("views.diff_view.unstage_hunk", "取消暂存此块", "Unstage hunk");
pub const ONLY_METADATA_IS_SHOWN_FOR_BINARY_FILES: UiMessage = UiMessage::new("views.diff_view.only_metadata_is_shown_for_binary_files", "二进制文件仅显示元信息", "Only metadata is shown for binary files");
pub const NO_TEXT_DIFFERENCES_TO_DISPLAY: UiMessage = UiMessage::new("views.diff_view.no_text_differences_to_display", "没有可显示的文本差异", "No text differences to display");
pub const ATTRIBUTION: UiMessage = UiMessage::new("views.diff_view.attribution", "追溯", "Attribution");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    STAGE_SELECTED_LINES_COUNT,
    UNSTAGE_SELECTED_LINES_COUNT,
    ENCODING_LABEL,
    DIFF,
    FULL_FILE,
    SWITCHED_TO_FULL_FILE_VIEW,
    SWITCHED_TO_DIFF_VIEW,
    FILE_IS_TOO_LARGE_SWITCHED_BACK_TO_DIFF_VIEW,
    WORKING_TREE_DIFF_ENCODING,
    COMMIT_DIFF_ENCODING,
    RECOVERY_DIFF_ENCODING,
    STASH_DIFF_ENCODING,
    BROWSE_ENCODING,
    ATTRIBUTION_ENCODING,
    DIFF_METADATA_CLICK_TO_COLLAPSE,
    DIFF_METADATA_CLICK_TO_EXPAND,
    STAGE_HUNK,
    UNSTAGE_HUNK,
    ONLY_METADATA_IS_SHOWN_FOR_BINARY_FILES,
    NO_TEXT_DIFFERENCES_TO_DISPLAY,
    ATTRIBUTION,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
