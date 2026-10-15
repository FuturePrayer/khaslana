use super::*;

#[test]
fn nested_scroll_respects_direction_boundaries_and_fractional_pixels() {
    assert_eq!(nested_vertical_offset(px(-50.0), px(100.0), px(-0.25)), Some(px(-50.25)));
    assert_eq!(nested_vertical_offset(px(-50.0), px(100.0), px(10.0)), Some(px(-40.0)));
    assert_eq!(nested_vertical_offset(px(0.0), px(100.0), px(10.0)), None);
    assert_eq!(nested_vertical_offset(px(-100.0), px(100.0), px(-10.0)), None);
    assert_eq!(nested_vertical_offset(px(0.0), px(0.0), px(-10.0)), None);
    assert_eq!(nested_vertical_offset(px(-50.0), px(100.0), px(0.0)), None);
    assert_eq!(nested_vertical_offset(px(-90.0), px(100.0), px(-80.0)), Some(px(-100.0)));
}

#[test]
fn drag_ends_on_modal_transition_or_mouse_release_not_within_same_layer() {
    use crate::TopOverlayKind;
    let settings = Some((TopOverlayKind::Settings, None));
    let dialog = Some((TopOverlayKind::Dialog, None));
    assert!(!pointer_drag_should_end(&None, &None, true));
    assert!(!pointer_drag_should_end(&settings, &settings, true));
    assert!(pointer_drag_should_end(&None, &settings, true));
    assert!(pointer_drag_should_end(&settings, &dialog, true));
    assert!(pointer_drag_should_end(&settings, &None, true));
    assert!(pointer_drag_should_end(&settings, &settings, false));
}

#[test]
fn drag_start_requires_bubble_hitbox_and_left_button() {
    use gpui::DispatchPhase::{Bubble, Capture};
    assert!(pointer_drag_can_start(Bubble, true, MouseButton::Left));
    assert!(!pointer_drag_can_start(Capture, true, MouseButton::Left));
    assert!(!pointer_drag_can_start(Bubble, false, MouseButton::Left));
    assert!(!pointer_drag_can_start(Bubble, true, MouseButton::Right));
}

struct PointerFixture {
    modal: bool,
    starts: usize,
    inside: usize,
}

impl gpui::Render for PointerFixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div().relative().size_full().child(canvas(
            |bounds, window, _| window.insert_hitbox(bounds, gpui::HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if pointer_drag_can_start(phase, hitbox.is_hovered(window), event.button) {
                        entity.update(cx, |this, _| this.starts += 1);
                    }
                });
            },
        ).size_full()).when(self.modal, |this| this.child(
            crate::ui::components::dialog_overlay().child(div().id("modal-child")
                .size_full().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    this.inside += 1;
                }))),
        ))
    }
}

#[gpui::test]
fn modal_hitbox_blocks_background_drag_but_normal_view_allows_it(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| PointerFixture { modal: false, starts: 0, inside: 0 });
    draw_nested(cx);
    let down = || gpui::PlatformInput::MouseDown(MouseDownEvent {
        button: MouseButton::Left, position: point(px(20.0), px(20.0)), ..Default::default()
    });
    cx.update(|window, cx| { window.dispatch_event(down(), cx); });
    cx.read(|cx| assert_eq!(view.read(cx).starts, 1));
    view.update(cx, |this, cx| { this.modal = true; cx.notify(); });
    draw_nested(cx);
    cx.update(|window, cx| { window.dispatch_event(down(), cx); });
    cx.read(|cx| {
        assert_eq!(view.read(cx).starts, 1);
        assert_eq!(view.read(cx).inside, 1, "模态内部仍能接收鼠标事件");
    });
}

struct NestedScrollFixture {
    inner: ScrollHandle,
    outer: ScrollHandle,
}

impl gpui::Render for NestedScrollFixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let content = div().id("test-inner").w_full().h(px(100.0)).overflow_y_scroll()
            .restrict_scroll_to_axis().track_scroll(&self.inner)
            .child(div().h(px(400.0)).w_full());
        div().id("test-outer").w(px(200.0)).h(px(180.0)).overflow_y_scroll()
            .track_scroll(&self.outer).child(div().h(px(100.0)).w_full()
                .child(nested_vertical_scroll(content, self.inner.clone())))
            .child(div().h(px(400.0)).w_full())
    }
}

