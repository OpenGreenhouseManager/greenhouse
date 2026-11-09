//! Device configuration: the same types (and JSON) on every platform.

use alloc::string::String;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use greenhouse_core::smart_device_dto::activation::ActivateRequestDto;
use greenhouse_core::smart_device_dto::config::{self as dto, ConfigResponseDto, TypeOption};

pub use greenhouse_core::smart_device_dto::config::ScriptingApi;

/// Device configuration as persisted by the device (config file / flash).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T> {
    pub port: u16,
    pub datasource_id: String,
    pub additional_config: T,
    pub scripting_api: Option<ScriptingApi>,
}

/// What the device offers; reported by `GET /config`.
#[derive(Debug, Clone)]
pub(crate) enum Mode {
    /// Can be written (`/write`), e.g. an actuator.
    Input(TypeOption),
    /// Can be read (`/read`), e.g. a sensor.
    Output(TypeOption),
    InputOutput(TypeOption, TypeOption),
}

impl<T: Clone> Config<T> {
    /// This config with `additional_config` replaced; what the default config
    /// interceptor does on `POST /config`.
    pub fn with_additional_config(&self, additional_config: T) -> Self {
        Self {
            port: self.port,
            datasource_id: self.datasource_id.clone(),
            additional_config,
            scripting_api: self.scripting_api.clone(),
        }
    }

    /// The config after `POST /activate`: unchanged apart from the scripting API.
    pub(crate) fn activated(&self, request: ActivateRequestDto) -> Self {
        Self {
            scripting_api: Some(ScriptingApi {
                url: request.url,
                token: request.token,
            }),
            ..self.clone()
        }
    }

    /// The `GET /config` response for a device in `mode`.
    pub(crate) fn to_response_dto(&self, mode: &Mode) -> ConfigResponseDto<T> {
        let (mode, input_type, output_type) = match mode {
            Mode::Input(t) => (dto::Mode::Input, Some(t.clone()), None),
            Mode::Output(t) => (dto::Mode::Output, None, Some(t.clone())),
            Mode::InputOutput(i, o) => (dto::Mode::InputOutput, Some(i.clone()), Some(o.clone())),
        };
        ConfigResponseDto {
            mode,
            input_type,
            output_type,
            scripting_api: self.scripting_api.clone(),
            additional_config: self.additional_config.clone(),
        }
    }
}

/// Result of [`parse_stored_config`].
#[derive(Debug)]
pub(crate) enum StoredConfig<T> {
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
pub(crate) fn parse_stored_config<T>(bytes: &[u8], default: Config<T>) -> StoredConfig<T>
where
    T: DeserializeOwned,
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

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
        let api = c.scripting_api.unwrap();
        assert_eq!(api.url, "https://api");
        assert_eq!(api.token, "tok");
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
    fn with_additional_config_keeps_the_rest() {
        let activated = default_config().activated(ActivateRequestDto {
            url: "u".to_string(),
            token: "t".to_string(),
        });
        let c = activated.with_additional_config(V2 { min: 5, max: 6 });
        assert_eq!(c.port, 6001);
        assert_eq!(c.additional_config, V2 { min: 5, max: 6 });
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
