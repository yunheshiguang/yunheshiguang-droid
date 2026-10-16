use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};
use yunheshiguang_contract::KnownEnum;

use serde::{Deserialize, Serialize};

use crate::workspace::contract::{AssistantSettings, StructuredOutput};
use crate::workspace::local_state::{read_json, write_json};

pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const APP_KEY_VARIABLE: &str = "YUNHESHIGUANG_AI_API_KEY";
const OPENAI_KEY_VARIABLE: &str = "OPENAI_API_KEY";
const OPENAI_HOST: &str = "api.openai.com";

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Credentials {
    api_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub structured_output: StructuredOutput,
}

struct Stored {
    settings: AssistantSettings,
    api_key: Option<String>,
}

pub struct SettingsStore {
    directory: Option<PathBuf>,
    stored: Mutex<Stored>,
    app_key: Option<String>,
    openai_key: Option<String>,
}

fn defaults() -> AssistantSettings {
    AssistantSettings {
        base_url: DEFAULT_BASE_URL.to_string(),
        structured_output: StructuredOutput::JsonSchema.into(),
        ..AssistantSettings::default()
    }
}

fn present(key: String) -> Option<String> {
    let key = key.trim().to_string();

    (!key.is_empty()).then_some(key)
}

fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok().and_then(present)
}

fn parsed(base_url: &str) -> Option<reqwest::Url> {
    reqwest::Url::parse(base_url.trim()).ok()
}

fn same_service(before: &str, after: &str) -> bool {
    match (parsed(before), parsed(after)) {
        (Some(before), Some(after)) => before.origin() == after.origin(),
        _ => before.trim() == after.trim(),
    }
}

fn is_openai(base_url: &str) -> bool {
    parsed(base_url).is_some_and(|url| {
        url.host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case(OPENAI_HOST))
    })
}

impl SettingsStore {
    pub fn load(directory: PathBuf) -> Self {
        let settings = read_json::<AssistantSettings>(&directory.join("settings.json"))
            .unwrap_or_else(defaults);
        let api_key = read_json::<Credentials>(&directory.join("credentials.json"))
            .and_then(|credentials| present(credentials.api_key));

        Self {
            directory: Some(directory),
            stored: Mutex::new(Stored { settings, api_key }),
            app_key: environment(APP_KEY_VARIABLE),
            openai_key: environment(OPENAI_KEY_VARIABLE),
        }
    }

    #[cfg(test)]
    pub fn in_memory(app_key: Option<&str>) -> Self {
        Self::in_memory_with(app_key, None)
    }

    #[cfg(test)]
    pub fn in_memory_with(app_key: Option<&str>, openai_key: Option<&str>) -> Self {
        Self {
            directory: None,
            stored: Mutex::new(Stored {
                settings: defaults(),
                api_key: None,
            }),
            app_key: app_key.map(str::to_string),
            openai_key: openai_key.map(str::to_string),
        }
    }

    fn environment_key(&self, base_url: &str) -> Option<String> {
        self.app_key.clone().or_else(|| {
            is_openai(base_url)
                .then(|| self.openai_key.clone())
                .flatten()
        })
    }

    fn stored(&self) -> std::sync::MutexGuard<'_, Stored> {
        self.stored.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn settings(&self) -> AssistantSettings {
        let stored = self.stored();

        AssistantSettings {
            api_key_set: stored.api_key.is_some()
                || self.environment_key(&stored.settings.base_url).is_some(),
            ..stored.settings.clone()
        }
    }

    pub fn endpoint(&self) -> Endpoint {
        let stored = self.stored();

        Endpoint {
            base_url: stored.settings.base_url.trim().to_string(),
            model: stored.settings.model.trim().to_string(),
            api_key: stored
                .api_key
                .clone()
                .or_else(|| self.environment_key(&stored.settings.base_url)),
            structured_output: StructuredOutput::known(stored.settings.structured_output)
                .unwrap_or_default(),
        }
    }

    pub fn save(&self, settings: AssistantSettings, api_key: Option<String>) -> io::Result<()> {
        let settings = AssistantSettings {
            api_key_set: false,
            ..settings
        };
        let mut stored = self.stored();
        let api_key = match api_key {
            Some(key) => present(key),
            None if same_service(&stored.settings.base_url, &settings.base_url) => {
                stored.api_key.clone()
            }
            None => None,
        };

        if let Some(directory) = &self.directory {
            write_json(
                &directory.join("credentials.json"),
                &Credentials {
                    api_key: api_key.clone().unwrap_or_default(),
                },
            )?;
            write_json(&directory.join("settings.json"), &settings)?;
        }

        *stored = Stored { settings, api_key };
        Ok(())
    }
}
