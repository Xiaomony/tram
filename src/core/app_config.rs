use crate::{core::error::AppResult, globals};
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};

#[derive(Deserialize, Serialize)]
struct SnapshotGroupConfig {
    group_name: String,
    // a list of pathes to subvolumes
    subvolumes: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct AppConfig {
    groups: Vec<SnapshotGroupConfig>,
    #[serde(skip, default)]
    first_time_launch: bool,
}

impl AppConfig {
    pub fn load_config() -> AppResult<AppConfig> {
        create_dir_all(&*globals::CONFIG_DIR)?;
        let config_file_path = &*globals::MAIN_CONFIG_FILE_PATH;
        if fs::exists(config_file_path)? {
            let s = std::fs::read_to_string(config_file_path)?;
            Ok(toml::from_str::<AppConfig>(&s)?)
        } else {
            let config = Self {
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
    pub fn write_config(&self) -> AppResult<()> {
        std::fs::write(&*globals::MAIN_CONFIG_FILE_PATH, toml::to_string(self)?)?;
        Ok(())
    }
}
