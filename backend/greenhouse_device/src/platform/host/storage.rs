//! The config as a JSON file.

use std::io::ErrorKind;
use std::path::PathBuf;

use crate::error::{Error, Result};

pub(crate) struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Returns the stored config bytes, or `None` if there is no config file.
    pub(crate) fn load(&mut self) -> Result<Option<Vec<u8>>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => {
                log::error!("reading {}: {e}", self.path.display());
                Err(Error::Storage)
            }
        }
    }

    /// Writes the config file in place (no temp file + rename, so a config
    /// file bind-mounted on its own into a container stays writable).
    pub(crate) fn store(&mut self, data: &[u8]) -> Result<()> {
        let write = || -> std::io::Result<()> {
            if let Some(parent) = self.path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&self.path, data)
        };
        write().map_err(|e| {
            log::error!("writing {}: {e}", self.path.display());
            Error::Storage
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{Config, Device, DeviceBuilder, Error, Platform, StatusCode};
    use greenhouse_core::smart_device_dto::Type;
    use greenhouse_core::smart_device_dto::config::TypeOption;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Current {
        min: i32,
    }

    fn default_config() -> Config<Current> {
        Config {
            port: 6001,
            datasource_id: "default".into(),
            additional_config: Current { min: 3 },
            scripting_api: None,
        }
    }

    fn device(path: &std::path::Path) -> Device<Current> {
        DeviceBuilder::new(Platform::new(path), default_config())
            .write(TypeOption::Number, |_: Type, _| async { StatusCode::OK })
            .build()
            .unwrap()
    }

    #[test]
    fn build_without_handler_fails() {
        let path = temp_path("no-handler");
        let result = DeviceBuilder::new(Platform::new(&path), default_config()).build();
        assert!(matches!(result, Err(Error::MissingHandler)));
        assert!(!path.exists(), "nothing is stored without a handler");
    }

    #[tokio::test]
    async fn default_status_and_config_interceptor() {
        let path = temp_path("defaults");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
        let device = device(&path);

        let reply =
            crate::handler::post_config(&device.state, br#"{"additional_config":{"min":9}}"#).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(
            device.config_handle().get().additional_config,
            Current { min: 9 }
        );

        let reply = crate::handler::status(&device.state).await;
        let body: serde_json::Value = serde_json::from_slice(&reply.body.unwrap()).unwrap();
        assert_eq!(body["datasource_id"], "default");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!("gh-device-{name}-{}", std::process::id()))
            .join("config.json")
    }

    #[test]
    fn missing_file_is_created_with_default() {
        let path = temp_path("create");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());

        let device = device(&path);
        assert_eq!(device.config_handle().get().datasource_id, "default");
        let stored: Config<Current> =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(stored.additional_config, Current { min: 3 });
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn migration_keeps_activation_and_rewrites_file() {
        let path = temp_path("migrate");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"{"port":6005,"datasource_id":"ds","additional_config":{"old":true},"scripting_api":{"url":"u","token":"t"}}"#,
        )
        .unwrap();

        let config = device(&path).config_handle().get();
        assert_eq!(config.port, 6005);
        assert_eq!(config.datasource_id, "ds");
        assert_eq!(config.additional_config, Current { min: 3 });
        assert_eq!(config.scripting_api.as_ref().unwrap().token, "t");

        // The migrated config was written back and now parses directly.
        let reread: Config<Current> =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(reread.datasource_id, "ds");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
