use gpui::{App, AppContext as _, Context, Entity, Focusable, IntoElement, Subscription, Window, div};
use super::{ConflictDocumentPane, native_editor::{Changed, NativeEditor}};
use crate::{ConflictSyntaxPane, MainMode, RepoTabId, RepositoryView};
use khaslana::ConflictFileKind;

/// 每个仓库、文件及冲突会话拥有独立缓冲区和撤销历史。
pub(crate) struct ConflictEditorHost {
    pub tab_id: RepoTabId,
    pub path: String,
    session_id: u64,
    pub(super) state: Entity<NativeEditor>,
    originals: [String; 2],
    pub jump_pending: bool,
    _subscriptions: Vec<Subscription>,
}

impl RepositoryView {
    pub(crate) fn selected_conflict_editor(&self) -> Option<&ConflictEditorHost> {
        let tab = self.active_tab_id()?;
        let path = self.conflict_workbench.selected_path.as_ref()?;
        self.conflict_editors.iter().find(|h| h.tab_id == tab && h.path == *path && h.session_id == self.conflict_workbench.session_id)
    }

    pub(crate) fn conflict_editor_focused(&self, window: &Window, cx: &App) -> bool {
        self.main_mode == MainMode::Conflict && self.selected_conflict_editor().is_some_and(|h| h.state.read(cx).focus_handle(cx).is_focused(window))
    }

    pub(crate) fn ensure_conflict_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current_tab = self.active_tab_id();
        let session = self.conflict_workbench.session_id;
        let tabs: Vec<_> = self.tabs.iter().map(|tab| tab.id).collect();
        self.conflict_editors.retain(|h| tabs.contains(&h.tab_id) && (Some(h.tab_id) != current_tab || h.session_id == session));
        if self.main_mode != MainMode::Conflict {return;}
        self.ensure_conflict_views_loaded();
        let Some(tab_id) = current_tab else {return;};
        let Some(path) = self.conflict_workbench.selected_path.clone() else {return;};
        let Some(view) = self.conflict_workbench.files.get(&path).filter(|v| v.kind == ConflictFileKind::Text).cloned() else {return;};
        self.conflict_editors.retain(|h| h.tab_id != tab_id || h.path != path || h.originals == [view.ours_text.clone(), view.theirs_text.clone()]);
        if self.selected_conflict_editor().is_none() {
            let state = cx.new(|cx| NativeEditor::new(view.clone(), cx));
            let identity = state.entity_id();
            let event_path = path.clone();
            let subscription = cx.subscribe_in(&state, window, move |this, state, _: &Changed, _, cx| {
                if this.active_tab_id() != Some(tab_id) || this.conflict_workbench.session_id != session
                    || this.conflict_workbench.selected_path.as_ref() != Some(&event_path)
                    || !this.conflict_editors.iter().any(|h| h.tab_id == tab_id && h.path == event_path && h.session_id == session && h.state.entity_id() == identity) {return;}
                if let Some(block)=state.read(cx).active_block {this.conflict_workbench.selected_block=block;cx.notify();}
                let Some(view) = this.conflict_workbench.files.get_mut(&event_path) else {return;};
                let changed = &state.read(cx).document.view;
                if view == changed {return;}
                *view = changed.clone();
                this.conflict_workbench.clear_pending_resolve();
                this.schedule_conflict_syntax_for_selected(&[ConflictSyntaxPane::Draft]);
                cx.notify();
            });
            self.conflict_editors.push(ConflictEditorHost {tab_id, path: path.clone(), session_id: session, state,
                originals: [view.ours_text.clone(), view.theirs_text.clone()], jump_pending: true, _subscriptions: vec![subscription]});
        }
        let index = self.conflict_editors.iter().position(|h| h.tab_id == tab_id && h.path == path).unwrap();
        let state = self.conflict_editors[index].state.clone();
        let syntax = [ConflictDocumentPane::Ours, ConflictDocumentPane::Result, ConflictDocumentPane::Theirs].map(|pane| self.conflict_syntax_spans(pane, &view));
        state.update(cx, |editor, cx| {editor.sync(&view, cx); editor.readonly = self.busy; editor.syntax = syntax;});
        self.conflict_editor.focus = state.read(cx).focus_handle(cx);
        if self.conflict_editors[index].jump_pending && state.read(cx).marked.is_none() {
            state.update(cx, |editor, cx| editor.jump(self.conflict_workbench.selected_block, cx));
            self.conflict_editors[index].jump_pending = false;
        }
    }

    pub(super) fn render_single_conflict_editor(&self) -> gpui::AnyElement {
        self.selected_conflict_editor().map_or_else(|| div().into_any_element(), |host| host.state.clone().into_any_element())
    }
}
