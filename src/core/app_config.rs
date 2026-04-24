use crate::core::{btrfs_objects::group::Group, error::AppResult};
use crate::globals;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};

#[derive(Debug, Deserialize, Serialize)]
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
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub schedule: AutoSnapshotSchedule,
    pub groups: Vec<Group>,
    #[serde(skip, default)]
    _first_time_launch: bool,
}

impl AppConfig {
    pub fn load_config() -> AppResult<AppConfig> {
        create_dir_all(&*globals::CONFIG_DIR)?;
        let config_file_path = &*globals::MAIN_CONFIG_FILE_PATH;
        if fs::exists(config_file_path)? {
            let s = std::fs::read_to_string(config_file_path)?;
            // TEST: test toml serilize
            let a = toml::from_str::<AppConfig>(&s)?;
            println!("{a:?}");
            Ok(a)
            // Ok(toml::from_str::<AppConfig>(&s)?)
        } else {
            // TEST: test toml serilize
            let config = Self {
                schedule: AutoSnapshotSchedule::new_default(),
                groups: vec![
                    Group::new(
                        "default".to_string(),
                        vec!["@".into(), "something else".into()],
                    ),
                    Group::new(
                        "default2".to_string(),
                        vec!["@home".into(), "something else2".into()],
                    ),
                ],
                _first_time_launch: true,
            };
            config.write_config()?;
            Ok(config)
        }
    }

    #[inline]
    pub fn _is_first_time_launch(&self) -> bool {
        self._first_time_launch
    }

    #[inline]
    pub fn write_config(&self) -> AppResult<()> {
        std::fs::write(&*globals::MAIN_CONFIG_FILE_PATH, toml::to_string(self)?)?;
        Ok(())
    }
}
