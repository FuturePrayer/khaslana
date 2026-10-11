//! 单份结果缓冲区与只读候选的显示映射。候选从不进入草稿、选区或剪贴板。
use std::{collections::HashSet, ops::Range};
use khaslana::{ConflictBlock, ConflictBlockResolution, ConflictBlockStatus, ConflictDraftStatus, ConflictFileView, ConflictResolutionSide};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Row {
    Header(usize),
    Label(usize, Option<ConflictResolutionSide>),
    Candidate { block: usize, side: ConflictResolutionSide, line: usize, range: Range<usize> },
    Result { line: usize, range: Range<usize>, owner: Option<usize> },
}

#[derive(Clone)]
struct Revision {
    draft: String,
    blocks: Vec<ConflictBlock>,
    expanded: HashSet<usize>,
    anchor: usize,
    caret: usize,
}

pub(super) struct Document {
    pub view: ConflictFileView,
    pub rows: Vec<Row>,
    pub expanded: HashSet<usize>,
    pub anchor: usize,
    pub caret: usize,
    pub generation: u64,
    history: Vec<Revision>,
    revision: usize,
}

impl Document {
    pub fn new(view: ConflictFileView) -> Self {
        let expanded = view.blocks.iter().enumerate().filter_map(|(i,b)| (b.status == ConflictBlockStatus::Unresolved).then_some(i)).collect();
        let mut this = Self {view, rows: Vec::new(), expanded, anchor: 0, caret: 0, generation: 0, history: Vec::new(), revision: 0};
        this.rebuild();
        this.record();
        this
    }

    pub fn selection(&self) -> Range<usize> { self.anchor.min(self.caret)..self.anchor.max(self.caret) }

    pub fn rebuild(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.rows.clear();
        let mut next_block = 0;
        for (line, range) in line_ranges(&self.view.draft).into_iter().enumerate() {
            while let Some(block) = self.view.blocks.get(next_block) {
                if block.start > range.end || (block.start == range.end && range.end > range.start) { break; }
                let index = next_block;
                self.rows.push(Row::Header(index));
                if self.expanded.contains(&index) {
                    for (side, text) in [(ConflictResolutionSide::Ours, &block.ours), (ConflictResolutionSide::Theirs, &block.theirs)] {
                        self.rows.push(Row::Label(index, Some(side)));
                        for (line, range) in candidate_ranges(text).into_iter().enumerate() {
                            self.rows.push(Row::Candidate {block: index, side, line, range});
                        }
                    }
                    self.rows.push(Row::Label(index, None));
                }
                next_block += 1;
            }
            let owner = self.view.blocks.iter().position(|b| b.start < range.end && b.end > range.start || b.start == b.end && b.start == range.start);
            self.rows.push(Row::Result {line, range, owner});
        }
    }

    pub fn record(&mut self) {
        if self.history.get(self.revision).is_some_and(|r| r.draft == self.view.draft && r.blocks == self.view.blocks) { return; }
        self.history.truncate(self.revision + 1);
        self.history.push(Revision {draft: self.view.draft.clone(), blocks: self.view.blocks.clone(), expanded: self.expanded.clone(), anchor: self.anchor, caret: self.caret});
        // 按字节预算和次数双重限制，避免大文件编辑历史无限增长。
        let mut bytes: usize = self.history.iter().map(|r| r.draft.len() + r.blocks.iter().map(|b| b.ours.len()+b.theirs.len()+b.base.as_ref().map_or(0,String::len)).sum::<usize>()).sum();
        while self.history.len() > 1 && (self.history.len() > 256 || bytes > 32*1024*1024) {
            let old = self.history.remove(0);
            bytes = bytes.saturating_sub(old.draft.len()+old.blocks.iter().map(|b| b.ours.len()+b.theirs.len()+b.base.as_ref().map_or(0,String::len)).sum::<usize>());
        }
        self.revision = self.history.len()-1;
    }

    pub fn checkpoint_selection(&mut self) {
        if let Some(r)=self.history.get_mut(self.revision) {
            r.anchor=self.anchor;r.caret=self.caret;r.expanded=self.expanded.clone();
        }
    }

    pub fn undo(&mut self, redo: bool) -> bool {
        let next = if redo { self.revision.checked_add(1).filter(|&i| i < self.history.len()) } else { self.revision.checked_sub(1) };
        let Some(next) = next else { return false };
        self.revision = next;
        let r = &self.history[next];
        self.view.draft = r.draft.clone();
        self.view.blocks = r.blocks.clone();
        self.view.draft_status = ConflictDraftStatus::Dirty;
        self.expanded = r.expanded.clone();
        self.anchor = r.anchor;
        self.caret = r.caret;
        self.rebuild();
        true
    }

