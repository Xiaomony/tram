use crate::core::btrfs_objects::group_snapshot::{GroupSnapshot, SnapshotType};
use crate::core::error::{AppError, AppResult, ExtendResult};
use crate::globals;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    group_name: String,
    // subvolume pathes loaded from configs
    subvolumes: Vec<PathBuf>,
    #[serde(skip, default)]
    snapshots: Vec<GroupSnapshot>,
}

impl Group {
    pub fn new(group_name: String, subvolumes: Vec<PathBuf>) -> Self {
        Self {
            group_name,
            subvolumes,
            snapshots: Vec::new(),
        }
    }

    /**
    related_subvolume_path: path of the related subvolume
    e.g. if subvolume path is `archlinux/@home`
    snapshot path should be `tram_btrfs/snapshot_groups/default/manually/2026-04-16_21:26:00/archlinux/@home`
    return: if the snapshot is successfully added
    */
    pub fn add_snapshot<T: AsRef<Path>>(
        &mut self,
        raw_path: T,
        snapshot_type: &str,
        datetime: &str,
        related_subvolume: PathBuf,
    ) -> bool {
        if let Some(snapshot_type) = SnapshotType::get_type(snapshot_type)
            && let Some((date, time)) = datetime.split_once('_')
        {
            // find if the snapshot group has existed
            if let Some(group_snapshot) = self
                .snapshots
                .iter_mut()
                .find(|x| *x == &(date, time, &snapshot_type))
            {
                group_snapshot.add_snapshot(raw_path, related_subvolume);
            } else {
                let mut new_group = GroupSnapshot::new(date, time, snapshot_type);
                new_group.add_snapshot(raw_path, related_subvolume);
                self.snapshots.push(new_group);
            }
            true
        } else {
            false
        }
    }

    /// this function guarantee to only cause `ConfigErrSubvolumeNotExist` error
    /// removed_subvolume: a Vec passed in to store those invalid and removed subvolumes
    pub fn verify_subvolumes(
        &mut self,
        available_subvolumes: &[PathBuf],
        removed_subvolume: &mut Vec<PathBuf>,
    ) {
        let mut i = 0;
        while i < self.subvolumes.len() {
            let crr = self.subvolumes.get(i).unwrap();
            if available_subvolumes.contains(crr) {
                i += 1;
            } else {
                // WARN: here uses `swap_remove`, which won't preserve the original order of subvolumes
                removed_subvolume.push(self.subvolumes.swap_remove(i));
            }
        }
    }

    pub fn _create_snapshot(&mut self) {
        // TODO:
        todo!()
    }

    pub fn rename_group<T: Into<String>>(&mut self, new_name: T) -> AppResult<()> {
        let mut err: Result<_, AppError> = Ok(());
        if !self.snapshots.is_empty() {
            let new_name = new_name.into();
            let new_group_path = PathBuf::from(globals::TOP_DIRECTORY_NAME)
                .join(globals::GROUPS_DIRECTORY_NAME)
                .join(&new_name);
            for x in self.snapshots.iter_mut() {
                // WARN: need test
                err.chain(x.rename_group_snapshot(&new_group_path));
            }
            self.group_name = new_name;
        }
        err
    }
}

impl PartialEq<str> for Group {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.group_name == other
    }
}
