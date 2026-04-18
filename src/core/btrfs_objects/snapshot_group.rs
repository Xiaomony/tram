use serde::{Deserialize, Serialize};

use crate::core::{btrfs_objects::snapshot::GroupSnapshot, error::AppResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct SnapshotGroup<'a> {
    group_name: String,
    // subvolume pathes loaded from configs
    subvolumes_config: Vec<String>,
    #[serde(skip, default)]
    snapshots: Vec<GroupSnapshot<'a>>,
}

impl SnapshotGroup<'_> {
    pub fn new(group_name: String, subvolumes: Vec<String>) -> Self {
        Self {
            group_name,
            subvolumes_config: subvolumes,
            snapshots: Vec::new(),
        }
    }
    pub fn add_snapshot(
        &self,
        snapshot_type: &str,
        datatime: &str,
        subvolume: &str,
    ) -> AppResult<()> {
        todo!()
    }
}

impl PartialEq<str> for SnapshotGroup<'_> {
    fn eq(&self, other: &str) -> bool {
        self.group_name == other
    }
}
