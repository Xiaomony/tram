use std::{io, path::PathBuf};
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
    NoRootPermission,

    #[error("[Not Btrfs] '{0}' is not a btrfs file system")]
    NotBtrfs(String),

    #[error("[System Operation Fail] {msg} \n\tRaw error: {raw_err}")]
    SystemOperationFail {
        raw_err: nix::errno::Errno,
        msg: String,
    },

    #[error("[Multiple Instance] Another Tram TUI instance is running\n\tRaw error: {0}")]
    MultipleInstance(io::Error),

    #[error("[Config Error] There're non-existent subvolumes in config")]
    ConfigErrSubvolumeNotExist(Vec<PathBuf>),

    #[error("[Io Error] {0}")]
    Io(#[from] io::Error),
    #[error("[Regex Error] {0}")]
    Regex(#[from] regex::Error),
    #[error("[Config Parsing Error] Fail to parse config\n\t{0}")]
    ParseConfigFail(#[from] toml::de::Error),
    #[error("[Config Generating Error] Fail to parse config\n\t{0}")]
    GenConfigFail(#[from] toml::ser::Error),

    #[error("[Bug] This might be a bug. Please report it:\n\t{0}")]
    Bug(String),
    // TODO: display of this type
    #[error("Multiple errors are caused:\n\t{0:?}")]
    CombinedError(Vec<AppError>),
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
    #[inline]
    fn is_combined_err(&self) -> bool {
        matches!(self, AppError::CombinedError(_))
    }

    pub fn chain(mut self, new: Self) -> Self {
        use AppError::CombinedError;
        match (&mut self, new) {
            (CombinedError(e), CombinedError(mut e_new)) => e.append(&mut e_new),
            (CombinedError(e), e_new) => e.push(e_new),
            (_, CombinedError(mut e_new)) => {
                e_new.push(self);
                self = AppError::CombinedError(e_new);
            }
            (_, e_new) => self = AppError::CombinedError(vec![self, e_new]),
        }
        self
    }
}

pub trait ExplainError {
    fn explain(self, msg: String) -> AppError;
}

impl ExplainError for nix::errno::Errno {
    #[inline]
    fn explain(self, msg: String) -> AppError {
        AppError::SystemOperationFail { raw_err: self, msg }
    }
}

pub trait ExtendResult<T> {
    fn chain(&mut self, new: AppResult<T>);
}

impl<T> ExtendResult<T> for Result<T, AppError> {
    fn chain(&mut self, new: AppResult<T>) {
        if let Err(e_new) = new {
            if let Err(e) = self {
                *e = std::mem::replace(
                    e,
                    AppError::Bug(
                        "Bug occured inside trait `ExtendResult`, function `chain()`".to_string(),
                    ),
                )
                .chain(e_new);
            } else {
                *self = Err(e_new);
            }
        }
    }
}
