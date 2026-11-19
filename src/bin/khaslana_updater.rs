#![cfg_attr(windows, windows_subsystem = "windows")]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use khaslana::storage::PreparedKladoData;

const EXE: &str = "klado.exe";
const UPDATER: &str = "klado_updater.exe";
const PREVIOUS_EXE: &str = "khaslana.exe";
const PREVIOUS_UPDATER: &str = "khaslana_updater.exe";

#[cfg(windows)]
#[path = "klado_update_windows.rs"]
mod windows;

fn main() {
    let result = if std::env::current_exe().ok()
        .is_some_and(|path| named(&path, PREVIOUS_EXE))
    {
        launch_from_previous_path()
    } else {
        let mut args = std::env::args_os().skip(1);
        if args.next().as_deref() == Some(OsStr::new("--migrate-installed-data")) {
            migrate_installed_data(args)
        } else {
            update()
        }
    };
    if let Err(error) = result {
        eprintln!("{error}");
        #[cfg(windows)]
        windows::show_failure(&error);
        std::process::exit(1);
    }
}

fn migrate_installed_data(mut args: impl Iterator<Item = OsString>) -> io::Result<()> {
    let previous = PathBuf::from(args.next().ok_or(io::ErrorKind::InvalidInput)?);
    let next = PathBuf::from(args.next().ok_or(io::ErrorKind::InvalidInput)?);
    let previous_dir = fs::canonicalize(previous.parent().ok_or(io::ErrorKind::InvalidInput)?)?;
    let next_dir = fs::canonicalize(next.parent().ok_or(io::ErrorKind::InvalidInput)?)?;
    let previous = previous_dir.join(PREVIOUS_EXE);
    let next = next_dir.join(EXE);
    if let Some(prepared) = khaslana::storage::prepare_klado_data(&previous, &next)? {
        prepared.commit()?;
    }
    if previous_dir == next_dir || previous_dir.with_file_name("Klado") == next_dir {
        let source = std::env::current_exe()?;
        if replace_from(&source, &previous).is_ok() {
            let _ = fs::remove_file(previous_dir.join(PREVIOUS_UPDATER));
        }
    }
    Ok(())
}

fn launch_from_previous_path() -> io::Result<()> {
    let current = std::env::current_exe()?;
    let parent = current.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let local = parent.join(EXE);
    let sibling = parent.with_file_name("Klado").join(EXE);
    let target = [local, sibling].into_iter().find(|path| path.is_file())
        .ok_or(io::ErrorKind::NotFound)?;
    let status = Command::new(target).args(std::env::args_os().skip(1)).status()?;
    std::process::exit(status.code().unwrap_or(1));
}

fn update() -> io::Result<()> {
    // 已发布客户端会从 staging 运行更新器，并继续传这组参数。
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let target = PathBuf::from(argument(&args, "--target-exe")?);
    let new_exe = PathBuf::from(argument(&args, "--new-exe")?);
    let new_updater = PathBuf::from(argument(&args, "--new-updater")?);
    let backup = PathBuf::from(argument(&args, "--backup-dir")?);
    let pid = argument(&args, "--pid")?.to_str().and_then(|pid| pid.parse().ok())
        .filter(|pid| *pid != 0).ok_or(io::ErrorKind::InvalidInput)?;
    let restart = args.iter().any(|arg| arg == "--restart");
    wait_for_exit(pid)?;
    let migrated_exe = match install(&target, &new_exe, &new_updater, &backup) {
        Ok(exe) => exe,
        Err(error) => {
            // 应用已退出。原入口仍可用时恢复旧进程，保留错误便于用户重试。
            if restart && target.is_file() {
                let _ = Command::new(&target).spawn();
            }
            return Err(error);
        }
    };
    if restart {
        Command::new(&migrated_exe)
            .arg("--complete-klado-migration")
            .current_dir(migrated_exe.parent().ok_or(io::ErrorKind::InvalidInput)?)
            .spawn()?;
    }
    Ok(())
}

