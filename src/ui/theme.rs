pub(crate) const APP_BG: u32 = 0xf7fbff;
pub(crate) const APP_BG_ALT: u32 = 0xeaf4ff;
pub(crate) const SURFACE: u32 = 0xffffff;
pub(crate) const SURFACE_MUTED: u32 = 0xf4f8ff;
pub(crate) const SURFACE_HOVER: u32 = 0xf0f6ff;
pub(crate) const PANEL_BG: u32 = 0xfbfdff;
pub(crate) const PANEL_TINT: u32 = 0xf7fbff;
pub(crate) const HEADER_BG: u32 = 0xf6faff;
pub(crate) const GLASS_BG: u32 = 0xfffffff0;
pub(crate) const GLASS_BG_STRONG: u32 = 0xfffffff8;
pub(crate) const GLASS_BORDER: u32 = 0xd7e5f8dd;
pub(crate) const BORDER: u32 = 0xcbdaf0;
pub(crate) const BORDER_MUTED: u32 = 0xe4edf8;
pub(crate) const BORDER_STRONG: u32 = 0x93b4e8;
pub(crate) const ACCENT: u32 = 0x2f6fea;
pub(crate) const ACCENT_STRONG: u32 = 0x1f5fd6;
pub(crate) const ACCENT_SOFT: u32 = 0xeaf3ff;
pub(crate) const ACCENT_VIVID: u32 = 0x6f9ff0;
pub(crate) const ACCENT_VIVID_SOFT: u32 = 0xf1f7ff;
pub(crate) const FOCUS_RING: u32 = 0x9bbcf2;
pub(crate) const TEXT: u32 = 0x10233f;
pub(crate) const TEXT_MUTED: u32 = 0x52627a;
pub(crate) const TEXT_FAINT: u32 = 0x91a0b5;
pub(crate) const ROW_SELECTED: u32 = 0xe8f2ff;
pub(crate) const ROW_HOVER: u32 = 0xf2f7ff;
pub(crate) const ROW_SELECTED_BORDER: u32 = 0xaec9f4;
pub(crate) const HASH_BG: u32 = 0xeef5ff;
pub(crate) const DANGER: u32 = 0xe5484d;
pub(crate) const DANGER_STRONG: u32 = 0xc62a31;
pub(crate) const DANGER_BORDER: u32 = 0xd33b40;
pub(crate) const DANGER_SOFT: u32 = 0xfef2f2;
pub(crate) const DANGER_BORDER_SOFT: u32 = 0xfca5a5;
pub(crate) const DANGER_TEXT: u32 = 0x991b1b;
#[allow(dead_code)]
pub(crate) const SUCCESS: u32 = 0x30a46c;
#[allow(dead_code)]
pub(crate) const SUCCESS_SOFT: u32 = 0xe9f7ef;
#[allow(dead_code)]
pub(crate) const WARNING: u32 = 0xf59f00;
#[allow(dead_code)]
pub(crate) const WARNING_SOFT: u32 = 0xfff7e6;
pub(crate) const WARNING_TEXT: u32 = 0x92400e;
pub(crate) const WARNING_HOVER: u32 = 0xfff0d1;
pub(crate) const WARNING_ACCENT_TEXT: u32 = 0xa15c00;
pub(crate) const WARNING_BADGE_BG: u32 = 0xfff2d9;
pub(crate) const TYPECHANGE: u32 = 0x7c3aed;
pub(crate) const BADGE_BG: u32 = 0x2f6fea;
pub(crate) const BADGE_BORDER: u32 = 0xdbeafe;
pub(crate) const TOOLTIP_BG: u32 = 0x10233f;
pub(crate) const TOOLTIP_BORDER: u32 = 0x243654;
pub(crate) const DIALOG_OVERLAY: u32 = 0x0f172a55;
pub(crate) const INPUT_BG: u32 = 0xfffffff5;
pub(crate) const INPUT_BG_FOCUSED: u32 = 0xfffffffa;
pub(crate) const INPUT_BORDER: u32 = 0xb9cbed;
pub(crate) const INPUT_BORDER_FOCUSED: u32 = 0x2f6fea;
pub(crate) const INPUT_PLACEHOLDER: u32 = 0x8fa0b8;
pub(crate) const INPUT_SELECTION: u32 = 0x2f6fea33;
pub(crate) const INPUT_CARET: u32 = 0x10233f;
pub(crate) const SEGMENT_BG: u32 = 0xf2f6fd;
pub(crate) const SEGMENT_SELECTED_BG: u32 = 0xe2eeff;
pub(crate) const SEGMENT_SELECTED_TEXT: u32 = 0x1f5fd6;
pub(crate) const HISTORY_GRAPH_COLORS: [u32; 8] = [
    0xf97316, 0x14b8a6, 0x3b82f6, 0xeab308, 0xef4444, 0x8b5cf6, 0x22c55e, 0xec4899,
];
pub(crate) const REF_LOCAL_BG: u32 = 0xedf7f1;
pub(crate) const REF_LOCAL_BORDER: u32 = 0xa8d5bc;
pub(crate) const REF_LOCAL_TEXT: u32 = 0x28784f;
pub(crate) const REF_REMOTE_BG: u32 = 0xeef5ff;
pub(crate) const REF_REMOTE_BORDER: u32 = 0xb8cbed;
pub(crate) const REF_REMOTE_TEXT: u32 = 0x3b5f8f;
pub(crate) const REF_TAG_BG: u32 = 0xf8f3e8;
pub(crate) const REF_TAG_BORDER: u32 = 0xe4cf9e;
pub(crate) const REF_TAG_TEXT: u32 = 0x856214;
pub(crate) const REF_HEAD_BG: u32 = 0x2f3742;
pub(crate) const REF_HEAD_TEXT: u32 = 0xffffff;
pub(crate) const FEEDBACK_BG: u32 = 0xfffffff6;
pub(crate) const FEEDBACK_INFO_BG: u32 = 0xeaf3ff;
pub(crate) const FEEDBACK_INFO_BORDER: u32 = 0xaec9f4;
pub(crate) const FEEDBACK_INFO_TEXT: u32 = 0x1f5fd6;
pub(crate) const FEEDBACK_SUCCESS_BG: u32 = 0xe9f7ef;
pub(crate) const FEEDBACK_SUCCESS_BORDER: u32 = 0xa8d5bc;
pub(crate) const FEEDBACK_SUCCESS_TEXT: u32 = 0x28784f;
pub(crate) const FEEDBACK_WARNING_BG: u32 = 0xfff7e6;
pub(crate) const FEEDBACK_WARNING_BORDER: u32 = 0xe4cf9e;
pub(crate) const FEEDBACK_WARNING_TEXT: u32 = 0xa15c00;
pub(crate) const FEEDBACK_ERROR_BG: u32 = 0xffe4e6;
pub(crate) const FEEDBACK_ERROR_BORDER: u32 = 0xf1a1a5;
pub(crate) const FEEDBACK_ERROR_TEXT: u32 = 0xc62a31;
pub(crate) const PROGRESS_TRACK: u32 = 0xe4edf8;
pub(crate) const PROGRESS_FILL: u32 = 0x2f6fea;

