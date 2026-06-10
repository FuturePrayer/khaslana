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
pub enum ConflictFileKind {
    Text,
    Binary,
    Unsupported,
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
    pub resolution: Option<ConflictBlockResolution>,
    pub has_manual_edits: bool,
}

impl ConflictBlock {
    pub fn resolved_text(&self, resolution: ConflictBlockResolution) -> String {
        resolution.render(&self.ours, &self.theirs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictFileView {
    pub path: String,
    pub kind: ConflictFileKind,
    pub draft: String,
    pub blocks: Vec<ConflictBlock>,
    pub draft_status: ConflictDraftStatus,
    pub fallback_reason: Option<String>,
}

impl ConflictFileView {
    pub fn unresolved_block_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| block.resolution.is_none())
            .count()
    }

    pub fn has_manual_blocks(&self) -> bool {
        self.blocks.iter().any(|block| block.has_manual_edits)
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
        self.replace_block_text(block_index, replacement, Some(resolution), false);
    }

    pub fn set_draft(&mut self, new_draft: String) {
        if self.draft == new_draft {
            return;
        }

        let old_draft = self.draft.clone();
        let prefix = shared_prefix_len(&old_draft, &new_draft);
        let suffix = shared_suffix_len(&old_draft[prefix..], &new_draft[prefix..]);
        let old_changed_end = old_draft.len().saturating_sub(suffix);
        let new_changed_end = new_draft.len().saturating_sub(suffix);
        let delta = (new_changed_end as isize - prefix as isize)
            - (old_changed_end as isize - prefix as isize);

        for block in &mut self.blocks {
            if block.end <= prefix {
                continue;
            }
            if block.start >= old_changed_end {
                shift_range(block, delta);
                continue;
            }

            block.has_manual_edits = true;
            if block.start > prefix {
                block.start = prefix;
            }
            block.end = add_signed(block.end, delta).max(block.start);
            block.resolution = None;
        }

        self.draft = new_draft;
        self.draft_status = ConflictDraftStatus::Dirty;
    }

    fn replace_block_text(
        &mut self,
        block_index: usize,
        replacement: String,
        resolution: Option<ConflictBlockResolution>,
        manual: bool,
    ) {
        let Some(block) = self.blocks.get(block_index).cloned() else {
            return;
        };

        self.draft.replace_range(block.start..block.end, &replacement);
        let delta = replacement.len() as isize - (block.end - block.start) as isize;
        if let Some(current) = self.blocks.get_mut(block_index) {
            current.end = current.start + replacement.len();
            current.resolution = resolution;
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

