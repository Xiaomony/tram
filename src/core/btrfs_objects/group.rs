use crate::core::btrfs_objects::{
    snapshot::{GroupSnapshot, SnapshotType},
    subvolume::Subvolume,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, rc::Rc};

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    group_name: String,
    // subvolume pathes loaded from configs
    subvolumes_config: Vec<String>,
    #[serde(skip, default)]
    snapshots: Vec<GroupSnapshot>,
}

impl Group {
    pub fn new(group_name: String, subvolumes: Vec<String>) -> Self {
        Self {
            group_name,
            subvolumes_config: subvolumes,
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
        related_subvolume: Rc<Subvolume>,
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
}

impl PartialEq<str> for Group {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.group_name == other
    }
}
