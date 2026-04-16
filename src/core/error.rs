use std::io;
use thiserror::Error;

use crate::core::utils::check_root_permission;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("[Child Process Error] command '{command}' cause an error:\n\t{err_msg}")]
    ChildProcess {
        command: &'static str,
        err_msg: String,
    },

    #[error("[Permission Error] {0}")]
    Permission(String),
    #[error("[Permission Error] This program needs root permission. Run it with 'sudo'.")]
    NeedRootPermission,

    #[error("[Not Btrfs] '{0}' is not a btrfs file system")]
    NotBtrfs(String),

    #[error("[Multiple Instance] Another Tram TUI instance is running\n\tRaw error: {0}")]
    MultipleInstance(io::Error),

    #[error("[Io Error] {0}")]
    Io(#[from] io::Error),
}

impl AppError {
    /// try to transform the error type when a child process failed
    pub fn try_new(command: &'static str, err_msg: String) -> AppError {
        let is_not_root = check_root_permission().is_err();
        if is_not_root && err_msg.to_lowercase().contains("operation not permitted") {
            AppError::Permission(format!("command `{command}`: {err_msg}"))
        } else {
            AppError::ChildProcess { command, err_msg }
        }
    }
}
