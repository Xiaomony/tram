use std::{path::PathBuf, sync::LazyLock};
pub const MOUNT_POINT: &str = "/run/tram_btrfs/";
pub const FILE_LOCK: &str = "/run/tram_btrfs.lock";
/**
snapshot folder structure:

the btrfs partion
└── tram_btrfs
    ├── broken
    │   └── broken snapshots
    └── snapshot_groups
        └── default
            ├── daily
            ├── manually
            │   └── 2026-04-16_21:26:00
            │       └── path/to/related/subvolume
            ├── monthly
            └── weekly

snapshot folder name format: yyyy-mm-dd_hh-MM-ss
The application should take a snapshot before recover to a subvolume and place it at `tram_btrfs/broken/`
The application should deny a request to recover a system subvolume
*/
pub const TOP_DIRECTORY_NAME: &str = "tram_btrfs/";
pub const GROUPS_DIRECTORY_NAME: &str = "snapshot_groups";
pub const BROKEN_DIRECTORY_NAME: &str = "broken";

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or(std::env::home_dir().unwrap().join(".config"))
        .join(TOP_DIRECTORY_NAME)
});
pub static MAIN_CONFIG_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| (*CONFIG_DIR).join("tram.toml"));
