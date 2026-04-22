use crate::core::error::{AppError, AppResult, ExplainError};
use crate::globals;
use nix::mount::{self, MsFlags};
use std::ffi::OsStr;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use time::{OffsetDateTime, macros::format_description};

/// check if the current program is running as root
#[inline]
pub fn check_root_permission() -> AppResult<()> {
    if nix::unistd::Uid::effective().is_root() {
        Ok(())
    } else {
        Err(AppError::NoRootPermission)
    }
}

pub fn exec_command<T: AsRef<OsStr>>(command: &'static str, args: &[T]) -> AppResult<String> {
    let child_output = Command::new(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .output()?;
    if child_output.status.success() {
        Ok(String::from_utf8_lossy(&child_output.stdout).to_string())
    } else {
        let err_msg = String::from_utf8_lossy(&child_output.stderr);
        Err(AppError::try_new(command, err_msg.to_string()))
    }
}

#[inline]
pub fn get_crr_os_device() -> AppResult<String> {
    exec_command("findmnt", &["-no", "SOURCE", "/"])
        .map(|x| x.split_once('[').map(|t| t.0.to_string()).unwrap_or(x))
}

/// check whether the given device is a btrfs filesystem
/// raise_error: if true, raise an error instead of return Ok(false)
pub fn check_is_btrfs_filesystem(device: &str) -> AppResult<()> {
    let output = exec_command("findmnt", &["-no", "FSTYPE", device])?;
    let result = output.trim().split('\n').all(|t| t == "btrfs");
    if result {
        Ok(())
    } else {
        Err(AppError::NotBtrfs(device.to_string()))
    }
}

#[inline]
pub fn mount_to_default_point(device: &str) -> AppResult<()> {
    create_dir_all(globals::MOUNT_POINT)?;
    mount::mount(
        Some(device),
        globals::MOUNT_POINT,
        Some("btrfs"),
        MsFlags::MS_NODEV | MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        None::<&str>,
    )
    .map_err(|e| {
        e.explain(format!(
            "Can't mount {} to {}",
            device,
            globals::MOUNT_POINT
        ))
    })
}

#[inline]
pub fn umount_from_default_point() -> AppResult<()> {
    mount::umount(globals::MOUNT_POINT)
        .map_err(|e| e.explain(format!("Can't umount from {}", globals::MOUNT_POINT)))
}

#[inline]
/// join the given path to the mount point
pub fn mount_point_join<T: AsRef<Path>>(path: T) -> PathBuf {
    PathBuf::from(globals::MOUNT_POINT).join(path)
}

/// return (date, time)
pub fn get_current_date_time() -> (String, String) {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let date = now
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap();
    let time = now
        .format(format_description!("[hour]:[minute]:[second]"))
        .unwrap();
    (date, time)
}
