use std::fs::create_dir_all;

use file_lock::{FileLock, FileOptions};

use crate::core::{
    error::{AppError, AppResult},
    utils::{check_is_btrfs_filesystem, check_root_permission, exec_command, get_crr_os_device},
};
use crate::globals;

pub struct BtrfsManager {
    device: String,
    file_lock: FileLock,
}

impl BtrfsManager {
    pub fn new(device: String) -> AppResult<Self> {
        check_root_permission()?;
        check_is_btrfs_filesystem(&device, true)?;

        // create mount point if not exist
        create_dir_all(globals::MOUNT_POINT)?;
        // mount the device to default mount point
        exec_command("mount", &[&device, globals::MOUNT_POINT])?;

        Self::get_subvolumes()?;

        // put this at last to ensure that root permission is checked through previous commands
        let file_lock = Self::create_file_lock()?;
        Ok(Self { device, file_lock })
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

    fn get_subvolumes() -> AppResult<()> {
        exec_command("btrfs", &["subvolume", "list", globals::MOUNT_POINT])?;
        todo!()
    }
}

impl Drop for BtrfsManager {
    fn drop(&mut self) {
        let _ = self.file_lock.unlock();
        let _ = exec_command("umount", &[globals::MOUNT_POINT]);
    }
}
