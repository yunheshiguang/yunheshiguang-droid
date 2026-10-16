use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct CopyBlock {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Deserialize)]
struct LocaleBundle {
    disclaimer: CopyBlock,
}

static EN_US: OnceLock<LocaleBundle> = OnceLock::new();
static ZH_CN: OnceLock<LocaleBundle> = OnceLock::new();

fn load_bundle(json: &'static str) -> LocaleBundle {
    serde_json::from_str(json).expect("workspace locale bundle is invalid")
}

fn normalize_locale(value: &str) -> &'static str {
    if value.to_ascii_lowercase().starts_with("zh") {
        "zh-CN"
    } else {
        "en-US"
    }
}

pub fn preferred_locale() -> &'static str {
    sys_locale::get_locale().map_or("en-US", |locale| normalize_locale(locale.trim()))
}

pub fn disclaimer(locale: &str) -> &'static CopyBlock {
    match normalize_locale(locale) {
        "zh-CN" => {
            &ZH_CN
                .get_or_init(|| load_bundle(include_str!("i18n/zh-CN.json")))
                .disclaimer
        }
        _ => {
            &EN_US
                .get_or_init(|| load_bundle(include_str!("i18n/en-US.json")))
                .disclaimer
        }
    }
}
