use std::{cell::{Cell, RefCell}, ops::Range, rc::Rc};

use gpui::{App, AppContext as _, Context, Entity, EntityInputHandler, Focusable, HighlightStyle,
    IntoElement, Subscription, Window, canvas, div, point, prelude::*, px};
use gpui_kit::component::input::{Editor, EditorState, InputEvent, TextDecoration, TextDecorationCollection};
use sha2::{Digest, Sha256};

use super::{ConflictDocumentPane, conflict_byte_range_to_lines, conflict_document_byte_range,
    conflict_scroll_sync_source};
use crate::{ConflictSyntaxPane, MainMode, RepoTabId, RepositoryView, ui::theme as theme};
use crate::ui::theme::rgb;
use khaslana::{ConflictBlockStatus, ConflictDraftStatus, ConflictFileKind, ConflictFileView};

/// 宿主按仓库和文件保存，切换文件不重建编辑器或清空撤销历史。
/// 订阅还检查 entity 身份，失效宿主不能回写同路径的新冲突会话。
pub(crate) struct ConflictEditorHost {
    pub tab_id: RepoTabId,
    pub path: String,
    session_id: u64,
    pub states: [Entity<EditorState>; 3],
    pub(super) viewports: [Rc<RefCell<Option<gpui::Bounds<gpui::Pixels>>>>; 3],
    originals: [String; 2],
    syntax: [TextDecorationCollection; 3],
    revisions: Vec<ConflictRevision>,
    revision: usize,
    replay: Rc<Cell<i8>>,
    syntax_key: [usize; 3],
    dark: bool,
    scroll_sync_pending: Rc<Cell<bool>>,
    pub jump_pending: bool,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct ConflictRevision {
    digest: [u8; 32],
    blocks: Vec<(usize, usize, ConflictBlockStatus, bool)>,
    status: ConflictDraftStatus,
}

impl ConflictRevision {
    fn new(view: &ConflictFileView) -> Self {
        Self {
            digest: Sha256::digest(view.draft.as_bytes()).into(),
            blocks: view.blocks.iter().map(|b| (b.start, b.end, b.status, b.has_manual_edits)).collect(),
            status: view.draft_status,
        }
    }

    fn restore(&self, text: String, view: &mut ConflictFileView) {
        view.draft = text;
        // 撤销只还原编辑器，不能撤销已发生的工作区写入。
        view.draft_status = if self.status == ConflictDraftStatus::Applied { ConflictDraftStatus::Dirty } else { self.status };
        for (block, &(start, end, status, manual)) in view.blocks.iter_mut().zip(&self.blocks) {
            block.start = start;
            block.end = end;
            block.status = status;
            block.has_manual_edits = manual;
        }
    }
}

impl ConflictEditorHost {
    fn record(&mut self, view: &ConflictFileView) {
        let revision = ConflictRevision::new(view);
        if self.revisions.get(self.revision).is_some_and(|old| old.digest == revision.digest && old.blocks == revision.blocks && old.status == revision.status) {
            return;
        }
        self.revisions.truncate(self.revision + 1);
        self.revisions.push(revision);
        // 与 Kit 的有限撤销历史一样限制元数据保留，不长期积累大块列表。
        if self.revisions.len() > 256 { self.revisions.remove(0); }
        self.revision = self.revisions.len() - 1;
    }

    fn replay_revision(&mut self, text: &str, direction: i8) -> Option<ConflictRevision> {
        let digest: [u8; 32] = Sha256::digest(text.as_bytes()).into();
        let index = if direction < 0 {
            (0..self.revision).rev().find(|&i| self.revisions[i].digest == digest)
        } else {
            (self.revision + 1..self.revisions.len()).find(|&i| self.revisions[i].digest == digest)
        }?;
        self.revision = index;
        Some(self.revisions[index].clone())
    }
}

impl RepositoryView {
    pub(crate) fn selected_conflict_editor(&self) -> Option<&ConflictEditorHost> {
        let tab = self.active_tab_id()?;
        let path = self.conflict_workbench.selected_path.as_ref()?;
        self.conflict_editors.iter().find(|host| host.tab_id == tab && host.path == *path)
    }

