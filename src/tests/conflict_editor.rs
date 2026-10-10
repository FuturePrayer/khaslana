use super::*;
use khaslana::{ConflictBlock, ConflictBlockResolution};

fn view() -> ConflictFileView {
    let draft = "head\nfirst\nmiddle\nsecond\nend\n".to_string();
    let blocks = ["first\n", "second\n"].into_iter().map(|text| {
        let start = draft.find(text).unwrap();
        ConflictBlock {
            base: Some(text.into()), ours: format!("ours-{text}"), theirs: format!("theirs-{text}"),
            start, end: start + text.len(), ours_start: start, ours_end: start + text.len(),
            theirs_start: start, theirs_end: start + text.len(), status: ConflictBlockStatus::Unresolved,
            has_manual_edits: false,
        }
    }).collect();
    ConflictFileView { path: "file.txt".into(), kind: ConflictFileKind::Text,
        ours_text: draft.clone(), theirs_text: draft.clone(), draft,
        blocks, draft_status: ConflictDraftStatus::Clean, fallback_reason: None }
}

#[test]
fn atomic_replacement_preserves_unicode_context_and_crlf() {
    let old = "前文\r\n旧值🦀\r\n后文\r\n";
    let new = "前文\r\n新值🙂🙂\r\n后文\r\n";
    let (range, text) = text_replacement(old, new);
    assert!(old.is_char_boundary(range.start) && old.is_char_boundary(range.end));
    let mut applied = old.to_string();
    applied.replace_range(range, text);
    assert_eq!(applied, new);
}

#[test]
fn revision_restores_block_choice_and_ranges_after_manual_edits() {
    let mut view = view();
    view.apply_block_resolution(0, ConflictBlockResolution::BothTheirsFirst);
    let text = view.draft.clone();
    let blocks = view.blocks.clone();
    let revision = ConflictRevision::new(&view);
    view.set_draft(format!("前缀\n{}", view.draft));
    revision.restore(text, &mut view);
    assert_eq!(view.blocks, blocks);
    assert_eq!(view.blocks[0].status, ConflictBlockStatus::Resolved(ConflictBlockResolution::BothTheirsFirst));
}

#[test]
fn insertion_in_empty_conflict_is_owned_and_can_be_replaced() {
    let mut view = view();
    view.blocks.truncate(1);
    view.draft = "head\nend\n".into();
    view.blocks[0].start = 5;
    view.blocks[0].end = 5;
    view.set_draft("head\n自定义\nend\n".into());
    let b = &view.blocks[0];
    assert_eq!(&view.draft[b.start..b.end], "自定义\n");
    assert!(b.has_manual_edits);
    view.apply_block_resolution(0, ConflictBlockResolution::Theirs);
    assert_eq!(view.draft, "head\ntheirs-first\nend\n");
}

#[test]
fn edit_across_blocks_keeps_ranges_disjoint_for_later_accept() {
    let mut view = view();
    view.set_draft("head\n新正文🦀\nend\n".into());
    assert!(view.blocks[0].end <= view.blocks[1].start);
    assert!(view.blocks.iter().all(|block| view.draft.is_char_boundary(block.start) && view.draft.is_char_boundary(block.end)));
    view.apply_block_resolution(1, ConflictBlockResolution::Theirs);
    let text = view.draft.clone();
    view.apply_block_resolution(0, ConflictBlockResolution::Ours);
    assert!(view.draft.contains("theirs-second\n"), "{text} => {}", view.draft);
}

#[test]
fn scroll_uses_corresponding_block_boundaries_after_length_change() {
    let anchors = [(0,0),(10,10),(12,18),(30,36)];
    assert_eq!(map_scroll_row(10.0,&anchors),10.0);
    assert_eq!(map_scroll_row(11.0,&anchors),14.0);
    assert_eq!(map_scroll_row(13.0,&anchors),19.0);
    assert_eq!(map_scroll_row(-10.0,&anchors),0.0);
}

#[test]
fn editing_before_a_block_moves_it_without_changing_its_resolution() {
    let mut view = view();
    view.apply_block_resolution(1, ConflictBlockResolution::Theirs);
    view.set_draft(format!("插入\n{}",view.draft));
    assert_eq!(view.blocks[1].status, ConflictBlockStatus::Resolved(ConflictBlockResolution::Theirs));
    assert_eq!(&view.draft[view.blocks[1].start..view.blocks[1].end],"theirs-second\n");
}

#[test]
fn typing_at_nonempty_block_boundary_is_preserved_when_accepting() {
    let mut view = view();
    view.set_draft("head\n新增注释\nfirst\nmiddle\nsecond\nend\n".into());
    assert!(!view.blocks[0].has_manual_edits);
    assert_eq!(&view.draft[view.blocks[0].start..view.blocks[0].end], "first\n");
    view.apply_block_resolution(0, ConflictBlockResolution::Ours);
    assert!(view.draft.contains("新增注释\nours-first\n"));
}
