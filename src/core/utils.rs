use std::process::{Command, Stdio};

use crate::core::error::{AppError, AppResult};

/// check if the current program is running as root
#[inline]
pub fn check_root_permission() -> AppResult<()> {
    if nix::unistd::Uid::effective().is_root() {
        Ok(())
    } else {
        Err(AppError::NeedRootPermission)
    }
    // let Ok(child_output) = Command::new("id")
    //     .stdin(Stdio::null())
    //     .stdout(Stdio::piped())
    //     .stderr(Stdio::piped())
    //     .arg("-u")
    //     .output()
    // else {
    //     return false;
    // };
    // if child_output.status.success() {
    //     String::from_utf8_lossy(&child_output.stdout).eq("0")
    // } else {
    //     false
    // }
}

pub fn exec_command(command: &'static str, args: &[&str]) -> AppResult<String> {
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
pub fn check_is_btrfs_filesystem(device: &str, raise_error: bool) -> AppResult<bool> {
    let output = exec_command("findmnt", &["-no", "FSTYPE", device])?;
    let result = output.trim().split('\n').all(|t| t == "btrfs");
    if raise_error && !result {
        Err(AppError::NotBtrfs(device.to_string()))
    } else {
        Ok(result)
    }
}
