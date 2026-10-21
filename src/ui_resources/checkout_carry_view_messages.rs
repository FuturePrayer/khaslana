//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const TARGET_LABEL_CONTAINS_CHANGES_THAT_CONFLICT_WITH_YOUR_UNCOMMITTED_CHANGES: UiMessage = UiMessage::new("views.checkout_carry_view.target_label_contains_changes_that_conflict_with_your_uncommitted_changes", "{target_label} 包含与当前未提交修改冲突的更改，直接切换会被 Git 拒绝。", "{target_label} contains changes that conflict with your uncommitted changes. Git will reject a direct checkout.");
pub const CHECKOUT_BLOCKED: UiMessage = UiMessage::new("views.checkout_carry_view.checkout_blocked", "切换被阻止", "Checkout blocked");
pub const STASH_THE_CURRENT_CHANGES_INCLUDING_UNTRACKED_FILES_BEFORE_CHECKING_OUT: UiMessage = UiMessage::new("views.checkout_carry_view.stash_the_current_changes_including_untracked_files_before_checking_out", "是否贮藏当前修改（含未跟踪文件）后再切换？", "Stash the current changes (including untracked files) before checking out?");
pub const STASH_AND_CHECKOUT_CHANGES_REMAIN_IN_THE_STASH_APPLY_THEM: UiMessage = UiMessage::new("views.checkout_carry_view.stash_and_checkout_changes_remain_in_the_stash_apply_them", "贮藏并切换：修改留在贮藏中，稍后从左侧「贮藏」手动应用", "Stash and checkout: changes remain in the stash. Apply them later from Stashes on the left.");
pub const CANCEL: UiMessage = UiMessage::new("views.checkout_carry_view.cancel", "取消", "Cancel");
pub const STASH_AND_CHECKOUT: UiMessage = UiMessage::new("views.checkout_carry_view.stash_and_checkout", "贮藏并切换", "Stash and checkout");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    TARGET_LABEL_CONTAINS_CHANGES_THAT_CONFLICT_WITH_YOUR_UNCOMMITTED_CHANGES,
    CHECKOUT_BLOCKED,
    STASH_THE_CURRENT_CHANGES_INCLUDING_UNTRACKED_FILES_BEFORE_CHECKING_OUT,
    STASH_AND_CHECKOUT_CHANGES_REMAIN_IN_THE_STASH_APPLY_THEM,
    CANCEL,
    STASH_AND_CHECKOUT,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
