use std::ffi::{OsStr, OsString};
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::Command;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW,
};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject};

const UNINSTALL_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{5A1F3C92-7B4E-4D8A-9F60-3E2B8C7DA941}_is1";

pub(super) fn registered_install_directory() -> io::Result<Option<PathBuf>> {
    let key = wide(OsStr::new(UNINSTALL_KEY));
    let name = wide(OsStr::new("Inno Setup: App Path"));
    let mut path = [0u16; 32768];
    let mut byte_count = std::mem::size_of_val(&path) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr(), RRF_RT_REG_SZ,
            std::ptr::null_mut(), path.as_mut_ptr().cast(), &mut byte_count,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let len = (byte_count as usize / 2).saturating_sub(1);
    if len == 0 {
        return Ok(None);
    }
    let path = PathBuf::from(OsString::from_wide(&path[..len]));
    match path.canonicalize() {
        Ok(path) => Ok(Some(path)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub(super) fn run_installer(setup: &Path, destination: &Path, legacy: bool) -> io::Result<()> {
    let mut directory = OsString::from("/DIR=");
    // Inno Setup 的目录参数使用普通 Win32 路径，去掉 canonicalize 的扩展前缀。
    let destination = destination.as_os_str().encode_wide().collect::<Vec<_>>();
    let local_prefix: Vec<_> = r"\\?\".encode_utf16().collect();
    let unc_prefix: Vec<_> = r"\\?\UNC\".encode_utf16().collect();
    let directory_path = if let Some(tail) = destination.strip_prefix(unc_prefix.as_slice()) {
        let mut unc = OsString::from(r"\\");
        unc.push(OsString::from_wide(tail));
        unc
    } else {
        OsString::from_wide(destination.strip_prefix(local_prefix.as_slice()).unwrap_or(&destination))
    };
    directory.push(directory_path);
    let mut command = Command::new(setup);
    command.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/NOCANCEL"])
        .arg(directory);
    if legacy {
        command.arg("/KLADOUPGRADE");
    }
    let status = command.status()?;
    if !status.success() {
        return Err(io::Error::other(format!("Klado Setup ({status})")));
    }
    Ok(())
}

pub(super) fn wait_for_exit(pid: u32) -> io::Result<()> {
    let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
    if process.is_null() {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_INVALID_PARAMETER as i32) {
            return Ok(());
        }
        return Err(error);
    }
    let wait = unsafe { WaitForSingleObject(process, 30_000) };
    let result = match wait {
        WAIT_OBJECT_0 => Ok(()),
        WAIT_TIMEOUT => Err(io::ErrorKind::TimedOut.into()),
        _ => Err(io::Error::last_os_error()),
    };
    unsafe { CloseHandle(process) };
    result
}

pub(super) fn show_failure(error: &io::Error) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW};

    let message = khaslana::update::installer_failure_message(error);
    let message = wide(OsStr::new(&message));
    let title = wide(OsStr::new("Klado"));
    unsafe { MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), MB_ICONERROR) };
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}