// 侧边栏使用低透明度色块模拟设计图里的蓝紫纵向渐变，避免内容区域过度抢眼。
pub(crate) const SIDEBAR_GRADIENT_TOP: u32 = 0xe5f2ffc7;
pub(crate) const SIDEBAR_GRADIENT_BOTTOM: u32 = 0xf2e8ffc4;
pub(crate) const SIDEBAR_GRADIENT_SOFTEN: u32 = 0xffffff66;

// 滚动条和 diff 行色也纳入主题，保证浅蓝白主题下不会出现突兀硬编码色。
pub(crate) const SCROLLBAR_TRACK: u32 = 0xf0f6ffcc;
pub(crate) const SCROLLBAR_THUMB: u32 = 0xb7d0f4dd;
pub(crate) const SCROLLBAR_THUMB_ACTIVE: u32 = 0x84ace8ee;
pub(crate) const DIFF_ADDED_BG: u32 = 0xedfbf4;
pub(crate) const DIFF_ADDED_TEXT: u32 = 0x168447;
pub(crate) const DIFF_REMOVED_BG: u32 = 0xfff0f2;
pub(crate) const DIFF_REMOVED_TEXT: u32 = 0xb8323a;
pub(crate) const DIFF_HEADER_BG: u32 = 0xeef5ff;
pub(crate) const DIFF_HEADER_TEXT: u32 = 0x52627a;
