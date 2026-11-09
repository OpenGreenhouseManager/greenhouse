use super::{Error, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use crate::smart_device_common::{Config, Mode, ScriptingApi};
use crate::smart_device_common::{StoredConfig, parse_stored_config};

// Default config file path for backward compatibility
pub(crate) const DEFAULT_CONFIG_FILE_NAME: &str = "./config/config.json";

pub fn update_config_file<T>(config: &Config<T>) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    update_config_file_with_path(config, DEFAULT_CONFIG_FILE_NAME)
}

pub fn read_config_file<T>() -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    read_config_file_with_path(DEFAULT_CONFIG_FILE_NAME)
}

pub fn update_config_file_with_path<T>(config: &Config<T>, config_path: &str) -> Result<()>
where
    T: Serialize + Clone + Default,
{
    let json_string = serde_json::to_string(&config).map_err(|_| Error::IllFormattedConfig)?;
    std::fs::write(config_path, json_string).map_err(|_| Error::MissingConfig)
}

pub fn read_config_file_with_path<T>(config_path: &str) -> Result<Config<T>>
where
    T: DeserializeOwned + Clone + Default,
{
    let data = std::fs::read_to_string(config_path).map_err(|_| Error::MissingConfig)?;
    serde_json::from_str(&data).map_err(|_| Error::IllFormattedConfig)
}

/// Reads the config file at startup. If it no longer matches `Config<T>` (the
/// device's `additional_config` changed shape), the device-level fields
/// (`port`, `datasource_id`, `scripting_api`) are kept, `additional_config` is
/// reset to `T::default()`, and the migrated config is written back, so the
/// device stays activated.
pub(crate) fn load_config_file_with_migration<T>(config_path: &str) -> Result<Config<T>>
where
    T: Serialize + DeserializeOwned + Clone + Default,
{
    let data = std::fs::read(config_path).map_err(|_| Error::MissingConfig)?;
    match parse_stored_config(&data, Config::default()) {
        StoredConfig::Valid(config) => Ok(config),
        StoredConfig::Migrated(config) => {
            tracing::warn!(
                "{config_path}: additional_config does not match this device; \
                 resetting it, keeping port/datasource_id/scripting_api"
            );
            update_config_file_with_path(&config, config_path)?;
            Ok(config)
        }
        StoredConfig::Invalid => Err(Error::IllFormattedConfig),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum TypeOptionDto {
    Number,
    Boolean,
    Object,
    Measurement,
    Stream,
    #[default]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
    struct Current {
        min: i32,
    }

    #[test]
    fn migration_keeps_activation_and_rewrites_file() {
        let path = std::env::temp_dir().join(format!("gh-migrate-{}.json", std::process::id()));
        let path = path.to_str().unwrap();
        std::fs::write(
            path,
            r#"{"port":6001,"datasource_id":"ds","additional_config":{"old":true},"scripting_api":{"url":"u","token":"t"}}"#,
        )
        .unwrap();

        let config: Config<Current> = load_config_file_with_migration(path).unwrap();
        assert_eq!(config.port, 6001);
        assert_eq!(config.datasource_id, "ds");
        assert_eq!(config.additional_config, Current::default());
        assert_eq!(config.scripting_api.unwrap().token, "t");

        // The migrated config was written back and now parses directly.
        let reread: Config<Current> = read_config_file_with_path(path).unwrap();
        assert_eq!(reread.datasource_id, "ds");
        std::fs::remove_file(path).unwrap();
    }
}
