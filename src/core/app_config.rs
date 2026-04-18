use crate::{
    core::{btrfs_objects::snapshot_group::SnapshotGroup, error::AppResult},
    globals,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig<'a> {
    pub snapshot_groups: Vec<SnapshotGroup<'a>>,
    #[serde(skip, default)]
    _first_time_launch: bool,
}

impl<'a> AppConfig<'a> {
    pub fn load_config() -> AppResult<AppConfig<'a>> {
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
                snapshot_groups: vec![
                    SnapshotGroup::new(
                        "default".to_string(),
                        vec!["@".to_string(), "something else".to_string()],
                    ),
                    SnapshotGroup::new(
                        "default2".to_string(),
                        vec!["@home".to_string(), "something else2".to_string()],
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