    pub fn sync(&mut self, view: &ConflictFileView) {
        if self.view == *view { return; }
        self.checkpoint_selection();
        let (range, replacement) = text_replacement(&self.view.draft, &view.draft);
        let rebase = |p: usize| if p <= range.start {p} else if p >= range.end {p-range.len()+replacement.len()} else {range.start+replacement.len()};
        self.anchor = rebase(self.anchor);
        self.caret = rebase(self.caret);
        for (i, b) in view.blocks.iter().enumerate() {
            if b.status != ConflictBlockStatus::Unresolved && self.view.blocks.get(i).is_some_and(|old| old.status != b.status) { self.expanded.remove(&i); }
        }
        self.view = view.clone();
        self.rebuild();
        self.record();
    }

    pub fn edit(&mut self, range: Range<usize>, text: &str, record: bool) {
        if record {self.checkpoint_selection();}
        let mut draft = self.view.draft.clone();
        draft.replace_range(range.clone(), text);
        self.view.set_draft(draft);
        self.caret = range.start+text.len();
        self.anchor = self.caret;
        self.rebuild();
        if record { self.record(); }
    }

    pub fn accept(&mut self, block: usize, resolution: ConflictBlockResolution) {
        // D 的块头是完整的保留选择；展开后重新选择应替换该块，而非隐式追加。
        let mut view = self.view.clone();
        view.apply_block_resolution(block, resolution);
        self.sync(&view);
        self.expanded.remove(&block);
        self.rebuild();
        if let Some(r)=self.history.get_mut(self.revision) {r.expanded=self.expanded.clone();}
    }

    pub fn reset(&mut self, block: usize) {
        let mut view = self.view.clone();
        view.reset_block_to_base(block);
        self.expanded.insert(block);
        self.sync(&view);
    }

    pub fn adopt_line(&mut self, block: usize, side: ConflictResolutionSide, line: usize) {
        let mut view = self.view.clone();
        view.accept_candidate_line(block, side, line);
        self.sync(&view);
    }
}

pub(super) fn line_ranges(text: &str) -> Vec<Range<usize>> {
    let mut start = 0;
    let mut ranges = Vec::new();
    for (i,b) in text.bytes().enumerate() { if b == b'\n' { ranges.push(start..i+1); start=i+1; } }
    ranges.push(start..text.len());
    ranges
}

pub(super) fn candidate_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = line_ranges(text);
    if ranges.last().is_some_and(Range::is_empty) { ranges.pop(); }
    ranges
}

pub(super) fn text_replacement<'a>(old: &str, new: &'a str) -> (Range<usize>, &'a str) {
    let prefix = old.chars().zip(new.chars()).take_while(|(a,b)| a == b).map(|(c,_)| c.len_utf8()).sum::<usize>();
    let suffix = old[prefix..].chars().rev().zip(new[prefix..].chars().rev()).take_while(|(a,b)| a == b).map(|(c,_)| c.len_utf8()).sum::<usize>();
    (prefix..old.len()-suffix, &new[prefix..new.len()-suffix])
}

/// 按词 LCS 标出候选相对于祖先的变化；超长行回退字符前后缀，限制二次复杂度。
pub(super) fn word_changes(base: &str, text: &str) -> Vec<Range<usize>> {
    let a: Vec<_> = base.split_word_bounds().collect();
    let b: Vec<_> = text.split_word_bound_indices().collect();
    if a.len()*b.len() > 65536 {
        let (range, replacement) = text_replacement(base,text);
        return vec![range.start..range.start+replacement.len()];
    }
    let width=b.len()+1;
    let mut matrix=vec![0usize;(a.len()+1)*width];
    for i in (0..a.len()).rev() { for j in (0..b.len()).rev() {
        matrix[i*width+j]=if a[i]==b[j].1 {matrix[(i+1)*width+j+1]+1} else {matrix[(i+1)*width+j].max(matrix[i*width+j+1])};
    } }
    let (mut i,mut j)=(0,0);
    let mut changes=Vec::new();
    while j<b.len() {
        if i<a.len() && a[i]==b[j].1 {i+=1;j+=1;}
        else if i<a.len() && matrix[(i+1)*width+j]>matrix[i*width+j+1] {i+=1;}
        else {changes.push(b[j].0..b[j].0+b[j].1.len());j+=1;}
    }
    changes
}

#[cfg(test)]
#[path = "../tests/conflict_document.rs"]
mod tests;
