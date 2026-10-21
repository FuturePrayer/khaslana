//! 冲突工作台专用的单文档编辑组件。
//! UTF-16 输入桥接与 ShapedLine 绘制适配自 Zed GPUI examples/input.rs（Apache-2.0）。
//! 来源与许可见本目录 THIRD_PARTY.md；候选映射、合并事务和虚拟绘制为本项目实现。
#[path = "../ui_resources/conflicts_native_editor_messages.rs"]
mod messages;

use std::{ops::Range, sync::Arc};
use gpui::{prelude::*, App, Bounds, ClipboardItem, Context, ElementInputHandler, EntityInputHandler, EventEmitter, FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, ShapedLine, TextRun, UTF16Selection, Window, canvas, div, fill, point, px, size};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use khaslana::{ConflictBlockResolution as Resolution, ConflictBlockStatus, ConflictFileView, ConflictResolutionSide as Side};
use unicode_segmentation::UnicodeSegmentation;
use super::document::{Document, Row, line_ranges, word_changes};
use crate::ui::{components::workbench_command_button, theme::{self, rgb}};

const LINE: f32 = 24.0;
const GUTTER: f32 = 92.0;
const TRACK: f32 = 12.0;

pub(super) struct Changed;

struct PaintedLine {
    shaped: ShapedLine,
    origin: Point<Pixels>,
    result: Option<Range<usize>>,
    background: u32,
    height: f32,
    label: Option<String>,
    changed: Vec<Range<usize>>,
    accent: u32,
    offsets: Vec<(usize,usize)>,
}

struct Geometry {
    range: Range<usize>,
    line: ShapedLine,
    origin: Point<Pixels>,
    offsets: Vec<(usize,usize)>,
}

pub(super) struct NativeEditor {
    pub document: Document,
    focus: FocusHandle,
    pub readonly: bool,
    pub marked: Option<Range<usize>>,
    pub active_block: Option<usize>,
    pub syntax: [Option<Arc<khaslana::syntax::SyntaxSpans>>; 3],
    scroll: Point<Pixels>,
    viewport: Bounds<Pixels>,
    row_tops: Vec<f32>,
    content_width: f32,
    geometry: Vec<Geometry>,
    selecting: bool,
    dragging_bar: Option<(bool, f32, f32)>,
    reveal_caret: bool,
    pending_jump: Option<usize>,
    layout_generation: u64,
    layout_narrow: bool,
}

impl EventEmitter<Changed> for NativeEditor {}
impl Focusable for NativeEditor { fn focus_handle(&self, _: &App) -> FocusHandle { self.focus.clone() } }

impl NativeEditor {
    pub fn new(view: ConflictFileView, cx: &mut Context<Self>) -> Self {
        Self {document: Document::new(view), focus: cx.focus_handle(), readonly: false, marked: None, active_block: None,
            syntax: [None,None,None], scroll: point(px(0.0),px(0.0)), viewport: Bounds::default(), row_tops: Vec::new(),
            content_width: 0.0, geometry: Vec::new(), selecting: false, dragging_bar: None, reveal_caret: false, pending_jump: Some(0), layout_generation: u64::MAX, layout_narrow: false}
    }

    pub fn sync(&mut self, view: &ConflictFileView, cx: &mut Context<Self>) {
        if self.marked.is_some() || self.document.view == *view { return; }
        self.preserve_top(|doc| doc.sync(view));
        cx.notify();
    }

    pub fn jump(&mut self, block: usize, cx: &mut Context<Self>) { self.pending_jump=Some(block); self.active_block=Some(block); cx.notify(); }

    fn publish(&mut self, cx: &mut Context<Self>) {
        // 绘制从宿主的异步高亮缓存取颜色；编辑后先清掉旧结果，避免错位着色。
        self.syntax[1]=None;
        cx.emit(Changed);
        cx.notify();
    }

    pub fn history(&mut self, redo: bool, cx: &mut Context<Self>) {
        if self.readonly || self.marked.is_some() { return; }
        let mut changed=false;
        self.preserve_top(|doc| changed=doc.undo(redo));
        if changed { self.publish(cx); }
    }

    /// 以顶部结果行/块头作为锚点，候选折叠不把视口归零，也不触发导航。
    fn preserve_top(&mut self, change: impl FnOnce(&mut Document)) {
        let index=self.row_tops.partition_point(|&top| top<=f32::from(self.scroll.y)).saturating_sub(1);
        let top=self.document.rows.get(index).cloned();
        let relative=f32::from(self.scroll.y)-self.row_tops.get(index).copied().unwrap_or(0.0);
        change(&mut self.document);
        self.layout_rows();
        let match_index=top.and_then(|row| self.document.rows.iter().position(|candidate| match (&row,candidate) {
            (Row::Result {line:a,..},Row::Result {line:b,..})=>a==b,
            (Row::Header(a),Row::Header(b))=>a==b,
            _=>candidate==&row,
        }).or_else(|| {
            let block=match row {Row::Header(b)|Row::Label(b,_)|Row::Candidate {block:b,..}=>Some(b),_=>None}?;
            self.document.rows.iter().position(|r| *r==Row::Header(block))
        }));
        if let Some(i)=match_index {self.scroll.y=px((self.row_tops[i]+relative).max(0.0));}
        self.clamp_scroll();
    }

