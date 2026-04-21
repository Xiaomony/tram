use crate::core::btrfs_objects::subvolume_snapshot::SubvolumeSnapshot;
use crate::core::error::{AppError, AppResult, ExtendResult};
use crate::core::utils::{exec_command, get_current_date_time};
use crate::globals;
use std::fmt::Display;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum SnapshotType {
    Manually,
    Daily,
    Monthly,
    Weekly,
}

impl SnapshotType {
    pub fn get_type(string: &str) -> Option<Self> {
        match string {
            "Manually" | "manually" => Some(SnapshotType::Manually),
            "Daily" | "daily" => Some(SnapshotType::Daily),
            "Monthly" | "monthly" => Some(SnapshotType::Monthly),
            "Weekly" | "weekly" => Some(SnapshotType::Weekly),
            _ => None,
        }
    }
}

impl Display for SnapshotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Manually => "manually",
                Self::Daily => "daily",
                Self::Weekly => "weekly",
                Self::Monthly => "monthly",
            }
        )
    }
}

impl AsRef<str> for SnapshotType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Manually => "manually",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }
}

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
    /// create a new object with current date and time
    fn _new_at_current_time(snapshot_type: SnapshotType) -> Self {
        let (date, time) = get_current_date_time();
        Self {
            date,
            time,
            snapshot_type,
            subvolume_snapshots: Vec::new(),
        }
    }

    /// record a snapshot when loading configuration
    pub fn add_snapshot<T: AsRef<Path>>(&mut self, full_path: T, related_subvolume: PathBuf) {
        self.subvolume_snapshots.push(SubvolumeSnapshot::new(
            full_path.as_ref().to_path_buf(),
            Some(related_subvolume),
        ));
    }

    pub fn _delete(self) -> AppResult<()> {
        let fullpaths = self.subvolume_snapshots.iter().map(|x| x.get_fullpath());
        let args: Vec<String> = ["subvolume".to_string(), "delete".to_string()]
            .into_iter()
            .chain(fullpaths)
            .collect();
        exec_command("btrfs", &args)?;
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
}

impl PartialEq<(&str, &str, &SnapshotType)> for GroupSnapshot {
    #[inline]
    /// test the equality of GroupSnapshot and (date, time, snapshot_type)
    fn eq(&self, other: &(&str, &str, &SnapshotType)) -> bool {
        self.date == other.0 && self.time == other.1 && self.snapshot_type.eq(other.2)
    }
}
