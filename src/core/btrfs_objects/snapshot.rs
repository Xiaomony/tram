use std::path::PathBuf;

use crate::core::btrfs_objects::subvolume::Subvolume;

#[derive(Debug)]
/// Snapshots of a single subvolume
pub struct SubvolumeSnapshot<'a> {
    path: PathBuf,
    related_subvolume: &'a Subvolume,
}

impl<'a> SubvolumeSnapshot<'a> {
    pub fn new(path: PathBuf, related_subvolume: &'a Subvolume) -> Self {
        Self {
            path,
            related_subvolume,
        }
    }
}

#[derive(Debug)]
enum SnapshotType {
    Manually,
    Daily,
    Monthly,
    Weekly,
}

#[derive(Debug)]
/// Snapshots of a group, consists of snapshots of subvolumes in that group
pub struct GroupSnapshot<'a> {
    _subvolume_snapshots: Vec<SubvolumeSnapshot<'a>>,
    _date: String,
    _time: String,
    snapshot_type: SnapshotType,
}

impl<'a> GroupSnapshot<'a> {
    pub fn new() -> Self {
        Self {
            _date: String::new(),
            _time: String::new(),
            _subvolume_snapshots: Vec::new(),
            snapshot_type: SnapshotType::Manually,
        }
    }
}