    fn layout_rows(&mut self) {
        let narrow=f32::from(self.viewport.size.width)<760.0;
        if self.layout_generation==self.document.generation && self.layout_narrow==narrow {return;}
        self.layout_generation=self.document.generation;
        self.layout_narrow=narrow;
        self.row_tops.clear();
        let mut y=0.0;
        for row in &self.document.rows {
            self.row_tops.push(y);
            y+=if matches!(row,Row::Header(_)) {if narrow {64.0} else {32.0}} else {LINE};
        }
        self.row_tops.push(y);
        // 宽度估算只负责滚动范围；实际命中与光标均使用字体整形结果。
        self.content_width=self.document.view.draft.lines().map(|l| l.chars().map(|c| if c=='\t' {4} else if c.is_ascii() {1} else {2}).sum::<usize>()).max().unwrap_or(0) as f32*8.5+GUTTER+24.0;
        for b in &self.document.view.blocks {
            self.content_width=self.content_width.max(b.ours.lines().chain(b.theirs.lines()).map(|l| l.chars().count() as f32*13.0+GUTTER+24.0).fold(0.0,f32::max));
        }
    }

    fn max_scroll(&self) -> Point<Pixels> {
        point(px((self.content_width-f32::from(self.viewport.size.width)+TRACK).max(0.0)),
            px((self.row_tops.last().copied().unwrap_or(0.0)-f32::from(self.viewport.size.height)+TRACK).max(0.0)))
    }

    fn clamp_scroll(&mut self) {
        let max=self.max_scroll();
        self.scroll.x=self.scroll.x.clamp(px(0.0),max.x);
        self.scroll.y=self.scroll.y.clamp(px(0.0),max.y);
    }

    fn result_point(&self, position: Point<Pixels>, nearest: bool) -> Option<usize> {
        let item=self.geometry.iter().filter(|g| nearest || position.y>=g.origin.y && position.y<g.origin.y+px(LINE))
            .min_by(|a,b| (f32::from(position.y-a.origin.y)-LINE/2.0).abs().total_cmp(&(f32::from(position.y-b.origin.y)-LINE/2.0).abs()))?;
        let x=position.x-item.origin.x;
        let display=item.line.closest_index_for_x(x);
        let local=item.offsets.iter().rev().find(|(_,d)|*d<=display).map_or(0,|(s,_)|*s);
        Some(item.range.start+local.min(item.range.len()))
    }

    fn click(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.button!=MouseButton::Left {return;}
        if event.position.x>=self.viewport.right()-px(TRACK) {
            self.start_bar(true,event.position.y,cx); return;
        }
        if event.position.y>=self.viewport.bottom()-px(TRACK) {
            self.start_bar(false,event.position.x,cx); return;
        }
        if let Some(offset)=self.result_point(event.position,false) {
            window.focus(&self.focus,cx);
            self.marked=None;
            if event.click_count>=3 {
                let start=self.document.view.draft[..offset].rfind('\n').map_or(0,|p|p+1);
                let end=self.document.view.draft[offset..].find('\n').map_or(self.document.view.draft.len(),|p|offset+p+1);
                self.document.anchor=start;self.document.caret=end;
            } else if event.click_count==2 {
                let text=&self.document.view.draft;
                let range=text.split_word_bound_indices().find(|(i,s)| *i<=offset && offset<i+s.len()).map(|(i,s)| i..i+s.len()).unwrap_or(offset..offset);
                self.document.anchor=range.start;self.document.caret=range.end;
            } else {
                if !event.modifiers.shift {self.document.anchor=offset;}
                self.document.caret=offset;
            }
            self.active_block=self.document.view.blocks.iter().position(|b|b.start<=offset && offset<=b.end);
            cx.emit(Changed);
            self.selecting=true;
            cx.notify();
        }
    }

    fn bar_metrics(&self, vertical: bool) -> (f32,f32,f32) {
        let extent=if vertical {f32::from(self.viewport.size.height)} else {f32::from(self.viewport.size.width)}-TRACK;
        let max=if vertical {f32::from(self.max_scroll().y)} else {f32::from(self.max_scroll().x)};
        let thumb=(extent*extent/(extent+max).max(1.0)).max(24.0).min(extent.max(0.0));
        (extent,thumb,max)
    }

    fn start_bar(&mut self, vertical: bool, coordinate: Pixels, cx: &mut Context<Self>) {
        let (extent,thumb,max)=self.bar_metrics(vertical);
        if max<=0.0 {return;}
        let origin=if vertical {self.viewport.top()} else {self.viewport.left()};
        let value=if vertical {self.scroll.y} else {self.scroll.x};
        let local=f32::from(coordinate-origin);
        let start=f32::from(value)/max*(extent-thumb);
        if local<start || local>start+thumb {
            let offset=px(((local-thumb/2.0)/(extent-thumb).max(1.0)*max).clamp(0.0,max));
            if vertical {self.scroll.y=offset;} else {self.scroll.x=offset;}
        }
        self.dragging_bar=Some((vertical,f32::from(coordinate),if vertical {f32::from(self.scroll.y)} else {f32::from(self.scroll.x)}));
        cx.notify();
    }

