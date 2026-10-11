use super::*;
use khaslana::{ConflictBlock, ConflictFileKind};

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
fn candidate_rows_never_change_result_content_or_line_numbers() {
    let view=view(); let doc=Document::new(view.clone());
    assert_eq!(doc.view.draft,view.draft);
    let lines:Vec<_>=doc.rows.iter().filter_map(|r| match r {Row::Result {line,..}=>Some(*line),_=>None}).collect();
    assert_eq!(lines,vec![0,1,2,3,4,5]);
    assert_eq!(doc.rows.iter().filter(|r|matches!(r,Row::Header(_))).count(),2);
}
#[test]
fn choices_and_manual_edits_share_one_undo_transaction_history() {
    let mut doc=Document::new(view());let initial=doc.view.clone();
    doc.accept(0,ConflictBlockResolution::BothTheirsFirst);
    assert!(!doc.expanded.contains(&0));
    let selected=doc.view.clone();
    assert!(doc.view.draft.contains("theirs-first\nours-first\n"));
    doc.edit(0..0,"中文🦀\n",true);
    assert!(doc.undo(false));assert_eq!(doc.view.draft,selected.draft);assert_eq!(doc.view.blocks,selected.blocks);
    assert!(doc.undo(false));assert_eq!(doc.view.draft,initial.draft);assert_eq!(doc.view.blocks,initial.blocks);assert!(doc.expanded.contains(&0));
    assert!(doc.undo(true));assert_eq!(doc.view.draft,selected.draft);
}
#[test]
fn choosing_again_replaces_block_and_reset_restores_base() {
    let mut doc=Document::new(view());
    doc.accept(0,ConflictBlockResolution::Ours);doc.accept(0,ConflictBlockResolution::Theirs);
    assert!(doc.view.draft.contains("theirs-first\n"));assert!(!doc.view.draft.contains("ours-first\n"));
    doc.reset(0);assert_eq!(doc.view.draft,view().draft);assert!(doc.expanded.contains(&0));
    assert_eq!(doc.view.blocks[0].status,ConflictBlockStatus::Unresolved);
}
#[test]
fn line_adoption_is_idempotent_and_requires_confirmation() {
    let mut doc=Document::new(view());doc.adopt_line(0,ConflictResolutionSide::Theirs,0);
    let draft=doc.view.draft.clone();doc.adopt_line(0,ConflictResolutionSide::Theirs,0);
    assert_eq!(doc.view.draft,draft);assert!(doc.view.blocks[0].has_manual_edits);
    assert_eq!(doc.view.blocks[0].status,ConflictBlockStatus::Unresolved);
    assert_eq!(&doc.view.draft[doc.view.blocks[1].start..doc.view.blocks[1].end],"second\n");
}
#[test]
fn empty_and_same_anchor_conflicts_remain_actionable() {
    let mut v=view();v.draft="".into();for b in &mut v.blocks {b.start=0;b.end=0;}
    let doc=Document::new(v);assert_eq!(doc.rows.iter().filter(|r|matches!(r,Row::Header(_))).count(),2);
}
#[test]
fn word_diff_handles_unicode_and_separate_changes() {
    let text="let 名称 = 新值 + 新值;";
    let ranges=word_changes("let 名称 = 旧值 + 旧值;",text);
    assert!(!ranges.is_empty());assert!(ranges.iter().all(|r|text.is_char_boundary(r.start)&&text.is_char_boundary(r.end)));
    assert!(!ranges.iter().any(|r|text[r.clone()].contains("名称")));
}
#[test]
fn atomic_replacement_preserves_unicode_and_crlf() {
    let old="前文\r\n旧值🦀\r\n后文";let new="前文\r\n新值🙂🙂\r\n后文";
    let (range,replacement)=text_replacement(old,new);let mut result=old.to_owned();result.replace_range(range,replacement);assert_eq!(result,new);
}
