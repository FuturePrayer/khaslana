//! 切换被未提交修改阻止时的确认弹窗：贮藏后切换。

#[path = "ui_resources/checkout_carry_view_messages.rs"]
mod messages;

use crate::*;

impl RepositoryView {
    pub(crate) fn render_confirm_carry_checkout_dialog(
        &self,
        target: &CheckoutTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let target_label = target.label();
        self.dialog_panel(khaslana::ui_i18n::text(messages::CHECKOUT_BLOCKED), cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                    .child(khaslana::ui_i18n::format(messages::TARGET_LABEL_CONTAINS_CHANGES_THAT_CONFLICT_WITH_YOUR_UNCOMMITTED_CHANGES, &[("target_label", &(target_label).to_string())])),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(khaslana::ui_i18n::text(messages::STASH_THE_CURRENT_CHANGES_INCLUDING_UNTRACKED_FILES_BEFORE_CHECKING_OUT)),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(khaslana::ui_i18n::text(messages::STASH_AND_CHECKOUT_CHANGES_REMAIN_IN_THE_STASH_APPLY_THEM)),
            )
            .child(
                dialog_actions()
                    .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.primary_button(
                        khaslana::ui_i18n::text(messages::STASH_AND_CHECKOUT),
                        !self.busy,
                        {
                            let target = target.clone();
                            move |this, _, _| this.confirm_carry_checkout_stash_only(target.clone())
                        },
                        cx,
                    )),
            )
    }
}
