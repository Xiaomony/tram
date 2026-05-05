use crate::core::error::{AppError, AppResult};
use crate::core::utils::{exec_command, mount_point_join};
use crate::globals;
use std::path::{Path, PathBuf};

#[derive(Debug)]
/// Snapshots of a single subvolume
pub struct SubvolumeSnapshot {
    path: PathBuf,
    related_subvolume: Option<PathBuf>,
}

impl SubvolumeSnapshot {
    pub fn new<T: Into<PathBuf>>(path: T, related_subvolume: Option<PathBuf>) -> Self {
        Self {
            path: path.into(),
            related_subvolume,
        }
    }

    #[inline]
    pub fn get_fullpath(&self) -> PathBuf {
        mount_point_join(&self.path)
    }

    #[inline]
    pub fn get_fullpath_string(&self) -> String {
        self.get_fullpath().to_string_lossy().into()
    }

    #[inline]
    pub fn get_relate_subvolume_path(&self) -> Option<&str> {
        // TODO: may need to handle error and report it as a bug if `related_subvolume` is None
        self.related_subvolume.as_ref().and_then(|x| x.to_str())
    }

    /// new_group_path: the new path of the group snapshot, not containing mount point
    /// e.g. when renaming group name from `default` to `new_group_name`
    /// and for example,  the snapshot should be moved from
    /// `/run/tram_btrfs/tram_btrfs/snapshot_groups/default/manually/2026-04-16_21:26:00/@`
    /// to `/run/tram_btrfs/tram_btrfs/snapshot_groups/new_group_name/manually/2026-04-16_21:26:00/@`
    /// and this parameter should be `tram_btrfs/snapshot_groups/new_group_name/manually/2026-04-16_21:26:00`
    pub fn move_snapshot<T: AsRef<Path>>(&mut self, new_group_snapshot_path: T) -> AppResult<()> {
        let mount_point = PathBuf::from(globals::MOUNT_POINT);
        let oldpath = mount_point.join(&self.path);
        // erase its readonly property
        exec_command(
            "btrfs",
            [
                "property",
                "set",
                "-t",
                "subvol",
                oldpath.to_string_lossy().as_ref(),
                "ro",
                "false",
            ],
        )?;

        let Some(subvol_path) = self.related_subvolume.as_ref() else {
            return Err(AppError::Bug(format!(
                "No related subvolume when moving snapshot:\n\tfrom: {:?}\n\tto: {:?}",
                self.related_subvolume,
                new_group_snapshot_path.as_ref()
            )));
        };
        let newpath = mount_point.join(new_group_snapshot_path).join(subvol_path);
        std::fs::create_dir_all(&newpath)?;
        std::fs::rename(oldpath, &newpath)?;

        // set the moved snapshot to readonly
        exec_command(
            "btrfs",
            [
                "property",
                "set",
                "-t",
                "subvol",
                newpath.to_string_lossy().as_ref(),
                "ro",
                "true",
            ],
        )?;

        Ok(())
    }
}
