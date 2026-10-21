use super::*;

#[test]
fn locale_normalization_and_order() {
    for locale in ["en", "en-US", "EN_gb", " en-AU ", "en_US.UTF-8", "en_GB@euro"] {
        assert_eq!(supported_locale(locale), Some(Language::En));
    }
    for locale in ["zh-CN", "zh_Hans", "zh-Hans-CN", "zh-SG", "zh_CN.UTF-8"] {
        assert_eq!(supported_locale(locale), Some(Language::ZhCn));
    }
    for locale in ["", "zh", "zh-TW", "zh-Hant", "zh-Hant-CN", "zh-HK", "fr"] {
        assert_eq!(supported_locale(locale), None);
    }
    assert_eq!(preferred_language(["zh-TW", "fr", "en-GB", "zh-CN"]), Language::En);
    assert_eq!(preferred_language(["zh-CN", "en"]), Language::ZhCn);
    assert_eq!(preferred_language(["fr", "zh-Hant"]), Language::ZhCn);
    assert_eq!(preferred_language([]), Language::ZhCn);
}

#[test]
fn unknown_preferences_fall_back_independently() {
    assert_eq!(UiLanguagePreference::from_id("future"), UiLanguagePreference::System);
    assert_eq!(PromptLanguagePreference::from_id("future"), PromptLanguagePreference::FollowUi);
    for pref in [UiLanguagePreference::System, UiLanguagePreference::ZhCn, UiLanguagePreference::En] {
        assert_eq!(UiLanguagePreference::from_id(pref.id()), pref);
    }
    for pref in [PromptLanguagePreference::FollowUi, PromptLanguagePreference::ZhCn, PromptLanguagePreference::En] {
        assert_eq!(PromptLanguagePreference::from_id(pref.id()), pref);
    }
}

#[test]
fn task_snapshot_separates_system_and_output() {
    for ui in [Language::ZhCn, Language::En] {
        for prompt in [PromptLanguagePreference::ZhCn, PromptLanguagePreference::En] {
            let snapshot = TaskLanguages::new(ui, prompt);
            assert_eq!(snapshot.output, ui);
            assert_eq!(snapshot.system, prompt.resolve(ui));
            assert!(!snapshot.output_instruction().is_empty());
        }
        assert_eq!(TaskLanguages::new(ui, PromptLanguagePreference::FollowUi).system, ui);
    }
    assert!(TaskLanguages::new(Language::ZhCn, PromptLanguagePreference::En)
        .output_instruction().contains("Simplified Chinese"));
}

#[test]
fn preferences_are_independent_and_persistent() {
    let storage = crate::storage::AppStorage::open_in_memory().unwrap();
    assert_eq!(storage.load_language_preferences().unwrap(), LanguagePreferences::default());
    storage.save_prompt_language(PromptLanguagePreference::En).unwrap();
    storage.save_ui_language(UiLanguagePreference::ZhCn).unwrap();
    assert_eq!(storage.load_language_preferences().unwrap(), LanguagePreferences {
        ui: UiLanguagePreference::ZhCn, prompt: PromptLanguagePreference::En,
    });
    storage.save_ui_language(UiLanguagePreference::System).unwrap();
    assert_eq!(storage.load_language_preferences().unwrap().prompt, PromptLanguagePreference::En);
}

#[test]
fn old_database_and_invalid_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.sqlite3");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE schema_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO schema_meta VALUES ('schema_version', '7');").unwrap();
    }
    let storage = crate::storage::AppStorage::open(&path).unwrap();
    assert_eq!(storage.load_language_preferences().unwrap(), LanguagePreferences::default());
    storage.save_prompt_language(PromptLanguagePreference::En).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute("UPDATE language_preferences SET ui_language = 'future'", []).unwrap();
    assert_eq!(storage.load_language_preferences().unwrap(), LanguagePreferences {
        ui: UiLanguagePreference::System, prompt: PromptLanguagePreference::En,
    });
    conn.execute("UPDATE language_preferences SET ui_language = 'en', prompt_language = 'future'", []).unwrap();
    assert_eq!(storage.load_language_preferences().unwrap(), LanguagePreferences {
        ui: UiLanguagePreference::En, prompt: PromptLanguagePreference::FollowUi,
    });
    conn.execute_batch("CREATE TRIGGER reject_language BEFORE UPDATE ON language_preferences
        BEGIN SELECT RAISE(ABORT, 'read only'); END;").unwrap();
    assert!(storage.save_ui_language(UiLanguagePreference::ZhCn).is_err());
    assert_eq!(storage.load_language_preferences().unwrap().ui, UiLanguagePreference::En);
}
