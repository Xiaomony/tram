use std::io;

pub type CResult<T> = color_eyre::Result<T>;
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("[Child Process Error] command '{command}' cause an error:\n\t{err_msg}")]
    ChildProcess {
        /// Use &'static str to ensure that the command is defined in compile time
        /// Cause this program need root permission, this forbids external command injection
        command: &'static str,
        err_msg: String,
    },

    #[error("[Permission Error]")]
    Permission,
    #[error("[Not Btrfs] '{0}' is not a btrfs file system")]
    NotBtrfs(String),

    #[error("[Multiple Instance] Another Tram TUI instance is running\n\tRaw error: {0}")]
    MultipleInstance(io::Error),

    // TODO: merge this to config error
    #[error("[Config Error] There's something wrong in your config file")]
    InvalidConfig,

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
}

#[inline]
pub fn throw_bug<T: Into<String>, E>(msg: T) -> CResult<E> {
    Err(AppError::Bug(msg.into()).into())
}
#[inline]
pub fn throw_invalid_index<T: Into<String>, E>(index: usize, period: T) -> CResult<E> {
    throw_bug(format!(
        "Invalid index({index}) occurs when {}",
        period.into()
    ))
}
