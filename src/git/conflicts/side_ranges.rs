use git2::{DiffOptions, Patch};

use crate::{ConflictBlock, GitError, Result};

/// 用 libgit2 的上下文行关系映射 diff3 投影到原始索引版本。
/// 不用字符串 find：重复代码及另一侧的非冲突增删会让 find 命中错误块。
pub(super) fn map_side_ranges(
    projected: &str,
    original: &str,
    blocks: &mut [ConflictBlock],
    ours: bool,
) -> Result<()> {
    if projected == original {
        return Ok(());
    }
    let offsets = line_offsets(projected);
    let original_offsets = line_offsets(original);
    let mut line_map = vec![None; offsets.len()];
    let mut options = DiffOptions::new();
    // xdiff 将上下文长度转为有符号整数，不能使用 u32::MAX。
    options.context_lines(offsets.len().min(i32::MAX as usize) as u32).force_text(true);
    let patch = Patch::from_buffers(
        projected.as_bytes(), None, original.as_bytes(), None, Some(&mut options),
    )?;
    for hunk in 0..patch.num_hunks() {
        for index in 0..patch.num_lines_in_hunk(hunk)? {
            let line = patch.line_in_hunk(hunk, index)?;
            if line.origin() == ' '
                && let (Some(old), Some(new)) = (line.old_lineno(), line.new_lineno())
                && let Some(entry) = line_map.get_mut(old as usize - 1)
            {
                *entry = Some(new as usize - 1);
            }
        }
    }
    for block in blocks {
        let (start, end) = if ours {
            (block.ours_start, block.ours_end)
        } else {
            (block.theirs_start, block.theirs_end)
        };
        let start_line = offsets.partition_point(|offset| *offset <= start).saturating_sub(1);
        let boundary = |line: usize| -> Option<usize> {
            line_map.get(line).copied().flatten()
                .or_else(|| (line > 0).then(|| line_map[line - 1].map(|line| line + 1)).flatten())
                .and_then(|line| original_offsets.get(line).copied())
                .or_else(|| (line == offsets.len() - 1 && offsets[line] == projected.len()).then_some(original.len()))
        };
        let mapped_start = boundary(start_line)
            .ok_or_else(|| GitError::Message("无法定位原始版本中的冲突块".into()))?;
        let mapped_end = mapped_start + end.saturating_sub(start);
        let expected = projected.get(start..end);
        if original.get(mapped_start..mapped_end) != expected {
            return Err(GitError::Message("原始版本与冲突块内容不匹配，请刷新仓库".into()));
        }
        if ours {
            block.ours_start = mapped_start;
            block.ours_end = mapped_end;
        } else {
            block.theirs_start = mapped_start;
            block.theirs_end = mapped_end;
        }
    }
    Ok(())
}

fn line_offsets(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect()
}