    pub(crate) fn conflict_editor_focused(&self, window: &Window, cx: &App) -> bool {
        self.main_mode == MainMode::Conflict && self.selected_conflict_editor().is_some_and(|host|
            host.states.iter().any(|state| state.read(cx).focus_handle(cx).is_focused(window)))
    }

    pub(crate) fn ensure_conflict_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current_tab = self.active_tab_id();
        let current_session = self.conflict_workbench.session_id;
        let tabs: Vec<_> = self.tabs.iter().map(|tab| tab.id).collect();
        self.conflict_editors.retain(|host| tabs.contains(&host.tab_id) && (Some(host.tab_id) != current_tab || host.session_id == current_session));
        if self.main_mode != MainMode::Conflict {
            return;
        }
        self.ensure_conflict_views_loaded();
        let Some(tab_id) = self.active_tab_id() else { return };
        let Some(path) = self.conflict_workbench.selected_path.clone() else { return };
        let Some(view) = self.conflict_workbench.files.get(&path).filter(|v| v.kind == ConflictFileKind::Text).cloned() else { return };
        let invalid: Vec<_> = self.conflict_editors.iter().filter(|host| host.tab_id == tab_id &&
            !self.conflict_workbench.files.get(&host.path).is_some_and(|v| v.ours_text == host.originals[0] && v.theirs_text == host.originals[1]))
            .map(|host| host.path.clone()).collect();
        self.conflict_editors.retain(|host| host.tab_id != tab_id || !invalid.contains(&host.path));
        if self.selected_conflict_editor().is_none() {
            let texts = [&view.ours_text, &view.draft, &view.theirs_text];
            let states = std::array::from_fn(|index| cx.new(|cx| {
                let mut state = EditorState::new(window, cx).language("plaintext").line_number(true)
                    .folding(false).soft_wrap(false).default_value(texts[index].clone())
                    .scroll_beyond_last_line(None);
                state.set_readonly(index != 1, cx);
                state
            }));
            let syntax = std::array::from_fn(|index| states[index].update(cx, |state, cx|
                state.create_decorations_collection(Vec::new(), cx)));
            let mut subscriptions = Vec::new();
            for state in &states {
                subscriptions.push(cx.observe(state, |_, _, cx| cx.notify()));
            }
            let identity = states[1].entity_id();
            let event_path = path.clone();
            subscriptions.push(cx.subscribe_in(&states[1], window, move |this, state, event: &InputEvent, _, cx| {
                if !matches!(event, InputEvent::Change) || this.active_tab_id() != Some(tab_id)
                    || this.conflict_workbench.selected_path.as_ref() != Some(&event_path) {
                    return;
                }
                let Some(index) = this.conflict_editors.iter().position(|host|
                    host.tab_id == tab_id && host.path == event_path && host.states[1].entity_id() == identity) else { return };
                let text = state.read(cx).value().to_string();
                let direction = this.conflict_editors[index].replay.replace(0);
                let revision = (direction != 0).then(|| this.conflict_editors[index].replay_revision(&text, direction)).flatten();
                let Some(view) = this.conflict_workbench.files.get_mut(&event_path) else { return };
                if view.draft == text { return }
                if let Some(revision) = revision {
                    revision.restore(text, view);
                } else {
                    view.set_draft(text);
                    let snapshot = view.clone();
                    this.conflict_editors[index].record(&snapshot);
                }
                this.conflict_workbench.clear_pending_resolve();
                this.schedule_conflict_syntax_for_selected(&[ConflictSyntaxPane::Draft]);
                cx.notify();
            }));
            self.conflict_editor.focus = states[1].read(cx).focus_handle(cx);
            self.conflict_editors.push(ConflictEditorHost {
                tab_id, path: path.clone(), session_id: current_session, states, originals: [view.ours_text.clone(), view.theirs_text.clone()],
                viewports: std::array::from_fn(|_| Rc::new(RefCell::new(None))),
                syntax, revisions: vec![ConflictRevision::new(&view)], revision: 0,
                replay: Rc::new(Cell::new(0)), syntax_key: [0; 3], dark: theme::active_variant().is_dark(),
                jump_pending: true, _subscriptions: subscriptions,
                scroll_sync_pending: Rc::new(Cell::new(false)),
            });
        }
        let index = self.conflict_editors.iter().position(|host| host.tab_id == tab_id && host.path == path).unwrap();
        let result = self.conflict_editors[index].states[1].clone();
        self.conflict_editor.focus = result.read(cx).focus_handle(cx);
        let composing = result.update(cx, |state, cx| state.marked_text_range(window, cx).is_some());
        if !composing && result.read(cx).value().as_ref() != view.draft {
            self.conflict_editors[index].record(&view);
            // 替换差异片段而非 set_value，单次块接受/AI 回填保留 Kit 撤销历史。
            let old = result.read(cx).value().to_string();
            let (range, replacement) = text_replacement(&old, &view.draft);
            result.update(cx, |state, cx| {
                let offset = state.scroll_offset();
                state.set_selected_range(range, cx);
                state.replace(replacement, window, cx);
                state.set_scroll_offset(offset, cx);
            });
        }
        result.update(cx, |state, cx| state.set_readonly(self.busy, cx));
        let dark = theme::active_variant().is_dark();
        for (pane, kind) in [ConflictDocumentPane::Ours, ConflictDocumentPane::Result, ConflictDocumentPane::Theirs].into_iter().enumerate() {
            let spans = self.conflict_syntax_spans(kind, &view);
            let key = spans.as_ref().map_or(0, |value| std::sync::Arc::as_ptr(value) as usize);
            if self.conflict_editors[index].syntax_key[pane] != key || self.conflict_editors[index].dark != dark {
                let text = [&view.ours_text, &view.draft, &view.theirs_text][pane];
                let mut offset = 0;
                let decorations = text.split_inclusive('\n').enumerate().flat_map(|(line, text)| {
                    let start = offset;
                    offset += text.len();
                    spans.as_ref().and_then(|spans| spans.lines.get(line)).into_iter().flatten().map(move |span|
                        TextDecoration::new(start + span.start..start + span.end, HighlightStyle {
                            color: Some(rgb(span.color).into()), ..Default::default()
                        }))
                }).collect();
                self.conflict_editors[index].syntax[pane].set(decorations, cx);
                self.conflict_editors[index].syntax_key[pane] = key;
            }
        }
        self.conflict_editors[index].dark = dark;
        if self.conflict_editors[index].jump_pending && !composing {
            if let Some(block) = view.blocks.get(self.conflict_workbench.selected_block) {
                for (pane, kind) in [ConflictDocumentPane::Ours, ConflictDocumentPane::Result, ConflictDocumentPane::Theirs].into_iter().enumerate() {
                    let text = [&view.ours_text, &view.draft, &view.theirs_text][pane];
                    let (start, _) = conflict_document_byte_range(block, kind);
                    let row = text[..start.min(text.len())].bytes().filter(|b| *b == b'\n').count();
                    self.conflict_editors[index].states[pane].update(cx, |state, cx| {
                        let height = state.line_height().unwrap_or(px(21.0));
                        state.set_scroll_offset(point(px(0.0), -height * row.saturating_sub(3) as f32), cx);
                    });
                }
            }
            self.conflict_editors[index].jump_pending = false;
        }
    }

