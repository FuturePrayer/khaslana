#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictResolutionSide {
    Ours,
    Theirs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictBlockResolution {
    Ours,
    Theirs,
    BothOursFirst,
    BothTheirsFirst,
}

impl ConflictBlockResolution {
    pub fn render(self, ours: &str, theirs: &str) -> String {
        match self {
            Self::Ours => ours.to_string(),
            Self::Theirs => theirs.to_string(),
            Self::BothOursFirst => format!("{ours}{theirs}"),
            Self::BothTheirsFirst => format!("{theirs}{ours}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictBlockStatus {
    Unresolved,
    Resolved(ConflictBlockResolution),
    Ignored,
    /// 已用自定义合并文本解决（AI 合并建议综合两侧生成的内容）。
    /// 解决后的文本以当前草稿为准，不属于 `ConflictBlockResolution`
    /// 的四种取法，因此独立成状态而非扩展取法枚举。
    Merged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictFileKind {
    Text,
    Binary,
    Unsupported,
    /// 贮藏恢复只在工作区写入标记，没有 Git 索引冲突阶段。
    WorktreeOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictDraftStatus {
    Clean,
    Dirty,
    Applied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictBlock {
    pub base: Option<String>,
    pub ours: String,
    pub theirs: String,
    pub start: usize,
    pub end: usize,
    pub ours_start: usize,
    pub ours_end: usize,
    pub theirs_start: usize,
    pub theirs_end: usize,
    pub status: ConflictBlockStatus,
    pub has_manual_edits: bool,
}

impl ConflictBlock {
    pub fn has_accepted_side(&self, side: ConflictResolutionSide) -> bool {
        !self.has_manual_edits && matches!((self.status, side),
            (ConflictBlockStatus::Resolved(ConflictBlockResolution::BothOursFirst | ConflictBlockResolution::BothTheirsFirst), _)
            | (ConflictBlockStatus::Resolved(ConflictBlockResolution::Ours), ConflictResolutionSide::Ours)
            | (ConflictBlockStatus::Resolved(ConflictBlockResolution::Theirs), ConflictResolutionSide::Theirs))
    }

    pub fn resolved_text(&self, resolution: ConflictBlockResolution) -> String {
        resolution.render(&self.ours, &self.theirs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictFileView {
    pub path: String,
    pub kind: ConflictFileKind,
    pub draft: String,
    pub ours_text: String,
    pub theirs_text: String,
    pub blocks: Vec<ConflictBlock>,
    pub draft_status: ConflictDraftStatus,
    pub fallback_reason: Option<String>,
}

impl ConflictFileView {
    pub fn unresolved_block_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| matches!(block.status, ConflictBlockStatus::Unresolved))
            .count()
    }

    pub fn ignored_block_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| matches!(block.status, ConflictBlockStatus::Ignored))
            .count()
    }

    pub fn handled_block_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| !matches!(block.status, ConflictBlockStatus::Unresolved))
            .count()
    }

    pub fn requires_resolution_confirmation(&self) -> bool {
        self.unresolved_block_count() > 0
    }

    pub fn has_manual_blocks(&self) -> bool {
        self.blocks.iter().any(|block| block.has_manual_edits)
    }

    /// 草稿是否已有本地处理痕迹（AI 合并建议覆盖前的确认判定）：
    /// 草稿被编辑过、任一块已接受/忽略解决、或任一块有手工编辑。
    pub fn has_local_edits(&self) -> bool {
        self.draft_status != ConflictDraftStatus::Clean
            || self.blocks.iter().any(|block| {
                !matches!(block.status, ConflictBlockStatus::Unresolved) || block.has_manual_edits
            })
    }

    pub fn mark_applied(&mut self) {
        self.draft_status = ConflictDraftStatus::Applied;
    }

    pub fn mark_dirty(&mut self) {
        self.draft_status = ConflictDraftStatus::Dirty;
    }

    pub fn apply_block_resolution(
        &mut self,
        block_index: usize,
        resolution: ConflictBlockResolution,
    ) {
        let Some(block) = self.blocks.get(block_index).cloned() else {
            return;
        };
        let replacement = block.resolved_text(resolution);
        self.replace_block_text(
            block_index,
            replacement,
            ConflictBlockStatus::Resolved(resolution),
            false,
        );
    }

    /// 箭头逐侧接受：第二侧追加到已接受内容之后；重复接受同一侧不重复插入。
    /// 显式“接受两边”仍由 apply_block_resolution 指定顺序。
    pub fn accept_block_side(&mut self, block_index: usize, side: ConflictResolutionSide) {
        let Some(block) = self.blocks.get(block_index) else { return };
        if block.has_accepted_side(side) { return }
        let resolution = match (block.status, side) {
            (ConflictBlockStatus::Resolved(ConflictBlockResolution::Ours), ConflictResolutionSide::Theirs)
                if !block.has_manual_edits => ConflictBlockResolution::BothOursFirst,
            (ConflictBlockStatus::Resolved(ConflictBlockResolution::Theirs), ConflictResolutionSide::Ours)
                if !block.has_manual_edits => ConflictBlockResolution::BothTheirsFirst,
            (_, ConflictResolutionSide::Ours) => ConflictBlockResolution::Ours,
            (_, ConflictResolutionSide::Theirs) => ConflictBlockResolution::Theirs,
        };
        self.apply_block_resolution(block_index, resolution);
    }

    pub fn ignore_block(&mut self, block_index: usize) {
        let Some(block) = self.blocks.get_mut(block_index) else {
            return;
        };
        block.status = ConflictBlockStatus::Ignored;
        block.has_manual_edits = false;
        self.draft_status = ConflictDraftStatus::Dirty;
    }

    /// 重置只影响当前块；没有祖先的新增冲突不能伪造一个空祖先。
    pub fn reset_block_to_base(&mut self, block_index: usize) {
        let Some(base) = self.blocks.get(block_index).and_then(|b| b.base.clone()) else { return };
        self.replace_block_text(block_index, base, ConflictBlockStatus::Unresolved, false);
    }

    /// 单行采纳替换结果块中的对应行，超出结果行数则追加；仍需确认整个块。
    pub fn accept_candidate_line(&mut self, block_index: usize, side: ConflictResolutionSide, line: usize) {
        let Some(block) = self.blocks.get(block_index) else { return };
        let source = match side { ConflictResolutionSide::Ours => &block.ours, ConflictResolutionSide::Theirs => &block.theirs };
        let Some(candidate) = source.split_inclusive('\n').nth(line) else { return };
        let current = &self.draft[block.start..block.end];
        let mut lines: Vec<&str> = current.split_inclusive('\n').collect();
        if line < lines.len() { lines[line] = candidate; } else { lines.push(candidate); }
        self.replace_block_text(block_index, lines.concat(), ConflictBlockStatus::Unresolved, true);
    }

    /// 用整份新草稿覆盖（手动编辑路径）：被改动覆盖的块标记手工编辑
    /// 并回到未处理状态，等待用户重新处理。
    pub fn set_draft(&mut self, new_draft: String) {
        self.set_draft_inner(new_draft, false);
    }

    /// 用 AI 合并结果覆盖整份草稿：**所有**冲突块标记为「已合并」——
    /// AI 对整份文件做出了完整合并决定，内容与当前侧一致的块（AI 选择
    /// 保留当前侧）同样视为已处理，否则这些块会永远停留在未处理状态；
    /// 不计入未处理，黄色警告横幅与「还有 N 个未处理」确认随之消失。
    pub fn set_merged_draft(&mut self, new_draft: String) {
        self.set_draft_inner(new_draft, true);
    }

    fn set_draft_inner(&mut self, new_draft: String, merged: bool) {
        if self.draft == new_draft {
            if merged {
                // AI 合并结果与当前草稿完全一致（例如 AI 决定整体保留当前
                // 侧）：内容虽未变化，块状态也是 AI 的合并决定，同样标记
                // 已合并，否则界面停留在「未处理」而状态栏却提示已填入。
                for block in &mut self.blocks {
                    block.status = ConflictBlockStatus::Merged;
                    block.has_manual_edits = false;
                }
                self.draft_status = ConflictDraftStatus::Dirty;
            }
            return;
        }

        let old_draft = self.draft.clone();
        let prefix = shared_prefix_len(&old_draft, &new_draft);
        let suffix = shared_suffix_len(&old_draft[prefix..], &new_draft[prefix..]);
        let old_changed_end = old_draft.len().saturating_sub(suffix);
        let new_changed_end = new_draft.len().saturating_sub(suffix);
        let delta = (new_changed_end as isize - prefix as isize)
            - (old_changed_end as isize - prefix as isize);

        let mut previous_end = 0;
        for block in &mut self.blocks {
            let empty_at_edit = block.start == block.end && block.start == prefix;
            let touches = empty_at_edit || if prefix == old_changed_end {
                block.start < prefix && prefix < block.end
            } else {
                block.start < old_changed_end && block.end > prefix
            };
            if touches && !merged {
                block.has_manual_edits = true;
                block.status = ConflictBlockStatus::Unresolved;
            }
            // 与锚点的左右偏向一致：编辑前的边界不变，编辑后的边界平移，
            // 被删除的边界收敛到替换区。跨块编辑不能让后续接受覆盖相邻块。
            // 参照 Zed side_range：非空块排除边界输入，空块包住新输入。
            let map_start = if prefix == old_changed_end && block.start == prefix && !empty_at_edit {
                new_changed_end
            } else if block.start <= prefix { block.start }
                else if block.start >= old_changed_end { add_signed(block.start, delta) }
                else { prefix };
            let map_end = if empty_at_edit { new_changed_end }
                else if block.end <= prefix { block.end }
                else if block.end >= old_changed_end { add_signed(block.end, delta) }
                else { new_changed_end };
            block.start = clamp_to_char_boundary(&new_draft, map_start.max(previous_end));
            block.end = clamp_to_char_boundary(&new_draft, map_end.max(block.start));
            previous_end = block.end;
        }

        if merged {
            // AI 重新决定了整份文件：所有块（含未被改动区触及、内容与
            // 当前侧一致的块）统一标记已合并。
            for block in &mut self.blocks {
                block.status = ConflictBlockStatus::Merged;
                block.has_manual_edits = false;
            }
        }

        self.draft = new_draft;
        self.draft_status = ConflictDraftStatus::Dirty;
    }

    fn replace_block_text(
        &mut self,
        block_index: usize,
        replacement: String,
        status: ConflictBlockStatus,
        manual: bool,
    ) {
        let Some(block) = self.blocks.get(block_index).cloned() else {
            return;
        };

        self.draft
            .replace_range(block.start..block.end, &replacement);
        let delta = replacement.len() as isize - (block.end - block.start) as isize;
        if let Some(current) = self.blocks.get_mut(block_index) {
            current.end = current.start + replacement.len();
            current.status = status;
            current.has_manual_edits = manual;
        }
        for later in self.blocks.iter_mut().skip(block_index + 1) {
            shift_range(later, delta);
        }
        self.draft_status = ConflictDraftStatus::Dirty;
    }
}

fn shift_range(block: &mut ConflictBlock, delta: isize) {
    block.start = add_signed(block.start, delta);
    block.end = add_signed(block.end, delta).max(block.start);
}

fn add_signed(value: usize, delta: isize) -> usize {
    if delta >= 0 {
        value.saturating_add(delta as usize)
    } else {
        value.saturating_sub(delta.unsigned_abs())
    }
}

/// 把字节偏移吸附到不超过它的最近 UTF-8 字符边界。
/// 越界或本身就在边界上时原样返回（钳制到文本长度）。
fn clamp_to_char_boundary(text: &str, mut offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn shared_prefix_len(left: &str, right: &str) -> usize {
    let mut prefix = 0;
    let mut left_iter = left.char_indices();
    let mut right_iter = right.char_indices();
    loop {
        match (left_iter.next(), right_iter.next()) {
            (Some((left_index, left_ch)), Some((right_index, right_ch)))
                if left_index == prefix && right_index == prefix && left_ch == right_ch =>
            {
                prefix = left_index + left_ch.len_utf8();
            }
            _ => break,
        }
    }
    prefix
}

fn shared_suffix_len(left: &str, right: &str) -> usize {
    let mut suffix = 0;
    let mut left_iter = left.chars().rev();
    let mut right_iter = right.chars().rev();
    loop {
        match (left_iter.next(), right_iter.next()) {
            (Some(left_ch), Some(right_ch)) if left_ch == right_ch => {
                suffix += left_ch.len_utf8();
            }
            _ => break,
        }
    }
    suffix
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_text_view() -> ConflictFileView {
        ConflictFileView {
            path: "f".into(),
            kind: ConflictFileKind::Text,
            draft: "ab".into(),
            ours_text: "a".into(),
            theirs_text: "b".into(),
            blocks: vec![ConflictBlock {
                base: None,
                ours: "a".into(),
                theirs: "b".into(),
                start: 0,
                end: 1,
                ours_start: 0,
                ours_end: 1,
                theirs_start: 0,
                theirs_end: 1,
                status: ConflictBlockStatus::Unresolved,
                has_manual_edits: false,
            }],
            draft_status: ConflictDraftStatus::Clean,
            fallback_reason: None,
        }
    }

    #[test]
    fn has_local_edits_reflects_block_handling_and_manual_edits() {
        // 初始状态：无任何本地处理痕迹。
        let mut view = clean_text_view();
        assert!(!view.has_local_edits());

        // 块级接受解决后：有痕迹。
        view.apply_block_resolution(0, ConflictBlockResolution::Ours);
        assert!(view.has_local_edits());

        // 忽略块：有痕迹。
        let mut view = clean_text_view();
        view.ignore_block(0);
        assert!(view.has_local_edits());

        // 整份草稿被手工改写（set_draft 置 Dirty）：有痕迹。
        let mut view = clean_text_view();
        view.set_draft("rewritten".into());
        assert!(view.has_local_edits());

        // 块带手工编辑标记（即使状态回到 Unresolved）：有痕迹。
        let mut view = clean_text_view();
        view.blocks[0].has_manual_edits = true;
        assert!(view.has_local_edits());
    }

    #[test]
    fn set_merged_draft_marks_touched_blocks_merged() {
        // 双块视图：AI 合并改写第一块区域，第二块内容不动。
        // draft = "AA\nmid\nBB\n"，block0 覆盖 "AA\n"、block1 覆盖 "BB\n"。
        let mut view = ConflictFileView {
            path: "f".into(),
            kind: ConflictFileKind::Text,
            draft: "AA\nmid\nBB\n".into(),
            ours_text: "AA\nmid\nBB\n".into(),
            theirs_text: "XX\nmid\nBB\n".into(),
            blocks: vec![
                ConflictBlock {
                    base: None,
                    ours: "AA\n".into(),
                    theirs: "XX\n".into(),
                    start: 0,
                    end: 3,
                    ours_start: 0,
                    ours_end: 3,
                    theirs_start: 0,
                    theirs_end: 3,
                    status: ConflictBlockStatus::Unresolved,
                    has_manual_edits: false,
                },
                ConflictBlock {
                    base: None,
                    ours: "BB\n".into(),
                    theirs: "BB\n".into(),
                    start: 7,
                    end: 10,
                    ours_start: 7,
                    ours_end: 10,
                    theirs_start: 7,
                    theirs_end: 10,
                    status: ConflictBlockStatus::Unresolved,
                    has_manual_edits: false,
                },
            ],
            draft_status: ConflictDraftStatus::Clean,
            fallback_reason: None,
        };

        view.set_merged_draft("merged\nmid\nBB\n".into());

        assert_eq!(view.draft, "merged\nmid\nBB\n");
        // 被覆盖的块标记「已合并」且不带手工编辑标记（不触发黄色横幅）。
        assert_eq!(view.blocks[0].status, ConflictBlockStatus::Merged);
        assert!(!view.blocks[0].has_manual_edits);
        // 未被改动区触及的块（内容与当前侧一致，AI 选择保留）同样视为
        // 已合并——AI 对整份文件做出了完整决定。
        assert_eq!(view.blocks[1].status, ConflictBlockStatus::Merged);
        assert_eq!(view.unresolved_block_count(), 0);
        assert!(view.draft_status == ConflictDraftStatus::Dirty);
        // 有本地处理痕迹：重复生成 AI 建议仍应弹覆盖确认。
        assert!(view.has_local_edits());
        // 已合并块不计入手工块统计。
        assert!(!view.has_manual_blocks());
    }

    #[test]
    fn set_merged_draft_marks_all_blocks_when_output_matches_current_draft() {
        // AI 合并结果与当前草稿完全一致（AI 整体保留当前侧）：内容不变，
        // 但块状态仍应标记已合并，否则界面停留在「未处理」。
        let mut view = clean_text_view();
        let identical = view.draft.clone();
        view.set_merged_draft(identical);
        assert_eq!(view.blocks[0].status, ConflictBlockStatus::Merged);
        assert_eq!(view.unresolved_block_count(), 0);
        assert!(!view.requires_resolution_confirmation());
        assert!(view.has_local_edits());
        // 手动编辑路径同样输入一致内容时不做任何标记（无可编辑差异）。
        let mut manual = clean_text_view();
        manual.set_draft(manual.draft.clone());
        assert_eq!(manual.blocks[0].status, ConflictBlockStatus::Unresolved);
        assert_eq!(manual.draft_status, ConflictDraftStatus::Clean);
    }

    #[test]
    fn set_merged_draft_clears_confirmation_when_all_blocks_covered() {
        let mut view = clean_text_view();
        view.set_merged_draft("rewritten".into());
        // 唯一块被覆盖：未处理数归零，「应用并标记已解决」不再弹确认。
        assert_eq!(view.unresolved_block_count(), 0);
        assert!(!view.requires_resolution_confirmation());
        assert_eq!(view.blocks[0].status, ConflictBlockStatus::Merged);
    }

    #[test]
    fn set_draft_manual_path_still_marks_unresolved_manual() {
        // 手动编辑路径行为保持不变：被覆盖块回到未处理 + 手工编辑标记。
        let mut view = clean_text_view();
        view.set_draft("rewritten".into());
        assert_eq!(view.blocks[0].status, ConflictBlockStatus::Unresolved);
        assert!(view.blocks[0].has_manual_edits);
        assert_eq!(view.unresolved_block_count(), 1);
    }

    #[test]
    fn set_draft_clamps_block_end_to_char_boundary() {
        let mut view = ConflictFileView {
            path: "f".into(),
            kind: ConflictFileKind::Text,
            draft: "XabY".into(),
            ours_text: String::new(),
            theirs_text: String::new(),
            blocks: vec![ConflictBlock {
                base: None,
                ours: "Xa".into(),
                theirs: String::new(),
                start: 0,
                end: 2,
                ours_start: 0,
                ours_end: 2,
                theirs_start: 0,
                theirs_end: 2,
                status: ConflictBlockStatus::Unresolved,
                has_manual_edits: false,
            }],
            draft_status: ConflictDraftStatus::Clean,
            fallback_reason: None,
        };

        // 把编辑区内的 ASCII 段替换为多字节字符：块尾平移 +1 后落在“中”
        // （3 字节）内部，必须吸附到字符边界，否则后续按字节 replace_range
        // 会 panic。
        view.set_draft("X中Y".into());

        assert_eq!(view.draft, "X中Y");
        assert!(view.draft.is_char_boundary(view.blocks[0].start));
        assert!(view.draft.is_char_boundary(view.blocks[0].end));
        assert!(view.blocks[0].start <= view.blocks[0].end);
    }

    #[test]
    fn clamp_to_char_boundary_snaps_backwards() {
        // “中”占 1..4，字节 3 在其内部，应吸附到 1。
        assert_eq!(clamp_to_char_boundary("a中b", 3), 1);
        assert_eq!(clamp_to_char_boundary("a中b", 4), 4);
        assert_eq!(clamp_to_char_boundary("a中b", 99), 5);
        assert_eq!(clamp_to_char_boundary("a中b", 0), 0);
    }
}