fn install(target: &Path, new_exe: &Path, new_updater: &Path, backup: &Path) -> io::Result<PathBuf> {
    let current_dir = target.parent().ok_or(io::ErrorKind::InvalidInput)?;
    let current_dir = fs::canonicalize(current_dir)?;
    let lock = fs::OpenOptions::new().write(true).create(true).truncate(false)
        .open(current_dir.join(".klado-update.lock"))?;
    lock.try_lock()?;
    let legacy = !named(target, EXE);
    #[cfg(windows)]
    let installed = windows::registered_install_directory()?.is_some_and(|dir| dir == current_dir);
    #[cfg(not(windows))]
    let installed = false;
    let destination = if installed && legacy {
        if current_dir.parent().is_none() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        current_dir.with_file_name("Klado")
    } else {
        current_dir.clone()
    };
    let next = destination.join(EXE);
    if destination != current_dir && destination.try_exists()?
        && fs::read_dir(&destination)?.any(|entry| {
            entry.map_or(true, |entry| {
                let name = entry.file_name();
                name != ".klado-data-migration.lock" && !name.to_string_lossy().starts_with(".klado-data-")
            })
        })
    {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, destination.display().to_string()));
    }
    if legacy && next.exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, next.display().to_string()));
    }
    let prepared_data = if legacy {
        khaslana::storage::prepare_klado_data(target, &next)?
    } else {
        None
    };

    if installed {
        #[cfg(windows)]
        windows::run_installer(&new_exe.with_file_name("klado-setup.exe"), &destination, legacy)?;
        if let Some(prepared) = prepared_data {
            prepared.commit()?;
        }
    } else {
        install_portable(&next, new_exe, new_updater, backup, prepared_data)?;
    }
    if legacy && named(target, PREVIOUS_EXE) {
        // 旧任务栏固定项与外部 MCP 配置仍可经原路径进入新版本。
        if replace_from(new_updater, &current_dir.join(PREVIOUS_EXE)).is_ok() {
            let _ = fs::remove_file(current_dir.join(PREVIOUS_UPDATER));
        }
    }
    Ok(next)
}

fn install_portable(
    target: &Path,
    new_exe: &Path,
    new_updater: &Path,
    backup: &Path,
    prepared_data: Option<PreparedKladoData>,
) -> io::Result<()> {
    fs::create_dir_all(backup)?;
    let target_updater = target.with_file_name(UPDATER);
    let saved_exe = backup.join("klado.exe.bak");
    let saved_updater = backup.join("klado_updater.exe.bak");
    let had_exe = backup_existing(target, &saved_exe)?;
    let had_updater = backup_existing(&target_updater, &saved_updater)?;
    replace_from(new_updater, &target_updater)?;
    if let Err(error) = replace_from(new_exe, target) {
        restore_backup(&target_updater, &saved_updater, had_updater)?;
        return Err(error);
    }
    if let Some(prepared) = prepared_data
        && let Err(error) = prepared.commit()
    {
        let restore_app = restore_backup(target, &saved_exe, had_exe);
        restore_backup(&target_updater, &saved_updater, had_updater)?;
        restore_app?;
        return Err(error);
    }
    if had_exe {
        let _ = fs::remove_file(saved_exe);
    }
    if had_updater {
        let _ = fs::remove_file(saved_updater);
    }
    Ok(())
}

fn restore_backup(target: &Path, saved: &Path, existed: bool) -> io::Result<()> {
    if existed {
        replace_from(saved, target)
    } else {
        fs::remove_file(target)
    }
}

fn backup_existing(target: &Path, saved: &Path) -> io::Result<bool> {
    if !target.try_exists()? {
        return Ok(false);
    }
    fs::copy(target, saved).map(|_| true)
}

fn replace_from(source: &Path, target: &Path) -> io::Result<()> {
    // 临时文件与目标同卷。最后一步才替换，复制或写入失败时旧 exe 仍在。
    let staging = target.with_extension(format!("replacing-{}", std::process::id()));
    let mut destination = fs::OpenOptions::new().create_new(true).write(true).open(&staging)?;
    let result = (|| {
        io::copy(&mut fs::File::open(source)?, &mut destination)?;
        destination.sync_all()?;
        drop(destination);
        fs::rename(&staging, target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result
}

fn named(path: &Path, name: &str) -> bool {
    path.file_name().and_then(OsStr::to_str).is_some_and(|file| file.eq_ignore_ascii_case(name))
}

fn argument<'a>(args: &'a [OsString], name: &str) -> io::Result<&'a OsStr> {
    args.windows(2).find(|pair| pair[0] == name).map(|pair| pair[1].as_os_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, name))
}

#[cfg(windows)]
fn wait_for_exit(pid: u32) -> io::Result<()> {
    windows::wait_for_exit(pid)
}

#[cfg(not(windows))]
fn wait_for_exit(_pid: u32) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

#[cfg(test)]
#[path = "../tests/klado_updater.rs"]
mod tests;