    pub(super) fn render_merge_editor(&self, pane: ConflictDocumentPane, view: &ConflictFileView, _cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(host) = self.selected_conflict_editor() else { return div().into_any_element() };
        let index = match pane { ConflictDocumentPane::Ours => 0, ConflictDocumentPane::Result => 1, ConflictDocumentPane::Theirs => 2 };
        let state = host.states[index].clone();
        let draw_state = state.clone();
        let blocks = view.blocks.clone();
        let text = [&view.ours_text, &view.draft, &view.theirs_text][index].clone();
        let selected = self.conflict_workbench.selected_block;
        let replay = host.replay.clone();
        let redo = replay.clone();
        let key_replay = replay.clone();
        let mouse_replay = replay.clone();
        let viewport = host.viewports[index].clone();
        div().relative().flex_1().min_w(px(0.0)).min_h(px(0.0)).overflow_hidden()
            .bg(rgb(theme::SURFACE_BASE))
            .capture_key_down(move |event,_,_| {
                if !event.keystroke.modifiers.control || !matches!(event.keystroke.key.as_str(), "z" | "y") {key_replay.set(0)}
            })
            .capture_any_mouse_down(move |_,_,_| mouse_replay.set(0))
            .capture_action(move |_: &gpui_kit::base::input::Undo, _, _| {if index==1 {replay.set(-1)}})
            .capture_action(move |_: &gpui_kit::base::input::Redo, _, _| {if index==1 {redo.set(1)}})
            .child(canvas(move |bounds, _, cx| {
                let changed = viewport.borrow().as_ref() != Some(&bounds);
                *viewport.borrow_mut() = Some(bounds);
                if changed { cx.refresh_windows(); }
            }, move |bounds, _, window, cx| {
                let state = draw_state.read(cx);
                let Some((origin, height)) = editor_row_origin(state) else { return };
                for (i, block) in blocks.iter().enumerate() {
                    let (start, end) = conflict_document_byte_range(block, pane);
                    let range = conflict_byte_range_to_lines(&text, start, end);
                    let top = (origin + height * range.start as f32).max(f32::from(bounds.top()));
                    let bottom = (origin + height * range.end as f32).min(f32::from(bounds.bottom()));
                    if bottom <= top { continue }
                    let color = match pane {
                        ConflictDocumentPane::Ours => theme::CONFLICT_OURS_BG,
                        ConflictDocumentPane::Theirs => theme::CONFLICT_THEIRS_BG,
                        ConflictDocumentPane::Result if block.status == ConflictBlockStatus::Unresolved => theme::FEEDBACK_WARNING_BG,
                        ConflictDocumentPane::Result => theme::FEEDBACK_SUCCESS_BG,
                    };
                    window.paint_quad(gpui::fill(gpui::Bounds::new(point(bounds.left(), px(top)), gpui::size(bounds.size.width, px(bottom-top))), rgb(color)));
                    if i == selected {
                        window.paint_quad(gpui::fill(gpui::Bounds::new(point(bounds.left(), px(top)), gpui::size(px(2.0), px(bottom-top))), rgb(theme::PRIMARY)));
                    }
                }
            }).absolute().size_full())
            .child(Editor::new(&state).appearance(false).bordered(false).readonly(index != 1 || self.busy)
                .aria_label(match pane { ConflictDocumentPane::Ours => "当前版本，只读", ConflictDocumentPane::Result => "合并结果，可编辑", ConflictDocumentPane::Theirs => "传入版本，只读" })
                .h(gpui::relative(1.0)).font_family("Consolas").text_size(px(14.0)))
            .into_any_element()
    }

