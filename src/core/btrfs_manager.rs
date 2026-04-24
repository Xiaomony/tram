use file_lock::{FileLock, FileOptions};
use regex::Regex;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::core::app_config::AppConfig;
use crate::core::btrfs_objects::snapshot_type::SnapshotType;
use crate::core::btrfs_objects::subvolume_snapshot::SubvolumeSnapshot;
use crate::core::error::{AppError, AppResult, throw_invalid_index};
use crate::core::utils::*;
use crate::globals;

pub struct BtrfsManager {
    _device: String,
    file_lock: FileLock,
    subvolumes: Vec<PathBuf>,
    app_config: AppConfig,
    /// The application should take a snapshot before recover to a subvolume
    /// and place it at tram_btrfs/broken/
    /// Also, snapshots should be store in this variable
    /// when fail to parse the path, determine the owner group, snapshot type or date and time
    /// of a snapshot under tram_btrfs/
    broken_snapshots: Vec<SubvolumeSnapshot>,
}

impl BtrfsManager {
    /// create an object based on a specified block device
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
            broken_snapshots: Vec::new(),
        };
        new_obj.get_subvolumes_and_snapshots()?;

        Ok(new_obj)
    }

    /// create an object based on the partion at which the current system root is located
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
    fn get_subvolumes_and_snapshots(&mut self) -> AppResult<()> {
        let btrfs_output =
            exec_command("btrfs", &["subvolume", "list", "-o", globals::MOUNT_POINT])?;
        let r = Regex::new(r"(?m)^ID.*top level 5 path (.+)$")?;

        // store snapshot paths, snapshots must be parsed after subvolumes is fully added
        let mut snapshot_raw_pathes = Vec::new();
        for (_, [raw_path]) in r.captures_iter(&btrfs_output).map(|c| c.extract()) {
            if raw_path.starts_with(globals::TOP_DIRECTORY_NAME) {
                snapshot_raw_pathes.push(raw_path);
            } else {
                // TODO: Detect do subvolumes present as `@` and `@home` layout
                // and setup default snapshot group for first-time launch user
                self.subvolumes.push(PathBuf::from(raw_path));
            }
        }
        // verify subvolumes in config
        let mut removed_config_subvols = Vec::new();
        self.app_config
            .groups
            .iter_mut()
            .for_each(|x| x.verify_subvolumes(&(self.subvolumes), &mut removed_config_subvols));
        if !removed_config_subvols.is_empty() {
            return Err(AppError::ConfigErrSubvolumeNotExist(removed_config_subvols));
        }

        for raw_path in snapshot_raw_pathes {
            self.parse_snapshot_path(raw_path);
        }
        Ok(())
    }

    fn parse_snapshot_path(&mut self, raw_path: &str) {
        let path_parts: Vec<&str> = raw_path.split("/").skip(1).collect();

        // check if the snapshot is under tram_btrfs/snapshot_groups
        if let Some(&globals::SNAPSHOT_GROUPS_DIR_NAME) = path_parts.first()
            // get group name
            && let Some(&group_name) = path_parts.get(1)
            // find the group object it belongs to
            && let Some(group) = self.app_config.groups.iter_mut().find(|x| *x == group_name)
            // get snapshot_types, datetime, name
            && let Some(&snapshot_type) = path_parts.get(2)
            && let Some(&datetime) = path_parts.get(3)
            // get related subvolume path
            && let related_subvolume_path = path_parts[3..].join("/")
            && let Some(related_subvolume) = self.subvolumes.iter().find(|&x| x.eq(&related_subvolume_path))
        {
            if !group.add_snapshot(raw_path, snapshot_type, datetime, related_subvolume.clone()) {
                // regard it as a broken snapshot with related subvolume
                self.broken_snapshots.push(SubvolumeSnapshot::new(
                    raw_path,
                    Some(related_subvolume.clone()),
                ));
            }
        } else {
            // regard it as a broken snapshot without related subvolume
            self.broken_snapshots
                .push(SubvolumeSnapshot::new(raw_path, None));
        }
    }

    pub fn create_snapshot(&mut self, index: usize, snapshot_type: SnapshotType) -> AppResult<()> {
        let Some(group) = self.app_config.groups.get_mut(index) else {
            return throw_invalid_index(index, "creating snapshot");
        };
        group.create_snapshot(snapshot_type)
    }

    pub fn delete_snapshot(&mut self, group_index: usize, snapshot_index: usize) -> AppResult<()> {
        let Some(group) = self.app_config.groups.get_mut(group_index) else {
            return throw_invalid_index(group_index, "deleting snapshot(invalid group index)");
        };
        group.delete_snapshot(snapshot_index)
    }

    pub fn rename_group<T: Into<String>>(&mut self, index: usize, new_name: T) -> AppResult<()> {
        let Some(group) = self.app_config.groups.get_mut(index) else {
            return throw_invalid_index(index, "renaming group");
        };
        group.rename_group(new_name)
    }

    pub fn add_subvol_to_group(
        &mut self,
        group_index: usize,
        subvol_index: usize,
    ) -> AppResult<()> {
        let Some(subvol) = self.subvolumes.get(subvol_index) else {
            return throw_invalid_index(
                subvol_index,
                "add subvolume to group(invalid subvolume index)",
            );
        };
        let Some(group) = self.app_config.groups.get_mut(group_index) else {
            return throw_invalid_index(group_index, "add subvolume to group(invalid group index)");
        };
        group.add_subvolume(subvol);
        Ok(())
    }
}

impl Drop for BtrfsManager {
    fn drop(&mut self) {
        let _ = self.file_lock.unlock();
        let _ = umount_from_default_point();
    }
}
