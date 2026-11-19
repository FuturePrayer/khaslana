use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use rusqlite::{Connection, OpenFlags};

use super::{DB_FILE_NAME, LAST_DATA_HOME_POINTER_FILE, PORTABLE_MIGRATED_MARKER, pick_active_path};

const PREVIOUS_DB: &str = "khaslana.sqlite3";

fn previous_homes() -> Option<(PathBuf, PathBuf)> {
    let dirs = ProjectDirs::from("", "", "Khaslana")?;
    Some((dirs.config_dir().into(), dirs.data_local_dir().into()))
}

pub(super) fn previous_database_path(exe: &Path) -> Option<PathBuf> {
    let portable = exe.parent().map(|parent| parent.join("data").join(PREVIOUS_DB));
    let (roaming, fixed) = previous_homes()?;
    let pointer = fs::read_to_string(fixed.join(LAST_DATA_HOME_POINTER_FILE)).ok()
        .map(|text| PathBuf::from(text.trim()).join(PREVIOUS_DB))
        .filter(|path| path.is_absolute());
    let fixed_db = fixed.join(PREVIOUS_DB);
    pick_active_path(
        Some(roaming.join(PREVIOUS_DB)),
        portable,
        Some(roaming),
        Some(fixed_db.clone()),
        pointer,
        super::classify_exe_location(exe),
    ).filter(|path| path.is_file())
        .or_else(|| fixed_db.is_file().then_some(fixed_db))
}

/// 新目录提交后才切换活动库。更新失败时撤回本次复制，旧库和私钥原地可用。
pub struct PreparedKladoData {
    database: PathBuf,
    staged: PathBuf,
    in_place: bool,
    _lock: File,
}

impl PreparedKladoData {
    pub fn commit(self) -> io::Result<()> {
        self.publish()?;
        if let Some(home) = self.database.parent() {
            super::record_last_data_home(home);
        }
        Ok(())
    }

    fn publish(&self) -> io::Result<()> {
        let home = self.database.parent().ok_or(io::ErrorKind::InvalidInput)?;
        if self.database.try_exists()? || (!self.in_place && home.try_exists()?) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        if self.in_place {
            fs::rename(self.staged.join(DB_FILE_NAME), &self.database)
        } else {
            fs::rename(&self.staged, home)
        }
    }
}

impl Drop for PreparedKladoData {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.staged);
    }
}

pub fn prepare_klado_data(previous_exe: &Path, next_exe: &Path) -> io::Result<Option<PreparedKladoData>> {
    let Some(database) = previous_database_path(previous_exe) else { return Ok(None) };
    prepare_from(&database, &destination_for(&database, next_exe)?).map(Some)
}

pub fn migrate_klado_data_at_startup() -> io::Result<()> {
    let Some(database) = super::default_database_path()
        .filter(|path| path.file_name().is_some_and(|name| name == PREVIOUS_DB))
    else { return Ok(()) };
    let exe = std::env::current_exe()?;
    let destination = destination_for(&database, &exe)?;
    prepare_from(&database, &destination)?.commit()
}

fn destination_for(source: &Path, next_exe: &Path) -> io::Result<PathBuf> {
    let source_dir = source.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let (roaming, fixed) = previous_homes().ok_or(io::ErrorKind::NotFound)?;
    if same_directory(source_dir, &fixed) {
        super::fixed_database_dir().ok_or_else(|| io::ErrorKind::NotFound.into())
    } else if same_directory(source_dir, &roaming) {
        super::legacy_database_dir().ok_or_else(|| io::ErrorKind::NotFound.into())
    } else {
        Ok(next_exe.parent().ok_or(io::ErrorKind::InvalidInput)?.join("data"))
    }
}

fn prepare_from(source: &Path, destination: &Path) -> io::Result<PreparedKladoData> {
    let source_dir = source.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let in_place = same_directory(source_dir, destination);
    let parent = destination.parent().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(parent)?;
    let lock = OpenOptions::new().write(true).create(true).truncate(false)
        .open(parent.join(".klado-data-migration.lock"))?;
    lock.lock()?;

    let database = destination.join(DB_FILE_NAME);
    if database.exists() || (!in_place && destination.exists()) {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Klado data destination already exists"));
    }

    let staged = parent.join(format!(".klado-data-{}-{:x}", std::process::id(), super::now_seconds()));
    fs::create_dir(&staged)?;
    let prepared = PreparedKladoData { database, staged: staged.clone(), in_place, _lock: lock };
    let staged_db = staged.join(DB_FILE_NAME);
    // SQLite 生成一致快照，包括尚未 checkpoint 的 WAL，避免仅复制主库丢记录。
    let conn = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(invalid_database)?;
    conn.execute("VACUUM INTO ?1", [staged_db.to_str().ok_or(io::ErrorKind::InvalidInput)?])
        .map_err(invalid_database)?;
    let copy = Connection::open_with_flags(&staged_db, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(invalid_database)?;
    copy.query_row("SELECT COUNT(*) FROM schema_meta", [], |row| row.get::<_, i64>(0))
        .map_err(invalid_database)?;
    drop(copy);
    OpenOptions::new().write(true).open(&staged_db)?.sync_all()?;

    if !in_place {
        crate::ssh_keys::copy_managed_tree(&source_dir.join("ssh/keys"), &staged.join("ssh/keys"))?;
        copy_data_tree(source_dir, &staged, true)?;
    }
    Ok(prepared)
}

fn copy_data_tree(source: &Path, destination: &Path, top: bool) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if top && matches!(name.to_str(), Some(
            "khaslana.sqlite3" | "khaslana.sqlite3-wal" | "khaslana.sqlite3-shm" |
            "khaslana.sqlite3-journal" | DB_FILE_NAME | "klado.sqlite3-wal" | "klado.sqlite3-shm" |
            "klado.sqlite3-journal" | "updates" | LAST_DATA_HOME_POINTER_FILE |
            PORTABLE_MIGRATED_MARKER | super::PORTABLE_PENDING_MARKER |
            super::EXE_RELOCATION_PENDING_MARKER
        )) {
            continue;
        }
        if !top && source.file_name().is_some_and(|parent| parent == "ssh") && name == "keys" {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_data_tree(&entry.path(), &destination.join(name), false)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination.join(name))?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported entry in previous data directory"));
        }
    }
    Ok(())
}

fn invalid_database(error: rusqlite::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn same_directory(first: &Path, second: &Path) -> bool {
    first == second || fs::canonicalize(first).is_ok_and(|first| {
        fs::canonicalize(second).is_ok_and(|second| first == second)
    })
}

#[cfg(test)]
#[path = "../tests/brand_migration.rs"]
mod tests;
