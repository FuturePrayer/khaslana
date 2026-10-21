use std::fs;

use tempfile::TempDir;

use super::*;
use crate::git::test_support::git_test_support as git_support;

const FIRST: &str = "{ version: 1, name: '初版', steps: [{ op: 'ensureClean' }] }";
const SECOND: &str = "{ version: 1, name: '新版', steps: [{ op: 'ensureClean' }] }";

#[test]
fn listing_does_not_install_and_downloads_only_selected_template() {
    let (source_dir, source, service) = git_support::init_repo();
    git_support::write_file(source_dir.path(), "work-studio-files/first.json5", FIRST);
    git_support::write_file(source_dir.path(), "work-studio-files/second.json5", SECOND);
    git_support::commit_all(&source, "templates");
    let target = TempDir::new().unwrap();
    let url = git_support::path_url(source_dir.path());

    let list = list_templates_from_url(&service, target.path(), &url).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].display_name, "初版");
    assert!(!target.path().join("first.json5").exists());
    assert!(!target.path().join("second.json5").exists());

    let installed = download_cnb_template(target.path(), &list[0]).unwrap();
    assert_eq!(fs::read_to_string(installed).unwrap(), FIRST);
    assert!(!target.path().join("second.json5").exists());
}

#[test]
fn download_keeps_existing_local_template() {
    let (source_dir, source, service) = git_support::init_repo();
    git_support::write_file(source_dir.path(), "work-studio-files/demo.json5", SECOND);
    git_support::commit_all(&source, "remote template");
    let target = TempDir::new().unwrap();
    let url = git_support::path_url(source_dir.path());
    let list = list_templates_from_url(&service, target.path(), &url).unwrap();

    fs::write(target.path().join("demo.json5"), FIRST).unwrap();
    assert!(download_cnb_template(target.path(), &list[0]).is_err());
    assert_eq!(fs::read_to_string(target.path().join("demo.json5")).unwrap(), FIRST);
}

#[test]
fn invalid_remote_template_does_not_appear_in_catalog() {
    let (source_dir, source, service) = git_support::init_repo();
    git_support::write_file(source_dir.path(), "work-studio-files/demo.json5", "{broken");
    git_support::commit_all(&source, "invalid template");
    let target = TempDir::new().unwrap();
    let url = git_support::path_url(source_dir.path());

    assert!(list_templates_from_url(&service, target.path(), &url).is_err());
    assert!(!target.path().join("demo.json5").exists());
}

fn assert_remote_message(error: GitError, key: &str, zh: &str, en: &str) {
    use crate::{language::Language, ui_i18n::format_in};
    assert_eq!(error.message_key(), Some(key));
    let GitError::Localized(message) = error else { panic!("远端模板校验必须保留消息身份") };
    let args = message.args.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect::<Vec<_>>();
    assert_eq!(format_in(Language::ZhCn, message.message, &args), zh);
    assert_eq!(format_in(Language::En, message.message, &args), en);
}

#[test]
fn download_validation_messages_preserve_template_data_and_local_files() {
    let directory = tempfile::tempdir().unwrap();
    let mut template = RemoteTemplateEntry { file_name: "../用户模板.json5".into(), display_name: "用户模板".into(), commit: "commit".into(), content: FIRST.as_bytes().to_vec() };
    assert_remote_message(download_cnb_template(directory.path(), &template).unwrap_err(), "workflow.remote_templates.invalid_file_name", "远端工作流模板文件名无效", "Invalid remote workflow template file name");
    template.file_name = "用户模板.json5".into();
    template.content = vec![0xff];
    assert_remote_message(download_cnb_template(directory.path(), &template).unwrap_err(), "workflow.remote_templates.invalid_utf8", "远端工作流模板不是 UTF-8", "Remote workflow template is not UTF-8");
    assert!(!directory.path().join(&template.file_name).exists());
    template.content = SECOND.as_bytes().to_vec();
    fs::write(directory.path().join(&template.file_name), FIRST).unwrap();
    assert_remote_message(download_cnb_template(directory.path(), &template).unwrap_err(), "workflow.remote_templates.local_template_exists", "本地已存在同名模板：用户模板.json5", "A local template with the same name already exists: 用户模板.json5");
    assert_eq!(fs::read_to_string(directory.path().join(&template.file_name)).unwrap(), FIRST);
}

#[test]
fn catalog_rejects_non_utf8_and_oversized_templates_with_stable_identity() {
    let (source_dir, source, service) = git_support::init_repo();
    let folder = source_dir.path().join("work-studio-files");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("demo.json5"), [0xff]).unwrap();
    git_support::commit_all(&source, "invalid encoding");
    let target = tempfile::tempdir().unwrap();
    let url = git_support::path_url(source_dir.path());
    assert_remote_message(list_templates_from_url(&service, target.path(), &url).unwrap_err(), "workflow.remote_templates.invalid_named_utf8", "远端工作流模板不是 UTF-8：demo.json5", "Remote workflow template is not UTF-8: demo.json5");
    fs::write(folder.join("demo.json5"), vec![b' '; MAX_TEMPLATE_BYTES + 1]).unwrap();
    git_support::commit_all(&source, "oversized template");
    assert_remote_message(list_templates_from_url(&service, target.path(), &url).unwrap_err(), "workflow.remote_templates.template_too_large", "远端工作流模板过大：demo.json5", "Remote workflow template is too large: demo.json5");
}

#[test]
fn remote_template_resource_catalog_has_unique_keys_and_both_languages() {
    let resources = [messages::TEMPLATE_COUNT_LIMIT, messages::TEMPLATE_TOO_LARGE, messages::TOTAL_SIZE_LIMIT, messages::INVALID_NAMED_UTF8, messages::EMPTY_CATALOG, messages::INVALID_FILE_NAME, messages::INVALID_UTF8, messages::LOCAL_TEMPLATE_EXISTS];
    let mut keys = std::collections::HashSet::new();
    for message in resources {
        assert!(keys.insert(message.key));
        assert!(!message.zh.is_empty());
        assert!(!message.en.is_empty());
    }
}
