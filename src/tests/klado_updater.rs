use super::*;

#[test]
fn portable_update_restores_previous_files_when_replacement_fails() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join(EXE);
    let updater = temp.path().join(UPDATER);
    let backup = temp.path().join("backup");
    let incoming_updater = temp.path().join("incoming-updater.exe");
    fs::write(&app, b"old program").unwrap();
    fs::write(&updater, b"old updater").unwrap();
    fs::write(&incoming_updater, b"new updater").unwrap();

    let result = install_portable(
        &app, &temp.path().join("missing.exe"), &incoming_updater, &backup, None,
    );

    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(fs::read(app).unwrap(), b"old program");
    assert_eq!(fs::read(updater).unwrap(), b"old updater");
}
