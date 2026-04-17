use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use file_lock::{FileLock, FileOptions};
use regex::Regex;

use crate::core::{
    app_config::AppConfig,
    error::{AppError, AppResult},
    subvolume::Subvolume,
    utils::{
        check_is_btrfs_filesystem, check_root_permission, exec_command, get_crr_os_device,
        mount_to_default_point, umount_from_default_point,
    },
};
use crate::globals;

pub struct BtrfsManager {
    _device: String,
    file_lock: FileLock,
    subvolumes: Vec<Subvolume>,
    app_config: AppConfig,
}

impl BtrfsManager {
    pub fn new(device: String) -> AppResult<Self> {
        check_root_permission()?;
        let file_lock = Self::create_file_lock()?;
        check_is_btrfs_filesystem(&device)?;
        mount_to_default_point(&device)?;

        // create a directory to store snapshots under the mounted device
        create_dir_all(Path::new(globals::MOUNT_POINT).join(globals::TOP_DIRECTORY_NAME))?;

        let mut new_obj = Self {
            _device: device,
            file_lock,
            subvolumes: Vec::new(),
            app_config: AppConfig::load_config()?,
        };
        new_obj.get_subvolumes()?;

        Ok(new_obj)
    }

    pub fn new_default_partion() -> AppResult<Self> {
        Self::new(get_crr_os_device()?)
    }

    fn create_file_lock() -> AppResult<FileLock> {
        let options = FileOptions::new().write(true).create(true);
        match FileLock::lock(globals::FILE_LOCK, false, options) {
            Ok(file_lock) => Ok(file_lock),
            Err(e) => Err(AppError::MultipleInstance(e)),
        }
    }

    /*
    ID 256 gen 57067 top level 5 path @
    ID 257 gen 57067 top level 5 path @home
    ID 365 gen 56472 top level 5 path timeshift-btrfs/snapshots/2026-04-16_15-07-30/@
    ID 366 gen 56473 top level 5 path timeshift-btrfs/snapshots/2026-04-16_15-07-30/@home
    ID 369 gen 57190 top level 5 path tram_btrfs/snapshot_groups/default/manually/2026-04-16_21-26-00/@
    */
    fn get_subvolumes(&mut self) -> AppResult<()> {
        let btrfs_output =
            exec_command("btrfs", &["subvolume", "list", "-o", globals::MOUNT_POINT])?;
        let r = Regex::new(r"(?m)^ID.*top level 5 path (.+)$")?;
        for (_, [raw_path]) in r.captures_iter(&btrfs_output).map(|c| c.extract()) {
            println!("{raw_path}");
            let path = PathBuf::from(raw_path);
            if path.starts_with(globals::TOP_DIRECTORY_NAME) {
                let path_parts: Vec<&str> = raw_path.split("/").skip(1).collect();
                let Some(&p1) = path_parts.first() else {
                    todo!()
                };
                match p1 {
                    globals::GROUPS_DIRECTORY_NAME => {}
                    globals::BROKEN_DIRECTORY_NAME => {}
                    _ => (),
                }
            } else {
                self.subvolumes.push(Subvolume::new(path));
            }
        }
        Ok(())
    }
}

impl Drop for BtrfsManager {
    fn drop(&mut self) {
        let _ = self.file_lock.unlock();
        let _ = umount_from_default_point();
    }
}
