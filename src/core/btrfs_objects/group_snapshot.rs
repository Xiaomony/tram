use crate::core::btrfs_objects::snapshot_type::SnapshotType;
use crate::core::btrfs_objects::subvolume_snapshot::SubvolumeSnapshot;
use crate::core::error::{AppError, AppResult, ExtendResult};
use crate::core::utils::exec_command;
use crate::globals;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

#[derive(Debug)]
/// Snapshots of a group
/// consists of snapshots of subvolumes in that group
/// also store infomations like date, time, type(Manually, Daily, Monthly, Weekly)
pub struct GroupSnapshot {
    subvolume_snapshots: Vec<SubvolumeSnapshot>,
    date: String,
    time: String,
    snapshot_type: SnapshotType,
}

impl GroupSnapshot {
    pub fn new<T: Into<String>>(date: T, time: T, snapshot_type: SnapshotType) -> Self {
        Self {
            date: date.into(),
            time: time.into(),
            subvolume_snapshots: Vec::new(),
            snapshot_type,
        }
    }

    /// record a snapshot when loading configuration
    pub fn add_snapshot<T: AsRef<Path>, E: Into<PathBuf>>(
        &mut self,
        full_path: T,
        related_subvolume: E,
    ) {
        self.subvolume_snapshots.push(SubvolumeSnapshot::new(
            full_path.as_ref().to_path_buf(),
            Some(related_subvolume.into()),
        ));
    }

    pub fn delete(self, group_name: &str) -> AppResult<()> {
        let fullpaths = self
            .subvolume_snapshots
            .iter()
            .map(|x| x.get_fullpath_string());
        let args: Vec<String> = ["subvolume".to_string(), "delete".to_string()]
            .into_iter()
            .chain(fullpaths)
            .collect();
        exec_command("btrfs", args)?;

        // remove the directory of the current snapshot group
        let group_snapshot_fullpath = globals::SNAPSHOT_GROUP_DIR_PATH
            .join(group_name)
            .join(self.snapshot_type.as_ref())
            .join(self.date + "_" + &self.time);
        remove_dir_all(group_snapshot_fullpath)?;
        Ok(())
    }

    /// new_group_path: new path to group, not containing mount point
    /// e.g. when renaming group name from `default` to `new_group_name`
    /// and there're snapshots under `/run/tram_btrfs/tram_btrfs/snapshot_groups/default/manually/2026-04-16_21:26:00/`
    /// and this parameter should be `tram_btrfs/snapshot_groups/new_group_name/`
    pub fn rename_group_snapshot<T: AsRef<Path>>(&mut self, new_group_path: T) -> AppResult<()> {
        // path to /run/tram_btrfs/snapshot_group
        let new_group_snapshot_path = new_group_path
            .as_ref()
            .join(self.snapshot_type.as_ref())
            .join(format!("{}_{}", self.date, self.time));

        let mut err: Result<_, AppError> = Ok(());
        for x in self.subvolume_snapshots.iter_mut() {
            // WARN: need test
            err.chain(x.move_snapshot(&new_group_snapshot_path));
        }
        err
    }

    #[inline]
    pub fn get_type(&self) -> SnapshotType {
        self.snapshot_type
    }

    #[inline]
    pub fn get_date(&self) -> String {
        self.date.clone()
    }

    #[inline]
    pub fn get_time(&self) -> String {
        self.time.clone()
    }

    #[inline]
    /// returns a string containing all valid snapshots in the form like:
    /// "@  @home"
    pub fn get_snapshoted_subvolumes(&self) -> Vec<&str> {
        self.subvolume_snapshots
            .iter()
            .filter_map(|x| x.get_relate_subvolume_path())
            .collect()
    }
}

impl PartialEq<(&str, &str, &SnapshotType)> for GroupSnapshot {
    #[inline]
    /// test the equality of GroupSnapshot and (date, time, snapshot_type)
    fn eq(&self, other: &(&str, &str, &SnapshotType)) -> bool {
        self.date == other.0 && self.time == other.1 && self.snapshot_type.eq(other.2)
    }
}
