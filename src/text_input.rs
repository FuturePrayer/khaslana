use std::ops::{Deref, DerefMut, Range};

use gpui::{
    App, Bounds, Context, Element, ElementId, ElementInputHandler, FocusHandle, GlobalElementId,
    IntoElement, LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Style, TextRun, Window,
    fill, point, px, relative, rgb, rgba, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::ui_helpers::{COLOR_TEXT, COLOR_TEXT_FAINT};
use crate::{FieldId, RepositoryView};

#[derive(Clone, Debug)]
pub(crate) struct TextEditState {
    pub(crate) value: String,
    pub(crate) secret: bool,
    pub(crate) caret: usize,
    pub(crate) selection_anchor: Option<usize>,
    pub(crate) marked_range: Option<Range<usize>>,
    pub(crate) last_layout: Option<ShapedLine>,
    pub(crate) last_bounds: Option<Bounds<Pixels>>,
    pub(crate) is_selecting: bool,
}

impl TextEditState {
    pub(crate) fn new() -> Self {
        Self {
            value: String::new(),
            secret: false,
            caret: 0,
            selection_anchor: None,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(value: &str, secret: bool) -> Self {
        Self {
            value: value.to_string(),
            secret,
            caret: value.len(),
            selection_anchor: None,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    pub(crate) fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.caret = self.value.len();
        self.selection_anchor = None;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }

    pub(crate) fn clear(&mut self) {
        self.value.clear();
        self.caret = 0;
        self.selection_anchor = None;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }

    pub(crate) fn display_text(&self) -> String {
        if self.secret {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    pub(crate) fn display_byte_for_value_byte(&self, value_byte: usize) -> usize {
        if self.secret {
            self.value[..value_byte].chars().count()
        } else {
            value_byte
        }
    }

    fn value_byte_for_display_byte(&self, display_byte: usize) -> usize {
        if !self.secret {
            return clamp_to_char_boundary(&self.value, display_byte);
        }
        if display_byte == 0 {
            return 0;
        }
        self.value
            .char_indices()
            .nth(display_byte)
            .map(|(index, _)| index)
            .unwrap_or(self.value.len())
    }

    pub(crate) fn selected_range(&self) -> Option<Range<usize>> {
        let anchor = self.selection_anchor?;
        if anchor == self.caret {
            None
        } else if anchor < self.caret {
            Some(anchor..self.caret)
        } else {
            Some(self.caret..anchor)
        }
    }

    pub(crate) fn input_range(&self) -> Range<usize> {
        self.selected_range().unwrap_or(self.caret..self.caret)
    }

    pub(crate) fn selection_reversed(&self) -> bool {
        self.selection_anchor
            .is_some_and(|anchor| self.caret < anchor)
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        self.selected_range()
            .map(|range| self.value[range].to_string())
    }

    pub(crate) fn copyable_selected_text(&self) -> Option<String> {
        (!self.secret).then(|| self.selected_text()).flatten()
    }

    pub(crate) fn select_all(&mut self) {
        self.caret = self.value.len();
        self.selection_anchor = Some(0);
        self.marked_range = None;
    }

    pub(crate) fn delete_selection(&mut self) -> bool {
        let Some(range) = self.selected_range() else {
            return false;
        };
        let start = range.start;
        self.value.replace_range(range, "");
        self.caret = start;
        self.selection_anchor = None;
        self.marked_range = None;
        true
    }

    pub(crate) fn insert_text(&mut self, text: &str, multiline: bool) {
        self.delete_selection();
        let text = if multiline {
            text.to_string()
        } else {
            text.replace(['\r', '\n'], "")
        };
        self.value.insert_str(self.caret, &text);
        self.caret += text.len();
        self.selection_anchor = None;
        self.marked_range = None;
    }

    pub(crate) fn delete_backward(&mut self) {
        if self.delete_selection() || self.caret == 0 {
            return;
        }
        let previous = self.previous_grapheme_boundary(self.caret);
        self.value.replace_range(previous..self.caret, "");
        self.caret = previous;
    }

    pub(crate) fn delete_forward(&mut self) {
        if self.delete_selection() || self.caret >= self.value.len() {
            return;
        }
        let next = self.next_grapheme_boundary(self.caret);
        self.value.replace_range(self.caret..next, "");
    }

    pub(crate) fn move_caret_to(&mut self, position: usize, extend_selection: bool) {
        let position = clamp_to_char_boundary(&self.value, position);
        if extend_selection {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.caret);
            }
        } else {
            self.selection_anchor = None;
        }
        self.caret = position;
        self.marked_range = None;
    }

    pub(crate) fn move_to(&mut self, position: usize) {
        self.move_caret_to(position, false);
    }

    pub(crate) fn select_to(&mut self, position: usize) {
        self.move_caret_to(position, true);
    }

    pub(crate) fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection && let Some(range) = self.selected_range() {
            self.move_caret_to(range.start, false);
            return;
        }
        let previous = self.previous_grapheme_boundary(self.caret);
        self.move_caret_to(previous, extend_selection);
    }

    pub(crate) fn move_right(&mut self, extend_selection: bool) {
        if !extend_selection && let Some(range) = self.selected_range() {
            self.move_caret_to(range.end, false);
            return;
        }
        let next = self.next_grapheme_boundary(self.caret);
        self.move_caret_to(next, extend_selection);
    }

    fn previous_grapheme_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .rev()
            .find_map(|(index, _)| (index < offset).then_some(index))
            .unwrap_or(0)
    }

    fn next_grapheme_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .find_map(|(index, _)| (index > offset).then_some(index))
            .unwrap_or(self.value.len())
    }

    pub(crate) fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.value.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub(crate) fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.value.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    pub(crate) fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub(crate) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(crate) fn replace_text_in_utf16_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.input_range());
        let text = text.replace(['\r', '\n'], "");
        self.value.replace_range(range.clone(), &text);
        self.caret = range.start + text.len();
        self.selection_anchor = None;
        self.marked_range = None;
    }

    pub(crate) fn replace_and_mark_text_in_utf16_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        selected_range_utf16: Option<Range<usize>>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.input_range());
        let text = text.replace(['\r', '\n'], "");
        let selected_range = selected_range_utf16
            .as_ref()
            .map(|range| {
                offset_from_utf16_in_text(&text, range.start)
                    ..offset_from_utf16_in_text(&text, range.end)
            })
            .unwrap_or_else(|| text.len()..text.len());
        self.value.replace_range(range.clone(), &text);
        if text.is_empty() {
            self.marked_range = None;
        } else {
            self.marked_range = Some(range.start..range.start + text.len());
        }
        self.caret = range.start + selected_range.end;
        self.selection_anchor = Some(range.start + selected_range.start);
        if self.selection_anchor == Some(self.caret) {
            self.selection_anchor = None;
        }
    }

    pub(crate) fn text_for_utf16_range(&self, range_utf16: &Range<usize>) -> String {
        let range = self.range_from_utf16(range_utf16);
        if self.secret {
            "*".repeat(self.value[range].chars().count())
        } else {
            self.value[range].to_string()
        }
    }

    pub(crate) fn index_for_mouse_position(&self, position: gpui::Point<Pixels>) -> usize {
        if self.value.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.value.len();
        }
        let display_byte = line.closest_index_for_x(position.x - bounds.left());
        self.value_byte_for_display_byte(display_byte)
    }

    pub(crate) fn byte_for_approx_x(&self, x: f32) -> usize {
        let mut width = 0.0;
        let mut previous = 0;
        for (index, ch) in self.value.char_indices() {
            let char_width = approx_input_char_width(ch);
            if x < width + char_width / 2.0 {
                return index;
            }
            width += char_width;
            previous = index + ch.len_utf8();
        }
        previous
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TextFieldState {
    pub(crate) focus: FocusHandle,
    pub(crate) placeholder: SharedString,
    edit: TextEditState,
}

impl TextFieldState {
    pub(crate) fn new(
        cx: &mut Context<RepositoryView>,
        placeholder: impl Into<SharedString>,
    ) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            placeholder: placeholder.into(),
            edit: TextEditState::new(),
        }
    }

    pub(crate) fn secret(mut self) -> Self {
        self.edit.secret = true;
        self
    }
}

