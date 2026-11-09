//! Transport-agnostic smart-device types and logic, shared by the Linux runtime
//! (`smart_device_interface`, axum) and the ESP32 firmware (`greenhouse_esp32`,
//! embassy). `no_std` + `alloc`: keep anything runtime-specific out of here.

use alloc::string::String;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};
use crate::smart_device_dto::activation::ActivateRequestDto;
use crate::smart_device_dto::config::{self as dto, ConfigResponseDto, TypeOption};

/// Device configuration as persisted by the device (config file / flash).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T>
where
    T: Clone + Default,
{
    pub port: u16,
    pub datasource_id: String,
    pub additional_config: T,
    pub scripting_api: Option<ScriptingApi>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}

/// What the device offers; reported by `GET /config`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Mode {
    Input(TypeOption),
    Output(TypeOption),
    InputOutput(TypeOption, TypeOption),
    #[default]
    Unknown,
}

/// An alert raised by a device handler; see `trigger_alert` of each runtime.
pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

impl AlertCreation {
    /// The request body for `POST {scripting_api.url}/alert`.
    pub fn into_dto<T: Clone + Default>(self, config: &Config<T>) -> CreateAlertDto {
        CreateAlertDto {
            severity: self.severity,
            identifier: self.identifier,
            value: self.value,
            note: self.note,
            datasource_id: config.datasource_id.clone(),
        }
    }
}

impl<T: Clone + Default> Config<T> {
    /// The config after `POST /activate`: unchanged apart from the scripting API.
    pub fn activated(&self, request: ActivateRequestDto) -> Self {
        Self {
            scripting_api: Some(ScriptingApi {
                url: request.url,
                token: request.token,
            }),
            ..self.clone()
        }
    }

    /// The `GET /config` response for a device in `mode`.
    pub fn to_response_dto(&self, mode: &Mode) -> ConfigResponseDto<T> {
        let (mode, input_type, output_type) = match mode {
            Mode::Input(t) => (dto::Mode::Input, Some(t.clone()), None),
            Mode::Output(t) => (dto::Mode::Output, None, Some(t.clone())),
            Mode::InputOutput(i, o) => (dto::Mode::InputOutput, Some(i.clone()), Some(o.clone())),
            Mode::Unknown => (dto::Mode::Unknown, None, None),
        };
        ConfigResponseDto {
            mode,
            input_type,
            output_type,
            scripting_api: self.scripting_api.as_ref().map(|s| dto::ScriptingApi {
                url: s.url.clone(),
                token: s.token.clone(),
            }),
            additional_config: self.additional_config.clone(),
        }
    }
}

/// Result of [`parse_stored_config`].
#[derive(Debug)]
pub enum StoredConfig<T: Clone + Default> {
    /// The stored config matches the current `Config<T>`.
    Valid(Config<T>),
    /// The stored config is valid JSON but no longer matches `Config<T>` (e.g. a
    /// firmware update changed `additional_config`). Holds `default` with the
    /// device-level fields (`port`, `datasource_id`, `scripting_api`) taken from
    /// the stored config, so the device stays activated. Persist it.
    Migrated(Config<T>),
    /// Not a JSON object; nothing could be kept.
    Invalid,
}

/// The device-level fields of any stored config; everything else is ignored.
#[derive(Deserialize)]
struct StoredEnvelope {
    port: Option<u16>,
    datasource_id: Option<String>,
    scripting_api: Option<ScriptingApi>,
}

/// Parses a stored config, keeping the device-level fields if
/// `additional_config` no longer deserializes into `T`.
pub fn parse_stored_config<T>(bytes: &[u8], default: Config<T>) -> StoredConfig<T>
where
    T: Clone + Default + DeserializeOwned,
{
    if let Ok(config) = serde_json::from_slice(bytes) {
        return StoredConfig::Valid(config);
    }
    let Ok(stored) = serde_json::from_slice::<StoredEnvelope>(bytes) else {
        return StoredConfig::Invalid;
    };
    StoredConfig::Migrated(Config {
        port: stored.port.unwrap_or(default.port),
        datasource_id: stored.datasource_id.unwrap_or(default.datasource_id),
        additional_config: default.additional_config,
        scripting_api: stored.scripting_api.or(default.scripting_api),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
    struct V2 {
        min: i32,
        max: i32,
    }

    fn default_config() -> Config<V2> {
        Config {
            port: 6001,
            datasource_id: "default-id".to_string(),
            additional_config: V2 { min: 0, max: 100 },
            scripting_api: None,
        }
    }

    #[test]
    fn stored_config_valid() {
        let json = br#"{"port":1,"datasource_id":"d","additional_config":{"min":1,"max":2},"scripting_api":null}"#;
        let StoredConfig::Valid(c) = parse_stored_config(json, default_config()) else {
            panic!("expected Valid");
        };
        assert_eq!(c.port, 1);
        assert_eq!(c.additional_config, V2 { min: 1, max: 2 });
    }

    #[test]
    fn stored_config_migrated_keeps_activation() {
        // `additional_config` from an older firmware with a different shape.
        let json = br#"{"port":7000,"datasource_id":"stored-id","additional_config":{"threshold":5},"scripting_api":{"url":"https://api","token":"tok"}}"#;
        let StoredConfig::Migrated(c) = parse_stored_config(json, default_config()) else {
            panic!("expected Migrated");
        };
        assert_eq!(c.port, 7000);
        assert_eq!(c.datasource_id, "stored-id");
        assert_eq!(c.additional_config, V2 { min: 0, max: 100 });
        assert_eq!(
            c.scripting_api,
            Some(ScriptingApi {
                url: "https://api".to_string(),
                token: "tok".to_string()
            })
        );
    }

    #[test]
    fn stored_config_migrated_missing_fields_use_default() {
        let StoredConfig::Migrated(c) = parse_stored_config(br#"{"foo":1}"#, default_config())
        else {
            panic!("expected Migrated");
        };
        assert_eq!(c.port, 6001);
        assert_eq!(c.datasource_id, "default-id");
        assert!(c.scripting_api.is_none());
    }

    #[test]
    fn stored_config_invalid() {
        assert!(matches!(
            parse_stored_config(b"\xff\xff", default_config()),
            StoredConfig::Invalid
        ));
    }

    #[test]
    fn activated_only_changes_scripting_api() {
        let c = default_config().activated(ActivateRequestDto {
            url: "u".to_string(),
            token: "t".to_string(),
        });
        assert_eq!(c.port, 6001);
        assert_eq!(c.additional_config, V2 { min: 0, max: 100 });
        assert_eq!(c.scripting_api.unwrap().token, "t");
    }

    #[test]
    fn response_dto_types_follow_mode() {
        let r = default_config().to_response_dto(&Mode::Output(TypeOption::Number));
        assert!(matches!(r.mode, dto::Mode::Output));
        assert!(r.input_type.is_none());
        assert!(matches!(r.output_type, Some(TypeOption::Number)));
    }
}
