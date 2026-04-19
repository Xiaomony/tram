use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::core::{btrfs_objects::subvolume::Subvolume, utils::get_current_date_time};

#[derive(Debug)]
/// Snapshots of a single subvolume
pub struct SubvolumeSnapshot {
    _path: PathBuf,
    _related_subvolume: Option<Rc<Subvolume>>,
}

impl SubvolumeSnapshot {
    pub fn new<T: Into<PathBuf>>(path: T, related_subvolume: Option<Rc<Subvolume>>) -> Self {
        Self {
            _path: path.into(),
            _related_subvolume: related_subvolume,
        }
    }
}

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
    pub fn new_current_time(snapshot_type: SnapshotType) -> Self {
        let (date, time) = get_current_date_time();
        Self {
            date,
            time,
            snapshot_type,
            subvolume_snapshots: Vec::new(),
        }
    }

    pub fn add_snapshot<T: AsRef<Path>>(&mut self, full_path: T, related_subvolume: Rc<Subvolume>) {
        self.subvolume_snapshots.push(SubvolumeSnapshot::new(
            full_path.as_ref().to_path_buf(),
            Some(related_subvolume),
        ));
    }
}

impl PartialEq<(&str, &str, &SnapshotType)> for GroupSnapshot {
    #[inline]
    /// test the equality of GroupSnapshot and (date, time, snapshot_type)
    fn eq(&self, other: &(&str, &str, &SnapshotType)) -> bool {
        self.date == other.0 && self.time == other.1 && self.snapshot_type.eq(other.2)
    }
}