    pub(super) fn render_editor_connectors(&self, view: &ConflictFileView, selected: usize) -> gpui::AnyElement {
        let Some(host) = self.selected_conflict_editor() else { return div().into_any_element() };
        let states = host.states.clone();
        let viewports = host.viewports.clone();
        let prev = self.conflict_pane_scroll_sync.clone();
        let sync_pending = host.scroll_sync_pending.clone();
        let enabled = !self.conflict_workbench.independent_scroll;
        let texts = [&view.ours_text, &view.draft, &view.theirs_text];
        let kinds = [ConflictDocumentPane::Ours, ConflictDocumentPane::Result, ConflictDocumentPane::Theirs];
        let ranges: Vec<[Range<usize>; 3]> = view.blocks.iter().map(|block|
            std::array::from_fn(|pane| {
                let (start,end) = conflict_document_byte_range(block, kinds[pane]);
                conflict_byte_range_to_lines(texts[pane], start, end)
            })).collect();
        let totals = texts.map(|text| text.bytes().filter(|b| *b == b'\n').count() + 1);
        canvas(move |_, _, cx| {
            let current = std::array::from_fn(|i| f32::from(states[i].read(cx).scroll_offset().y));
            let skip = sync_pending.replace(false);
            if enabled && !skip && let Some(previous) = *prev.borrow()
                && let Some(source) = conflict_scroll_sync_source(current, previous)
            {
                let height = f32::from(states[source].read(cx).line_height().unwrap_or(px(21.0)));
                let row = -current[source] / height;
                for target in 0..3 {
                    if target == source { continue }
                    let anchors = std::iter::once((0,0)).chain(ranges.iter().flat_map(|r|
                        [(r[source].start,r[target].start),(r[source].end,r[target].end)]))
                        .chain(std::iter::once((totals[source],totals[target]))).collect::<Vec<_>>();
                    let target_row = map_scroll_row(row, &anchors);
                    states[target].update(cx, |state,cx| {
                        let height = state.line_height().unwrap_or(px(21.0));
                        state.set_scroll_offset(point(state.scroll_offset().x, -height * target_row), cx);
                    });
                }
                // Kit 下一帧才钳制并应用偏移；该帧只记录最终偏移，不能把
                // 被联动的单栏误认成新的用户滚动源而产生来回抖动。
                sync_pending.set(true);
            }
            *prev.borrow_mut() = Some(current);
            (states, viewports, ranges)
        }, move |_, (states, viewports, ranges), window, cx| {
            let geometry: Vec<_> = states.iter().zip(&viewports).map(|(state, viewport)|
                viewport.borrow().as_ref().copied().zip(editor_row_origin(state.read(cx)))).collect();
            for (index, ranges) in ranges.iter().enumerate() {
                for (source, target) in [(0,1),(2,1)] {
                    let (Some((a,(ao,ah))),Some((b,(bo,bh)))) = (geometry[source], geometry[target]) else { continue };
                    let ay = ao + ah * ranges[source].start as f32;
                    let by = bo + bh * ranges[target].start as f32;
                    if ay > f32::from(a.bottom()) || by > f32::from(b.bottom()) { continue }
                    let ay2 = ao + ah * ranges[source].end as f32;
                    let by2 = bo + bh * ranges[target].end as f32;
                    if ay2 < f32::from(a.top()) || by2 < f32::from(b.top()) { continue }
                    let (x1,x2) = if source == 0 {(a.right(),b.left())} else {(a.left(),b.right())};
                    let color = if index == selected {theme::PRIMARY} else {theme::BORDER_MUTED};
                    let mut path = gpui::PathBuilder::stroke(px(1.0));
                    let mid = (x1+x2)/2.0;
                    let y1=px(((ay.max(f32::from(a.top()))+ay2.min(f32::from(a.bottom())))/2.0).max(f32::from(a.top())));
                    let y2=px(((by.max(f32::from(b.top()))+by2.min(f32::from(b.bottom())))/2.0).max(f32::from(b.top())));
                    path.move_to(point(x1,y1));
                    path.cubic_bezier_to(point(x2,y2),point(mid,y1),point(mid,y2));
                    if let Ok(path)=path.build(){window.paint_path(path,rgb(color));}
                }
            }
        }).absolute().size_full().into_any_element()
    }
}

