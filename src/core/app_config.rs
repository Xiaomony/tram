use crate::core::btrfs_objects::group::Group;
use crate::core::error::{CResult, throw_invalid_index};
use crate::globals;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AutoSnapshotSchedule {
    pub daily_max: usize,
    pub weekly_max: usize,
    pub monthly_max: usize,
    pub boot_max: usize,
}
impl AutoSnapshotSchedule {
    #[inline]
    pub fn new_default() -> Self {
        Self {
            daily_max: 0,
            monthly_max: 0,
            weekly_max: 0,
            boot_max: 0,
        }
    }
    #[inline]
    pub fn _change_daily(&mut self, new: usize) {
        self.daily_max = new;
    }
    #[inline]
    pub fn _change_weekly(&mut self, new: usize) {
        self.weekly_max = new;
    }
    #[inline]
    pub fn _change_monthly(&mut self, new: usize) {
        self.monthly_max = new;
    }
    #[inline]
    pub fn _change_boot(&mut self, new: usize) {
        self.boot_max = new;
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    schedule: AutoSnapshotSchedule,
    pub groups: Vec<Group>,
    #[serde(skip, default)]
    first_time_launch: bool,
}

impl AppConfig {
    pub fn load_config() -> CResult<AppConfig> {
        create_dir_all(&*globals::CONFIG_DIR)?;
        let config_file_path = &*globals::MAIN_CONFIG_FILE_PATH;
        if fs::exists(config_file_path)? {
            let s = std::fs::read_to_string(config_file_path)?;
            Ok(toml::from_str::<AppConfig>(&s)?)
        } else {
            let config = Self {
                schedule: AutoSnapshotSchedule::new_default(),
                groups: Vec::new(),
                first_time_launch: true,
            };
            config.write_config()?;
            Ok(config)
        }
    }

    #[inline]
    pub fn is_first_time_launch(&self) -> bool {
        self.first_time_launch
    }

    #[inline]
    pub fn write_config(&self) -> CResult<()> {
        std::fs::write(&*globals::MAIN_CONFIG_FILE_PATH, toml::to_string(self)?)?;
        Ok(())
    }

    #[inline]
    pub fn add_new_group<T: Into<String>>(
        &mut self,
        group_name: T,
        subvolumes: Vec<PathBuf>,
    ) -> CResult<()> {
        self.groups.push(Group::new(group_name.into(), subvolumes));
        self.write_config()
    }

    #[inline]
    pub fn rename_group<T: Into<String>>(&mut self, index: usize, new_name: T) -> CResult<()> {
        let Some(group) = self.groups.get_mut(index) else {
            return throw_invalid_index(index, "renaming group");
        };
        group.rename_group(new_name)?;
        self.write_config()
    }

    #[inline]
    pub fn get_schedule(&self) -> AutoSnapshotSchedule {
        self.schedule
    }

    #[inline]
    pub fn change_schedule(&mut self, new_schedule: AutoSnapshotSchedule) -> CResult<()> {
        self.schedule = new_schedule;
        self.write_config()
    }
}

impl Drop for AppConfig {
    fn drop(&mut self) {
        let _ = self.write_config();
    }
}