impl Deref for TextFieldState {
    type Target = TextEditState;

    fn deref(&self) -> &Self::Target {
        &self.edit
    }
}

impl DerefMut for TextFieldState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.edit
    }
}

fn clamp_to_char_boundary(value: &str, mut position: usize) -> usize {
    position = position.min(value.len());
    while position > 0 && !value.is_char_boundary(position) {
        position -= 1;
    }
    position
}

fn offset_from_utf16_in_text(text: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_count = 0;
    for ch in text.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += ch.len_utf16();
        utf8_offset += ch.len_utf8();
    }
    utf8_offset
}

fn approx_input_char_width(ch: char) -> f32 {
    if ch == '\t' {
        28.0
    } else if ch.is_ascii() {
        7.0
    } else if ch_width_is_wide(ch) {
        14.0
    } else {
        8.5
    }
}

fn ch_width_is_wide(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF00..=0xFFEF
    )
}

pub(crate) struct SingleLineInputElement {
    pub(crate) field_id: FieldId,
    pub(crate) entity: gpui::Entity<RepositoryView>,
}

pub(crate) struct SingleLineInputPrepaint {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for SingleLineInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SingleLineInputElement {
    type RequestLayoutState = ();
    type PrepaintState = SingleLineInputPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let view = self.entity.read(cx);
        let field = view.field(self.field_id);
        let display = field.display_text();
        let is_empty = field.value.is_empty();
        let style = window.text_style();
        let display_text: SharedString = if is_empty {
            field.placeholder.clone()
        } else {
            display.clone().into()
        };
        let text_color: gpui::Hsla = if is_empty {
            rgba(COLOR_TEXT_FAINT).into()
        } else {
            style.color
        };
        let base_run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if !is_empty {
            if let Some(marked_range) = field.marked_range.as_ref() {
                let marked_start = field.display_byte_for_value_byte(marked_range.start);
                let marked_end = field.display_byte_for_value_byte(marked_range.end);
                vec![
                    TextRun {
                        len: marked_start,
                        ..base_run.clone()
                    },
                    TextRun {
                        len: marked_end.saturating_sub(marked_start),
                        underline: Some(gpui::UnderlineStyle {
                            color: Some(base_run.color),
                            thickness: px(1.0),
                            wavy: false,
                        }),
                        ..base_run.clone()
                    },
                    TextRun {
                        len: display_text.len().saturating_sub(marked_end),
                        ..base_run
                    },
                ]
                .into_iter()
                .filter(|run| run.len > 0)
                .collect()
            } else {
                vec![base_run]
            }
        } else {
            vec![base_run]
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text.clone(), font_size, &runs, None);
        let focused = field.focus.is_focused(window);
        let selection = if !is_empty {
            field.selected_range().map(|range| {
                let start = field.display_byte_for_value_byte(range.start);
                let end = field.display_byte_for_value_byte(range.end);
                fill(
                    Bounds::from_corners(
                        point(bounds.left() + line.x_for_index(start), bounds.top()),
                        point(bounds.left() + line.x_for_index(end), bounds.bottom()),
                    ),
                    rgba(0x6aa9ff55),
                )
            })
        } else {
            None
        };
        let cursor = if focused && selection.is_none() {
            let caret = if is_empty {
                0
            } else {
                field.display_byte_for_value_byte(field.caret)
            };
            Some(fill(
                Bounds::new(
                    point(bounds.left() + line.x_for_index(caret), bounds.top()),
                    size(px(1.5), bounds.bottom() - bounds.top()),
                ),
                rgb(COLOR_TEXT),
            ))
        } else {
            None
        };
        SingleLineInputPrepaint {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.entity.read(cx).field(self.field_id).focus.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.entity.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line = prepaint.line.take().unwrap_or_default();
        let _ = line.paint(bounds.origin, window.line_height(), window, cx);
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
        self.entity.update(cx, |view, _cx| {
            let field = view.field_mut(self.field_id);
            field.last_layout = Some(line);
            field.last_bounds = Some(bounds);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::TextEditState;

    #[test]
    fn text_field_edits_at_utf8_char_boundaries() {
        let mut field = TextEditState::for_test("ab你cd", false);

        field.move_caret_to(3, false);
        assert_eq!(field.caret, 2);
        field.insert_text("X", false);
        assert_eq!(field.value, "abX你cd");

        field.delete_backward();
        assert_eq!(field.value, "ab你cd");
        assert_eq!(field.caret, 2);

        field.move_caret_to("ab你".len(), false);
        field.delete_backward();
        assert_eq!(field.value, "abcd");
        assert_eq!(field.caret, 2);

        field.set_value("ab你cd");
        field.move_caret_to(2, false);
        field.delete_forward();
        assert_eq!(field.value, "abcd");
        assert_eq!(field.caret, 2);
    }

    #[test]
    fn text_field_selection_replace_and_navigation_work() {
        let mut field = TextEditState::for_test("abcdef", false);

        field.move_caret_to(2, false);
        field.move_caret_to(5, true);
        assert_eq!(field.selected_text().as_deref(), Some("cde"));

        field.insert_text("X", false);
        assert_eq!(field.value, "abXf");
        assert_eq!(field.caret, 3);
        assert_eq!(field.selected_range(), None);

        field.select_all();
        assert_eq!(field.selected_text().as_deref(), Some("abXf"));
        field.move_left(false);
        assert_eq!(field.caret, 0);
        assert_eq!(field.selected_range(), None);

        field.move_right(false);
        assert_eq!(field.caret, 1);
    }

    #[test]
    fn text_field_single_line_paste_strips_newlines() {
        let mut single_line = TextEditState::for_test("ab", false);
        single_line.move_caret_to(1, false);
        single_line.insert_text("x\ny\r\nz", false);
        assert_eq!(single_line.value, "axyzb");

        let mut multiline = TextEditState::for_test("ab", false);
        multiline.move_caret_to(1, false);
        multiline.insert_text("x\ny", true);
        assert_eq!(multiline.value, "ax\nyb");
    }

    #[test]
    fn text_field_secret_display_masks_and_blocks_copyable_text() {
        let mut field = TextEditState::for_test("密码12", true);

        assert_eq!(field.display_text(), "****");
        assert_eq!(field.display_byte_for_value_byte("密码".len()), 2);

        field.select_all();
        assert_eq!(field.selected_text().as_deref(), Some("密码12"));
        assert_eq!(field.copyable_selected_text(), None);

        field.clear();
        assert!(field.value.is_empty());
        assert_eq!(field.caret, 0);
        assert_eq!(field.selected_range(), None);
    }

    #[test]
    fn text_field_click_position_uses_wide_character_widths() {
        let field = TextEditState::for_test("a你b", false);

        assert_eq!(field.byte_for_approx_x(0.0), 0);
        assert_eq!(field.byte_for_approx_x(8.0), 1);
        assert_eq!(field.byte_for_approx_x(18.0), "a你".len());
        assert_eq!(field.byte_for_approx_x(80.0), "a你b".len());
    }

    #[test]
    fn text_field_utf16_ranges_round_trip() {
        let field = TextEditState::for_test("a你😀b", false);
        let range = "a你".len().."a你😀".len();

        assert_eq!(field.range_to_utf16(&range), 2..4);
        assert_eq!(field.range_from_utf16(&(2..4)), range);
        assert_eq!(field.text_for_utf16_range(&(1..4)), "你😀");
    }

    #[test]
    fn text_field_grapheme_navigation_keeps_emoji_together() {
        let mut field = TextEditState::for_test("a👨‍👩‍👧‍👦b", false);
        field.move_caret_to(field.value.len(), false);

        field.move_left(false);
        assert_eq!(field.caret, "a👨‍👩‍👧‍👦".len());
        field.move_left(false);
        assert_eq!(field.caret, 1);
        field.delete_forward();
        assert_eq!(field.value, "ab");
    }

    #[test]
    fn text_field_platform_replacement_strips_newlines() {
        let mut field = TextEditState::for_test("ab", false);
        field.move_caret_to(1, false);

        field.replace_text_in_utf16_range(None, "x\ny\r\nz");
        assert_eq!(field.value, "axyzb");
        assert_eq!(field.caret, 4);
    }

    #[test]
    fn text_field_marked_text_replacement_updates_selection() {
        let mut field = TextEditState::for_test("ab", false);
        field.move_caret_to(1, false);

        field.replace_and_mark_text_in_utf16_range(None, "你", Some(1..1));
        assert_eq!(field.value, "a你b");
        assert_eq!(field.marked_range, Some(1.."a你".len()));
        assert_eq!(field.caret, "a你".len());
        assert_eq!(field.selected_range(), None);
    }

    #[test]
    fn text_field_secret_utf16_text_is_masked() {
        let field = TextEditState::for_test("密码12", true);

        assert_eq!(field.text_for_utf16_range(&(0..4)), "****");
    }
}
