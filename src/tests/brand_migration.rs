use super::*;

fn existing_database(home: &Path) -> Connection {
    fs::create_dir_all(home).unwrap();
    let database = Connection::open(home.join(PREVIOUS_DB)).unwrap();
    database.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA wal_autocheckpoint=0;
         CREATE TABLE schema_meta (version INTEGER);
         INSERT INTO schema_meta VALUES (10);",
    ).unwrap();
    database
}

#[test]
fn portable_migration_snapshots_wal_and_rolls_back_only_new_database() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("Portable Klado").join("data");
    let _database = existing_database(&home);
    let source = home.join(PREVIOUS_DB);
    fs::write(home.join("workflow-mcp.json5"), b"workflow settings").unwrap();

    let prepared = prepare_from(&source, &home.canonicalize().unwrap()).unwrap();
    let migrated = Connection::open(prepared.staged.join(DB_FILE_NAME)).unwrap();
    let version: i64 = migrated.query_row("SELECT version FROM schema_meta", [], |row| row.get(0)).unwrap();
    assert_eq!(version, 10);
    drop(migrated);

    drop(prepared);
    assert!(source.is_file());
    assert!(home.join("workflow-mcp.json5").is_file());
    assert!(!home.join(DB_FILE_NAME).exists());
}

#[test]
fn install_migration_copies_workflows_and_reviews_without_update_cache() {
    let temp = tempfile::tempdir().unwrap();
    let previous = temp.path().join("Khaslana").join("data");
    let next = temp.path().join("Klado").join("data");
    let _database = existing_database(&previous);
    for directory in ["workflows", "ai-reviews", "updates"] {
        fs::create_dir_all(previous.join(directory)).unwrap();
        fs::write(previous.join(directory).join("fixture"), directory).unwrap();
    }
    let prepared = prepare_from(&previous.join(PREVIOUS_DB), &next).unwrap();
    assert!(!next.exists());
    prepared.publish().unwrap();
    assert!(prepared.database.is_file());
    assert_eq!(fs::read(next.join("workflows/fixture")).unwrap(), b"workflows");
    assert_eq!(fs::read(next.join("ai-reviews/fixture")).unwrap(), b"ai-reviews");
    assert!(!next.join("updates").exists());
    drop(prepared);
    assert!(next.exists());
    assert!(previous.join("updates/fixture").is_file());
}

#[test]
fn migration_refuses_to_replace_existing_klado_data() {
    let temp = tempfile::tempdir().unwrap();
    let previous = temp.path().join("Khaslana");
    let next = temp.path().join("Klado");
    let _database = existing_database(&previous);
    fs::create_dir(&next).unwrap();
    let existing = next.join(DB_FILE_NAME);
    fs::write(&existing, b"existing").unwrap();

    let result = prepare_from(&previous.join(PREVIOUS_DB), &next);
    assert!(matches!(result, Err(error) if error.kind() == io::ErrorKind::AlreadyExists));
    assert_eq!(fs::read(existing).unwrap(), b"existing");
}
