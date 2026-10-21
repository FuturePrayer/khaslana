use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct IndexRequest {
    pub repo_key: String,
    pub force_full: bool,
    pub user_initiated: bool,
    /// 自动刷新和开启请求须在出队时仍启用；显式手动任务不受开关限制。
    pub require_enabled: bool,
}

impl IndexRequest {
    pub fn eligible(&self, enabled: bool) -> bool {
        !self.require_enabled || enabled
    }
}

/// 仅管理 GUI 等待队列；活跃槽直到后台终态才释放，不改变引擎并发策略。
#[derive(Default)]
pub(crate) struct CodeIndexQueue(VecDeque<IndexRequest>);

impl CodeIndexQueue {
    pub fn position(&self, repo: &str) -> Option<usize> {
        self.0.iter().position(|r| r.repo_key == repo).map(|i| i + 1)
    }

    pub fn remove(&mut self, repo: &str) {
        self.0.retain(|r| r.repo_key != repo);
    }

    pub fn push(&mut self, repo: &str, force_full: bool, user_initiated: bool, require_enabled: bool) {
        if let Some(request) = self.0.iter_mut().find(|r| r.repo_key == repo) {
            // 合并不改变点击顺序，全量和用户提示只升级、不降级。
            request.force_full |= force_full;
            request.user_initiated |= user_initiated;
            // 任一合并请求是合法手动操作，就保留其停用时也能执行的资格。
            request.require_enabled &= require_enabled;
        } else {
            self.0.push_back(IndexRequest { repo_key: repo.into(), force_full, user_initiated, require_enabled });
        }
    }

    pub fn pop(&mut self) -> Option<IndexRequest> {
        self.0.pop_front()
    }
}

#[cfg(test)]
#[path = "tests/code_index_queue.rs"]
mod tests;