    fn drag(&mut self,event: &MouseMoveEvent,_: &mut Window,cx: &mut Context<Self>) {
        if let Some((vertical,start,offset))=self.dragging_bar {
            let (extent,thumb,max)=self.bar_metrics(vertical);
            let coordinate=if vertical {event.position.y} else {event.position.x};
            let value=px((offset+(f32::from(coordinate)-start)/(extent-thumb).max(1.0)*max).clamp(0.0,max));
            if vertical {self.scroll.y=value;} else {self.scroll.x=value;}
            cx.notify();
        } else if self.selecting {
            if event.position.y<self.viewport.top() {self.scroll.y-=px(LINE);}
            if event.position.y>self.viewport.bottom()-px(TRACK) {self.scroll.y+=px(LINE);}
            self.clamp_scroll();
            if let Some(offset)=self.result_point(event.position,true) {self.document.caret=offset;cx.notify();}
        }
    }

    fn move_caret(&mut self, offset: usize, extend: bool, cx: &mut Context<Self>) {
        self.document.caret=offset;
        if !extend {self.document.anchor=offset;}
        self.reveal_caret=true;
        cx.notify();
    }

    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.marked.is_some() {return;}
        let key=event.keystroke.key.as_str();
        let control=event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        if !self.focus.is_focused(window) && !(control && matches!(key,"z"|"y")) {return;}
        let shift=event.keystroke.modifiers.shift;
        let text=&self.document.view.draft;
        let pos=self.document.caret;
        let selection=self.document.selection();
        if control && matches!(key,"z"|"y") {self.history(key=="y" || shift,cx);}
        else if control && key=="a" {self.document.anchor=0;self.document.caret=text.len();cx.notify();}
        else if control && matches!(key,"c"|"x") {
            if !selection.is_empty() {cx.write_to_clipboard(ClipboardItem::new_string(text[selection.clone()].to_string()));}
            if key=="x" && !self.readonly {self.document.edit(selection,"",true);self.publish(cx);}
        } else if control && key=="v" {
            if !self.readonly && let Some(text)=cx.read_from_clipboard().and_then(|c|c.text()) {self.replace_text_in_range(None,&text,window,cx);}
        } else if matches!(key,"left"|"right") {
            let next=if !shift && !selection.is_empty() {if key=="left" {selection.start} else {selection.end}}
                else if control {
                    if key=="left" {text[..pos].unicode_word_indices().last().map_or(0,|(i,_)|i)}
                    else {text[pos..].unicode_word_indices().next().map_or(text.len(),|(i,s)|pos+i+s.len())}
                } else if key=="left" {text[..pos].grapheme_indices(true).last().map_or(0,|(i,_)|i)}
                else {text[pos..].graphemes(true).next().map_or(text.len(),|s|pos+s.len())};
            self.move_caret(next,shift,cx);
        } else if matches!(key,"up"|"down"|"pageup"|"pagedown"|"home"|"end") {
            let lines=line_ranges(text);
            let current=lines.partition_point(|r| r.start<=pos).saturating_sub(1);
            let count=if key.starts_with("page") {(f32::from(self.viewport.size.height)/LINE).max(1.0) as usize} else {1};
            let next=match key {
                "home"=>if control {0} else {lines[current].start},
                "end"=>if control {text.len()} else {lines[current].start+text[lines[current].clone()].trim_end_matches(['\r','\n']).len()},
                _=> {
                    let target=if key=="up" || key=="pageup" {current.saturating_sub(count)} else {(current+count).min(lines.len()-1)};
                    let column=text[lines[current].start..pos].graphemes(true).count();
                    let content=text[lines[target].clone()].trim_end_matches(['\r','\n']);
                    lines[target].start+content.grapheme_indices(true).nth(column).map_or(content.len(),|(i,_)|i)
                }
            };
            self.move_caret(next,shift,cx);
        } else if !self.readonly && matches!(key,"backspace"|"delete"|"enter"|"tab") && !control {
            let mut range=selection;
            let replacement=match key {"enter"=>if text.contains("\r\n") {"\r\n"} else {"\n"},"tab"=>"    ",_=>""};
            if range.is_empty() && key=="backspace" {range.start=text[..pos].grapheme_indices(true).last().map_or(0,|(i,_)|i);}
            if range.is_empty() && key=="delete" {range.end=text[pos..].graphemes(true).next().map_or(text.len(),|s|pos+s.len());}
            self.document.edit(range,replacement,true);self.reveal_caret=true;self.publish(cx);
        } else {return;}
        cx.stop_propagation();
    }

    fn header(&self, block: usize, narrow: bool, cx: &Context<Self>) -> gpui::AnyElement {
        let b=&self.document.view.blocks[block];
        let expanded=self.document.expanded.contains(&block);
        let pending=b.status==ConflictBlockStatus::Unresolved;
        let status=if b.has_manual_edits {khaslana::ui_i18n::text(messages::MANUALLY_ADJUSTED_CONFIRMATION_NEEDED)} else if pending {khaslana::ui_i18n::text(messages::PENDING_NOT_ACCEPTED)} else {khaslana::ui_i18n::text(messages::RESOLVED)};
        let mut actions=div().flex().items_center().gap_1();
        if expanded {
            for (label,resolution) in [(khaslana::ui_i18n::text(messages::KEEP_CURRENT),Resolution::Ours),(khaslana::ui_i18n::text(messages::KEEP_INCOMING),Resolution::Theirs)] {
                actions=actions.child(workbench_command_button(format!("keep-{block}-{resolution:?}"),label,false,!self.readonly,cx).debug_selector(move || format!("keep-{block}-{resolution:?}")).h(px(26.0)).px_2().text_size(px(11.0))
                    .on_click(cx.listener(move |this,_,_,cx| {if this.readonly || this.marked.is_some() {return;} this.active_block=Some(block);this.preserve_top(|doc|doc.accept(block,resolution));this.publish(cx);})));
            }
            let entity=cx.entity().downgrade();
            actions=actions.child(workbench_command_button(format!("both-{block}"),khaslana::ui_i18n::text(messages::KEEP_BOTH),false,!self.readonly,cx).h(px(26.0)).px_2().text_size(px(11.0)).dropdown_caret(true)
                .dropdown_menu(move |menu,_,_| {
                    let a=entity.clone();let b=entity.clone();
                    menu.item(PopupMenuItem::new(khaslana::ui_i18n::text(messages::CURRENT_FIRST_INCOMING_SECOND)).on_click(move |_,_,cx| {let _=a.update(cx,|this,cx| {if !this.readonly && this.marked.is_none() {this.active_block=Some(block);this.preserve_top(|d|d.accept(block,Resolution::BothOursFirst));this.publish(cx);}});}))
                        .item(PopupMenuItem::new(khaslana::ui_i18n::text(messages::INCOMING_FIRST_CURRENT_SECOND)).on_click(move |_,_,cx| {let _=b.update(cx,|this,cx| {if !this.readonly && this.marked.is_none() {this.active_block=Some(block);this.preserve_top(|d|d.accept(block,Resolution::BothTheirsFirst));this.publish(cx);}});}))
                }));
            actions=actions.child(workbench_command_button(format!("base-{block}"),khaslana::ui_i18n::text(messages::RESET_TO_BASE),false,!self.readonly && b.base.is_some(),cx).h(px(26.0)).px_2().text_size(px(11.0)).tooltip(if b.base.is_some() {khaslana::ui_i18n::text(messages::RESTORE_THIS_BLOCK_S_COMMON_ANCESTOR_AND_RESOLVE_AGAIN)} else {khaslana::ui_i18n::text(messages::THIS_BLOCK_HAS_NO_COMMON_ANCESTOR)})
                .on_click(cx.listener(move |this,_,_,cx| {if !this.readonly && this.marked.is_none() {this.active_block=Some(block);this.preserve_top(|d|d.reset(block));this.publish(cx);}})));
            if b.has_manual_edits {
                actions=actions.child(workbench_command_button(format!("confirm-{block}"),khaslana::ui_i18n::text(messages::CONFIRM_RESULT),false,!self.readonly,cx).h(px(26.0)).px_2().text_size(px(11.0))
                    .on_click(cx.listener(move |this,_,_,cx| {if this.readonly || this.marked.is_some() {return;}this.preserve_top(|d| {d.view.blocks[block].status=ConflictBlockStatus::Merged;d.expanded.remove(&block);d.rebuild();d.record();});this.publish(cx);})));
            }
        } else {
            actions=actions.child(workbench_command_button(format!("expand-{block}"),khaslana::ui_i18n::text(messages::EXPAND_CANDIDATES_CHOOSE_AGAIN),false,true,cx).h(px(26.0)).px_2().text_size(px(11.0))
                .on_click(cx.listener(move |this,_,_,cx| {this.preserve_top(|d|{d.expanded.insert(block);d.rebuild();});cx.notify();})));
        }
        div().flex().when(narrow,|d|d.flex_col().items_start()).when(!narrow,|d|d.items_center()).justify_between().size_full().px_2().gap_1()
            .bg(rgb(if pending {theme::CONFLICT_PENDING_BG} else {theme::CONFLICT_RESOLVED_BG}))
            .child(div().text_size(px(11.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child(khaslana::ui_i18n::format(messages::CONFLICT_BLOCK_STATUS, &[("block", &(block+1).to_string()), ("status", &(status).to_string())])))
            .child(actions).into_any_element()
    }

    fn prepare(&mut self,bounds: Bounds<Pixels>,window: &mut Window,cx: &mut Context<Self>) -> (Vec<PaintedLine>,Vec<gpui::AnyElement>) {
        self.viewport=bounds;
        self.layout_rows();
        if let Some(block)=self.pending_jump.take() && let Some(i)=self.document.rows.iter().position(|r| *r==Row::Header(block)) {self.scroll.y=px((self.row_tops[i]-LINE).max(0.0));}
        if self.reveal_caret {
            if let Some(i)=self.document.rows.iter().position(|r| matches!(r,Row::Result {range,..} if range.start<=self.document.caret && (self.document.caret<range.end || range.end==self.document.view.draft.len()))) {
                let y=self.row_tops[i];
                if y<f32::from(self.scroll.y) {self.scroll.y=px(y);}
                if y+LINE>f32::from(self.scroll.y+bounds.size.height)-TRACK {self.scroll.y=px(y+LINE-f32::from(bounds.size.height)+TRACK);}
                if let Row::Result {range,..}=&self.document.rows[i] {
                    let text=self.document.view.draft[range.clone()].trim_end_matches(['\r','\n']);
                    let (display,offsets)=expand_tabs(text);
                    let run=TextRun {len:display.len(),font:window.text_style().font(),color:rgb(theme::CONTENT_PRIMARY).into(),background_color:None,underline:None,strikethrough:None};
                    let shaped=window.text_system().shape_line(display.into(),px(13.0),&[run],None);
                    let x=shaped.x_for_index(display_offset(&offsets,self.document.caret.saturating_sub(range.start).min(text.len())));
                    let available=(bounds.size.width-px(GUTTER+TRACK+12.0)).max(px(1.0));
                    if x<self.scroll.x {self.scroll.x=x;}
                    if x>self.scroll.x+available {self.scroll.x=x-available;}
                    self.content_width=self.content_width.max(f32::from(x)+GUTTER+24.0);
                }
            }
            self.reveal_caret=false;
        }
        self.clamp_scroll();
        let start=self.row_tops.partition_point(|&top|top<f32::from(self.scroll.y)).saturating_sub(1);
        let end=self.row_tops.partition_point(|&top|top<f32::from(self.scroll.y+bounds.size.height)).min(self.document.rows.len());
        let mut painted=Vec::new();let mut controls=Vec::new();
        let style=window.text_style();
        let mask=Bounds::new(bounds.origin,size((bounds.size.width-px(TRACK)).max(px(0.0)),(bounds.size.height-px(TRACK)).max(px(0.0))));
        window.with_content_mask(Some(gpui::ContentMask {bounds:mask}), |window| {
        for i in start..end {
            let row=&self.document.rows[i];
            let y=bounds.top()+px(self.row_tops[i])-self.scroll.y;
            if let Row::Header(block)=row {
                let mut element=self.header(*block,f32::from(bounds.size.width)<760.0,cx);
                element.prepaint_as_root(point(bounds.left(),y),size((bounds.size.width-px(TRACK)).max(px(0.0)),px(self.row_tops[i+1]-self.row_tops[i])).into(),window,cx);
                controls.push(element);continue;
            }
            let (text,result,label,bg,accent,changes,syntax)=match row {
                Row::Result {line,range,owner} => {
                    let bg=owner.map_or(theme::CONFLICT_EDITOR_BG,|i| if self.document.view.blocks[i].status==ConflictBlockStatus::Unresolved {theme::CONFLICT_PENDING_BG} else {theme::CONFLICT_RESOLVED_BG});
                    (&self.document.view.draft[range.clone()],Some(range.clone()),Some((line+1).to_string()),bg,theme::CONTENT_PRIMARY,Vec::new(),self.syntax[1].as_ref().and_then(|s|s.lines.get(*line)))
                },
                Row::Candidate {block,side,line,range} => {
                    let b=&self.document.view.blocks[*block];let ours=*side==Side::Ours;
                    let text=if ours {&b.ours[range.clone()]} else {&b.theirs[range.clone()]};
                    let base=b.base.as_deref().unwrap_or("").split_inclusive('\n').nth(*line).unwrap_or("");
                    let source=if ours {&self.document.view.ours_text} else {&self.document.view.theirs_text};
                    let source_start=if ours {b.ours_start} else {b.theirs_start};
                    let source_line=source[..source_start.min(source.len())].bytes().filter(|b|*b==b'\n').count()+line;
                    let index=if ours {0} else {2};
                    let (block,side,line)=(*block,*side,*line);
                    let mut button=workbench_command_button(format!("line-{block}-{index}-{line}"),"+",false,!self.readonly,cx).h(px(20.0)).min_h(px(20.0)).w(px(22.0)).px_0().text_size(px(12.0))
                        .accessibility_label(khaslana::ui_i18n::format(if ours { messages::ACCEPT_CURRENT_LINE } else { messages::ACCEPT_INCOMING_LINE }, &[("line", &(line+1).to_string())])).tooltip(khaslana::ui_i18n::text(messages::ACCEPT_THIS_LINE_INTO_THE_CORRESPONDING_RESULT_LINE_THE_ENTIRE))
                        .on_click(cx.listener(move |this,_,_,cx| {if !this.readonly && this.marked.is_none() {this.active_block=Some(block);this.preserve_top(|d|d.adopt_line(block,side,line));this.publish(cx);}})).into_any_element();
                    button.prepaint_as_root(point(bounds.left()+px(64.0),y+px(2.0)),size(px(22.0),px(20.0)).into(),window,cx);controls.push(button);
                    (text,None,Some(format!("◀ {}{}",if ours {"O"} else {"T"},line+1)),if ours {theme::CONFLICT_OURS_BG} else {theme::CONFLICT_THEIRS_BG},if ours {theme::CONFLICT_OURS_ACCENT} else {theme::CONFLICT_THEIRS_ACCENT},word_changes(base.trim_end_matches(['\r','\n']),text.trim_end_matches(['\r','\n'])),self.syntax[index].as_ref().and_then(|s|s.lines.get(source_line)))
                },
                Row::Label(_,side) => (match side {Some(Side::Ours)=>khaslana::ui_i18n::text(messages::CURRENT_VERSION_READ_ONLY_CANDIDATE),Some(Side::Theirs)=>khaslana::ui_i18n::text(messages::INCOMING_VERSION_READ_ONLY_CANDIDATE),None=>khaslana::ui_i18n::text(messages::CURRENT_RESULT_DIRECTLY_EDITABLE)},None,None,
                    match side {Some(Side::Ours)=>theme::CONFLICT_OURS_BG,Some(Side::Theirs)=>theme::CONFLICT_THEIRS_BG,None=>theme::CONFLICT_PENDING_BG},theme::CONTENT_SECONDARY,Vec::new(),None),
                Row::Header(_)=>unreachable!(),
            };
            let text=text.trim_end_matches(['\r','\n']);
            let mut runs=Vec::new();let mut offset=0;
            let base_run=TextRun {len:0,font:style.font(),color:rgb(theme::CONTENT_PRIMARY).into(),background_color:None,underline:None,strikethrough:None};
            if let Some(spans)=syntax {for span in spans {
                let a=span.start.min(text.len());let b=span.end.min(text.len());
                if a<offset || !text.is_char_boundary(a) || !text.is_char_boundary(b) {continue;}
                if a>offset {runs.push(TextRun {len:a-offset,..base_run.clone()});}
                if b>a {runs.push(TextRun {len:b-a,color:gpui::rgb(span.color).into(),..base_run.clone()});}
                offset=b;
            }}
            if offset<text.len() {runs.push(TextRun {len:text.len()-offset,..base_run.clone()});}
            let (display,offsets)=expand_tabs(text);
            let mut source_offset=0;
            for run in &mut runs {
                let end=source_offset+run.len;
                run.len=display_offset(&offsets,end)-display_offset(&offsets,source_offset);
                source_offset=end;
            }
            let shaped=window.text_system().shape_line(display.into(),px(13.0),&runs,None);
            let result=result.map(|r|r.start..r.start+text.len());
            painted.push(PaintedLine {shaped,origin:point(bounds.left()+px(GUTTER)-self.scroll.x,y),result,background:bg,height:LINE,label,changed:changes,accent,offsets});
        }
        });
        (painted,controls)
    }

    fn paint(&mut self, lines: Vec<PaintedLine>,mut controls: Vec<gpui::AnyElement>,window: &mut Window,cx: &mut Context<Self>) {
        let bounds=self.viewport;
        let content=Bounds::new(bounds.origin,size((bounds.size.width-px(TRACK)).max(px(0.0)),(bounds.size.height-px(TRACK)).max(px(0.0))));
        window.handle_input(&self.focus,ElementInputHandler::new(content,cx.entity()),cx);
        let selected=self.document.selection();
        self.geometry.clear();
        window.with_content_mask(Some(gpui::ContentMask {bounds:content}),|window| {
            for row in lines {
                window.paint_quad(fill(Bounds::new(point(bounds.left(),row.origin.y),size(content.size.width,px(row.height))),rgb(row.background)));
                let code_mask=Bounds::new(point(bounds.left()+px(GUTTER),bounds.top()),size((content.size.width-px(GUTTER)).max(px(0.0)),content.size.height));
                window.with_content_mask(Some(gpui::ContentMask {bounds:code_mask}),|window| {
                    let x_for=|offset|row.shaped.x_for_index(display_offset(&row.offsets,offset));
                    for range in &row.changed {
                        window.paint_quad(fill(Bounds::new(point(row.origin.x+x_for(range.start),row.origin.y),size(x_for(range.end)-x_for(range.start),px(LINE))),rgb(row.accent).opacity(0.18)));
                    }
                    if let Some(range)=&row.result {
                        let start=selected.start.max(range.start);let end=selected.end.min(range.end);
                        if start<end || selected.start<=range.end && selected.end>range.end {
                            let a=x_for(start.saturating_sub(range.start).min(range.len()));
                            let b=if selected.end>range.end {row.shaped.width+px(8.0)} else {x_for(end.saturating_sub(range.start))};
                            window.paint_quad(fill(Bounds::new(point(row.origin.x+a,row.origin.y),size((b-a).max(px(2.0)),px(LINE))),rgb(theme::PRIMARY).opacity(0.20)));
                        }
                        if self.focus.is_focused(window) && self.document.caret>=range.start && self.document.caret<=range.end {
                            let x=row.origin.x+x_for(self.document.caret-range.start);
                            window.paint_quad(fill(Bounds::new(point(x,row.origin.y+px(3.0)),size(px(1.5),px(LINE-6.0))),rgb(theme::CONTENT_PRIMARY)));
                        }
                        if let Some(marked)=&self.marked {
                            let a=marked.start.max(range.start);let b=marked.end.min(range.end);
                            if a<b {window.paint_quad(fill(Bounds::new(point(row.origin.x+x_for(a-range.start),row.origin.y+px(LINE-3.0)),size(x_for(b-range.start)-x_for(a-range.start),px(1.0))),rgb(theme::PRIMARY)));}
                        }
                    }
                    let _=row.shaped.paint(row.origin,px(LINE),gpui::TextAlign::Left,None,window,cx);
                });
                if let Some(label)=row.label {
                    let run=TextRun {len:label.len(),font:window.text_style().font(),color:rgb(row.accent).into(),background_color:None,underline:None,strikethrough:None};
                    let label=window.text_system().shape_line(label.into(),px(11.0),&[run],None);
                    let _=label.paint(point(bounds.left()+px(8.0),row.origin.y),px(LINE),gpui::TextAlign::Left,None,window,cx);
                }
                if let Some(range)=row.result {self.geometry.push(Geometry {range,line:row.shaped,origin:row.origin,offsets:row.offsets});}
            }
            for control in &mut controls {control.paint(window,cx);}
        });
        // 滚动条位于独立轨道，最后绘制；块头与候选永远不能覆盖轨道或其命中区。
        for vertical in [true,false] {
            let (extent,thumb,max)=self.bar_metrics(vertical);
            if max<=0.0 {continue;}
            let offset=if vertical {f32::from(self.scroll.y)} else {f32::from(self.scroll.x)};
            let start=offset/max*(extent-thumb);
            let track=if vertical {Bounds::new(point(bounds.right()-px(TRACK),bounds.top()),size(px(TRACK),px(extent)))} else {Bounds::new(point(bounds.left(),bounds.bottom()-px(TRACK)),size(px(extent),px(TRACK)))};
            window.paint_quad(fill(track,rgb(theme::CONFLICT_GUTTER_BG)));
            let rect=if vertical {Bounds::new(point(track.left()+px(3.0),track.top()+px(start)),size(px(6.0),px(thumb)))} else {Bounds::new(point(track.left()+px(start),track.top()+px(3.0)),size(px(thumb),px(6.0)))};
            window.paint_quad(fill(rect,rgb(theme::CONTENT_TERTIARY).opacity(0.6)));
        }
    }
}

impl Render for NativeEditor {
    fn render(&mut self,_: &mut Window,cx: &mut Context<Self>) -> impl IntoElement {
        let entity=cx.entity();let painter=entity.clone();
        div().id("conflict-single-document").role(gpui::Role::MultilineTextInput).aria_label(khaslana::ui_i18n::text(messages::EDITOR_ACCESSIBILITY)).flex_1().size_full().min_w(px(0.0)).min_h(px(0.0)).overflow_hidden()
            .key_context("ConflictEditor").track_focus(&self.focus).font_family("Consolas").text_size(px(13.0)).bg(rgb(theme::CONFLICT_EDITOR_BG))
            .on_key_down(cx.listener(Self::key)).on_mouse_down(MouseButton::Left,cx.listener(Self::click))
            .on_mouse_move(cx.listener(Self::drag)).on_mouse_up(MouseButton::Left,cx.listener(|this,_,_,_| {this.selecting=false;this.dragging_bar=None;}))
            .on_mouse_up_out(MouseButton::Left,cx.listener(|this,_,_,_| {this.selecting=false;this.dragging_bar=None;}))
            .on_scroll_wheel(cx.listener(|this,event: &gpui::ScrollWheelEvent,_,cx| {let delta=event.delta.pixel_delta(px(LINE));this.scroll.x-=delta.x;this.scroll.y-=delta.y;this.clamp_scroll();cx.stop_propagation();cx.notify();}))
            .child(canvas(move |bounds,window,cx| entity.update(cx,|this,cx| this.prepare(bounds,window,cx)),
                move |_,(lines,controls),window,cx| painter.update(cx,|this,cx|this.paint(lines,controls,window,cx))).size_full())
    }
}

pub(super) fn from_utf16(text: &str, offset: usize) -> usize {
    let mut count=0;
    for (i,c) in text.char_indices() {if count>=offset {return i;} count+=c.len_utf16();}
    text.len()
}
fn to_utf16(text: &str, offset: usize) -> usize {text[..offset].encode_utf16().count()}

/// Tab 仅扩展显示，选区、撤销和保存仍使用原始字节坐标。
fn expand_tabs(text: &str) -> (String,Vec<(usize,usize)>) {
    let mut display=String::new();let mut offsets=Vec::new();let mut column=0;
    for (i,c) in text.char_indices() {
        offsets.push((i,display.len()));
        if c=='\t' {let count=4-column%4;display.extend(std::iter::repeat_n(' ',count));column+=count;}
        else {display.push(c);column+=1;}
    }
    offsets.push((text.len(),display.len()));(display,offsets)
}
fn display_offset(offsets: &[(usize,usize)], source: usize) -> usize {
    offsets.binary_search_by_key(&source,|(s,_)|*s).map(|i|offsets[i].1).unwrap_or_else(|i|offsets[i.saturating_sub(1)].1)
}

impl EntityInputHandler for NativeEditor {
    fn text_for_range(&mut self,range: Range<usize>,actual: &mut Option<Range<usize>>,_: &mut Window,_: &mut Context<Self>) -> Option<String> {
        let text=&self.document.view.draft;let range=from_utf16(text,range.start)..from_utf16(text,range.end);
        *actual=Some(to_utf16(text,range.start)..to_utf16(text,range.end));Some(text[range].into())
    }
    fn selected_text_range(&mut self,_: bool,_: &mut Window,_: &mut Context<Self>) -> Option<UTF16Selection> {
        let text=&self.document.view.draft;let r=self.document.selection();
        Some(UTF16Selection {range:to_utf16(text,r.start)..to_utf16(text,r.end),reversed:self.document.caret<self.document.anchor})
    }
    fn marked_text_range(&self,_: &mut Window,_: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r|to_utf16(&self.document.view.draft,r.start)..to_utf16(&self.document.view.draft,r.end))
    }
    fn unmark_text(&mut self,_: &mut Window,cx: &mut Context<Self>) {self.marked=None;self.document.record();self.publish(cx);}
    fn replace_text_in_range(&mut self,range: Option<Range<usize>>,text: &str,_: &mut Window,cx: &mut Context<Self>) {
        if self.readonly {return;}
        let draft=&self.document.view.draft;
        let r=range.map(|r|from_utf16(draft,r.start)..from_utf16(draft,r.end)).or(self.marked.take()).unwrap_or(self.document.selection());
        self.document.edit(r,text,true);self.marked=None;self.reveal_caret=true;self.publish(cx);
    }
    fn replace_and_mark_text_in_range(&mut self,range: Option<Range<usize>>,text: &str,selection: Option<Range<usize>>,_: &mut Window,cx: &mut Context<Self>) {
        if self.readonly {return;}
        if self.marked.is_none() {self.document.checkpoint_selection();}
        let draft=&self.document.view.draft;
        let r=range.map(|r|from_utf16(draft,r.start)..from_utf16(draft,r.end)).or(self.marked.clone()).unwrap_or(self.document.selection());
        let start=r.start;
        self.document.edit(r,text,false);
        self.marked=(!text.is_empty()).then_some(start..start+text.len());
        if let Some(r)=selection {self.document.anchor=start+from_utf16(text,r.start);self.document.caret=start+from_utf16(text,r.end);}
        self.reveal_caret=true;self.publish(cx);
    }
    fn bounds_for_range(&mut self,range: Range<usize>,_: Bounds<Pixels>,_: &mut Window,_: &mut Context<Self>) -> Option<Bounds<Pixels>> {
        let text=&self.document.view.draft;let start=from_utf16(text,range.start);let end=from_utf16(text,range.end);
        let g=self.geometry.iter().find(|g|g.range.start<=start && start<=g.range.end)?;
        let a=g.line.x_for_index(display_offset(&g.offsets,start-g.range.start));let b=g.line.x_for_index(display_offset(&g.offsets,end.min(g.range.end)-g.range.start));
        Some(Bounds::new(point(g.origin.x+a,g.origin.y),size((b-a).max(px(1.0)),px(LINE))))
    }
    fn character_index_for_point(&mut self,p: Point<Pixels>,_: &mut Window,_: &mut Context<Self>) -> Option<usize> {
        self.result_point(p,false).map(|i|to_utf16(&self.document.view.draft,i))
    }
    fn set_selected_text_range(&mut self,r: Range<usize>,_: &mut Window,cx: &mut Context<Self>) {
        self.document.anchor=from_utf16(&self.document.view.draft,r.start);self.document.caret=from_utf16(&self.document.view.draft,r.end);cx.notify();
    }
    fn accepts_text_input(&self,_: &mut Window,_: &mut Context<Self>) -> bool {!self.readonly}
    fn text_length_utf16(&mut self,_: &mut Window,_: &mut Context<Self>) -> Option<usize> {Some(self.document.view.draft.encode_utf16().count())}
}

#[cfg(test)]
#[path = "../tests/conflict_native_editor.rs"]
mod tests;