fn draw_nested(cx: &mut gpui::VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
}

#[gpui::test]
fn nested_scroll_dispatches_inner_before_outer(cx: &mut gpui::TestAppContext) {
    let inner = ScrollHandle::new();
    let outer = ScrollHandle::new();
    let (_, cx) = cx.add_window_view(|_, _| NestedScrollFixture {
        inner: inner.clone(), outer: outer.clone(),
    });
    draw_nested(cx);
    let position = point(px(30.0), px(30.0));
    let wheel = |dy| gpui::PlatformInput::ScrollWheel(gpui::ScrollWheelEvent {
        position, delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(dy))),
        ..Default::default()
    });
    cx.update(|window, cx| { window.dispatch_event(wheel(-25.5), cx); });
    draw_nested(cx);
    assert_eq!(inner.offset().y, px(-25.5));
    assert_eq!(outer.offset().y, px(0.0));
    cx.update(|window, cx| {
        window.dispatch_event(gpui::PlatformInput::ScrollWheel(gpui::ScrollWheelEvent {
            position, delta: gpui::ScrollDelta::Pixels(point(px(-15.0), px(0.0))),
            ..Default::default()
        }), cx);
    });
    draw_nested(cx);
    assert_eq!(inner.offset().y, px(-25.5), "纯横向不能映射为内层纵滚");
    inner.set_offset(point(px(0.0), -inner.max_offset().y));
    draw_nested(cx);
    cx.update(|window, cx| { window.dispatch_event(wheel(-20.0), cx); });
    draw_nested(cx);
    assert!(outer.offset().y < px(0.0));
    // 反方向仍由内层先处理；超过边界的本次余量不推动外层。
    let outer_before = outer.offset();
    cx.update(|window, cx| { window.dispatch_event(wheel(1000.0), cx); });
    draw_nested(cx);
    assert_eq!(inner.offset().y, px(0.0));
    assert_eq!(outer.offset(), outer_before);
    cx.update(|window, cx| { window.dispatch_event(wheel(10.0), cx); });
    draw_nested(cx);
    assert!(outer.offset().y > outer_before.y);
}


#[test]
fn change_state_color_uses_semantic_status_colors() {
    // GitHub 风格 Git 状态区分色：绿增 / 蓝改 / 红删 / 亮红冲突 / 橙更名 / 灰未跟踪
    assert_eq!(change_state_color(&ChangeState::Added), ui_theme::GIT_ADDED);
    assert_eq!(
        change_state_color(&ChangeState::Modified),
        ui_theme::GIT_MODIFIED
    );
    assert_eq!(
        change_state_color(&ChangeState::Deleted),
        ui_theme::GIT_REMOVED
    );
    assert_eq!(
        change_state_color(&ChangeState::Conflicted),
        ui_theme::DESTRUCTIVE
    );
    assert_eq!(
        change_state_color(&ChangeState::Renamed),
        ui_theme::GIT_RENAMED
    );
    assert_eq!(
        change_state_color(&ChangeState::Typechange),
        ui_theme::GIT_RENAMED
    );
    assert_eq!(
        change_state_color(&ChangeState::Untracked),
        ui_theme::GIT_UNTRACKED
    );
}

#[test]
fn repo_initials_multi_segment() {
    // 多段名取前两段首字母。
    assert_eq!(repo_initials("mc-manager"), "MM");
    assert_eq!(repo_initials("opencl-ffm"), "OF");
    assert_eq!(repo_initials("a/b/c"), "AB");
}

#[test]
fn repo_initials_single_word() {
    // 单词名：首字母 + 首个内部大写字母；纯小写仅首字母。
    assert_eq!(repo_initials("EasyTier"), "ET");
    assert_eq!(repo_initials("qqBot"), "QB");
    assert_eq!(repo_initials("khaslana"), "K");
    assert_eq!(repo_initials("optical"), "O");
}

#[test]
fn repo_initials_empty() {
    assert_eq!(repo_initials(""), "?");
    assert_eq!(repo_initials("---"), "?");
}
