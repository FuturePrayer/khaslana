// 分支浏览模式相关的领域类型。
//
// 用于「不切换分支查看其他分支/标签代码」功能：用户在侧边栏右键分支或标签
// 选择「浏览」后，应用进入只读浏览模式，按目录树展示目标引用 tip 提交的
// 完整文件树，并支持查看文件原始内容或与当前 HEAD 的差异。

use crate::types::DiffEncodingInfo;

/// 进入浏览模式时解析出的目标引用：显示名 + tip 提交 OID。
///
/// `display_name` 如 `feature/login`、`origin/main`、`v1.2.0`，
/// `commit_oid` 是该引用 peel 到 commit 后的完整 OID 字符串。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowseTarget {
    pub display_name: String,
    pub commit_oid: String,
}

/// 浏览树条目种类，映射 libgit2 `TreeEntry::kind()`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowseEntryKind {
    /// 目录（libgit2 Tree）。
    Directory,
    /// 文件（libgit2 Blob）。
    File,
    /// 子模块（libgit2 Commit，Gitlink）。
    Submodule,
}

/// 浏览文件树的一行。
///
/// 排序约定：目录在前、文件在后，各自按名称排序（由 Git 服务层保证）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowseEntry {
    /// 相对于仓库根的 git 风格路径，如 `src/main.rs`、`src/types`。
    pub path: String,
    /// 条目名称（路径末尾段），如 `main.rs`。
    pub name: String,
    pub kind: BrowseEntryKind,
    /// 文件字节数；目录填 0。
    pub size: u64,
}

/// 只读文件内容视图的数据。
///
/// 文本文件按选定/检测编码解码后按行切分；二进制文件 `is_binary` 为 true 且 `lines` 为空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowseFileContent {
    pub path: String,
    pub is_binary: bool,
    pub encoding: DiffEncodingInfo,
    pub lines: Vec<String>,
}
