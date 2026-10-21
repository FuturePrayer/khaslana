use super::*;
#[test]
fn utf16_offsets_never_split_surrogate_pairs() {
    assert_eq!(from_utf16("a🦀中文",3),5);
    assert_eq!(from_utf16("a🦀中文",4),8);
    assert_eq!(from_utf16("a🦀中文",999),11);
}

fn fixture() -> ConflictFileView {
    let draft=format!("head\nbase\n{}tail\n",(0..80).map(|i|format!("context {i}\n")).collect::<String>());
    ConflictFileView {path:"file.rs".into(),kind:khaslana::ConflictFileKind::Text,ours_text:draft.clone(),theirs_text:draft.clone(),draft,
        blocks:vec![khaslana::ConflictBlock {base:Some("base\n".into()),ours:"ours\n".into(),theirs:"theirs\nextra\n".into(),start:5,end:10,ours_start:5,ours_end:10,theirs_start:5,theirs_end:10,status:ConflictBlockStatus::Unresolved,has_manual_edits:false}],
        draft_status:khaslana::ConflictDraftStatus::Clean,fallback_reason:None}
}

fn harness(cx: &mut gpui::TestAppContext) -> (gpui::Entity<NativeEditor>, &mut gpui::VisualTestContext) {
    cx.update(|cx|gpui_kit::init(cx));
    let (view,cx)=cx.add_window_view(|_,cx|NativeEditor::new(fixture(),cx));
    cx.update(|window,cx| {window.resize(size(px(900.0),px(480.0)));window.focus(&view.read(cx).focus_handle(cx),cx);});
    draw(cx);
    (view,cx)
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window,cx|window.draw(cx).clear(cx));
    cx.run_until_parked();
}

#[gpui::test]
fn native_candidate_button_is_clickable_and_undo_restores_expansion(cx: &mut gpui::TestAppContext) {
    let (view,cx)=harness(cx);
    // 选择器使用动作身份，不使用随 UI 语言变化的按钮文案；后续仍实际点击并验证结果和撤销。
    let bounds=cx.debug_bounds("keep-0-Ours").expect("块头必须参与布局和命中");
    cx.update(|window,cx| {
        window.dispatch_event(gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {button:MouseButton::Left,position:bounds.center(),click_count:1,..Default::default()}),cx);
        window.dispatch_event(gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {button:MouseButton::Left,position:bounds.center(),click_count:1,..Default::default()}),cx);
    });
    draw(cx);
    cx.read(|cx| {let e=view.read(cx);assert!(e.document.view.draft.starts_with("head\nours\n"));assert!(!e.document.expanded.contains(&0));});
    view.update(cx,|e,cx|e.history(false,cx));draw(cx);
    cx.read(|cx| {let e=view.read(cx);assert_eq!(e.document.view.draft,fixture().draft);assert!(e.document.expanded.contains(&0));});
}

#[gpui::test]
fn ime_ranges_are_result_coordinates_and_one_undo_restores_original(cx: &mut gpui::TestAppContext) {
    let (view,cx)=harness(cx);
    cx.update(|window,cx|view.update(cx,|e,cx| {
        e.document.anchor=5;e.document.caret=5;
        e.replace_and_mark_text_in_range(None,"中文🦀",Some(2..4),window,cx);
        assert_eq!(e.selected_text_range(false,window,cx).unwrap().range,7..9);
        assert_eq!(e.marked_text_range(window,cx),Some(5..9));
    }));draw(cx);
    cx.update(|window,cx|view.update(cx,|e,cx| {
        let rect=e.bounds_for_range(7..9,e.viewport,window,cx).unwrap();
        assert!(rect.top()>e.viewport.top()+px(100.0),"候选必须增加显示高度而不改变输入法文档偏移");
        e.replace_text_in_range(None,"中文🦀",window,cx);
        e.history(false,cx);
        assert_eq!(e.document.view.draft,fixture().draft);
    }));
}

#[gpui::test]
fn virtual_rows_and_scrollbar_end_share_the_same_extent(cx: &mut gpui::TestAppContext) {
    let (view,cx)=harness(cx);
    view.update(cx,|e,cx| {e.scroll.y=px(100000.0);cx.notify();});draw(cx);
    cx.read(|cx| {
        let e=view.read(cx);assert_eq!(e.scroll.y,e.max_scroll().y);
        assert!(e.geometry.len() <= (f32::from(e.viewport.size.height)/LINE).ceil() as usize + 1,"只整形可见结果行");
        let final_row=e.geometry.last().unwrap();
        assert_eq!(final_row.range.end,e.document.view.draft.len());
        assert!((f32::from(final_row.origin.y+px(LINE)-e.viewport.bottom())+TRACK).abs()<0.1);
        let (extent,thumb,max)=e.bar_metrics(true);
        assert!((f32::from(e.scroll.y)/max*(extent-thumb)+thumb-extent).abs()<0.1);
    });
}

#[gpui::test]
fn readonly_blocks_typing_and_history_but_allows_selection(cx: &mut gpui::TestAppContext) {
    let (view,cx)=harness(cx);
    cx.update(|window,cx|view.update(cx,|e,cx| {e.readonly=true;e.replace_text_in_range(None,"mutated",window,cx);e.history(false,cx);assert_eq!(e.document.view.draft,fixture().draft);}));
}
