use super::*;
use crate::language::Language;

#[test]
fn coverage_reasons_keep_stable_codes_and_translate_old_data_at_display() {
    for (legacy, code, english) in [
        ("超过单文件解析上限", "parse_size_limit", "File exceeds the parsing size limit"),
        ("二进制文件", "binary_file", "Binary file"),
        ("解析器未返回语法树", "no_syntax_tree", "The parser did not return a syntax tree"),
        ("当前语言不支持符号解析", "unsupported_language", "Symbol parsing is not supported for this language"),
        ("文件名或生成产物过滤", "filtered_file", "Filtered file name or generated artifact"),
        ("目录剪枝或独立子仓库", "pruned_directory", "Pruned directory or separate nested repository"),
        ("非 UTF-8 路径", "non_utf8_path", "Non-UTF-8 path"),
        ("目录或产物过滤、符号链接、子模块", "filtered_snapshot_entry", "Filtered directory or artifact, symbolic link, or submodule"),
    ] {
        assert_eq!(normalize_coverage_reason(legacy), code);
        assert_eq!(normalize_coverage_reason(code), code);
        assert_eq!(coverage_reason_label_in(Language::ZhCn, code), legacy);
        assert_eq!(coverage_reason_label_in(Language::En, legacy), english);
        assert_eq!(coverage_reason_label_in(Language::En, code), english);
    }
}

#[test]
fn unknown_tool_reason_is_never_translated_or_normalized() {
    let original = "工具原文：解析器未返回语法树 {path}";
    assert_eq!(normalize_coverage_reason(original), original);
    assert_eq!(coverage_reason_label_in(Language::En, original), original);
    assert_eq!(coverage_reason_label_in(Language::ZhCn, original), original);
}