/// Zed 用锚点保持滚动位置；这里使用冲突边界的分段映射，不能把相同 offset
/// 直接复制到行数不同的版本，否则接受多行块后其余两栏会逐渐漂移。
pub(super) fn map_scroll_row(row: f32, anchors: &[(usize,usize)]) -> f32 {
    let mut previous = (0,0);
    for &(source,target) in anchors {
        if source > previous.0 && row < source as f32 {
            let ratio = (row-previous.0 as f32).max(0.0)/(source-previous.0) as f32;
            return previous.1 as f32 + ratio * target.saturating_sub(previous.1) as f32;
        }
        previous = (source,target);
    }
    previous.1 as f32
}

pub(super) fn editor_row_origin(state: &EditorState) -> Option<(f32, f32)> {
    let row = state.visible_row_range()?.start;
    let text = state.value();
    let offset = if row == 0 { 0 } else { text.match_indices('\n').nth(row - 1)?.0 + 1 };
    let bounds = state.range_to_bounds(&(offset..offset))?;
    let height = f32::from(state.line_height()?);
    Some((f32::from(bounds.top()) - row as f32 * height, height))
}

/// 按字符边界取最小替换，避免替换全文导致全部冲突锚点或选区失效。
pub(super) fn text_replacement<'a>(old: &str, new: &'a str) -> (Range<usize>, &'a str) {
    let prefix = old.chars().zip(new.chars()).take_while(|(a,b)| a == b).map(|(c,_)| c.len_utf8()).sum::<usize>();
    let suffix = old[prefix..].chars().rev().zip(new[prefix..].chars().rev()).take_while(|(a,b)| a == b).map(|(c,_)| c.len_utf8()).sum::<usize>();
    (prefix..old.len()-suffix, &new[prefix..new.len()-suffix])
}

#[cfg(test)]
#[path = "../tests/conflict_editor.rs"]
mod tests;
