use serde::Deserialize;

use super::build_info::is_commit;

pub const MANIFEST: &str = "release.json";
const SCHEMA_VERSION: u64 = 1;
const MAX_ASSETS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub channel: String,
    pub tag: String,
    pub version: String,
    pub commit: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Asset {
    pub name: String,
    pub os: String,
    pub arch: String,
    pub kind: String,
    pub size: u64,
    pub sha256: String,
    pub url: String,
}

#[derive(Deserialize)]
struct Document {
    schema_version: u64,
    channel: String,
    tag: String,
    version: String,
    commit: String,
    #[serde(default)]
    assets: Vec<serde_json::Value>,
}

pub fn parse(body: &[u8], repository: &str) -> Option<Manifest> {
    let document: Document = serde_json::from_slice(body).ok()?;
    let valid = document.schema_version == SCHEMA_VERSION
        && matches!(document.channel.as_str(), "nightly" | "release")
        && is_label(&document.tag, b"._-")
        && is_label(&document.version, b".+-")
        && is_commit(&document.commit);
    if !valid {
        return None;
    }

    let downloads = format!(
        "https://github.com/{repository}/releases/download/{}/",
        document.tag
    );
    let assets = document
        .assets
        .into_iter()
        .filter_map(|value| serde_json::from_value::<Asset>(value).ok())
        .filter(|asset| asset.is_valid(&downloads))
        .take(MAX_ASSETS)
        .collect();

    Some(Manifest {
        channel: document.channel,
        tag: document.tag,
        version: document.version,
        commit: document.commit,
        assets,
    })
}

impl Asset {
    fn is_valid(&self, downloads: &str) -> bool {
        self.name
            .strip_prefix("yunheshiguang-")
            .is_some_and(|rest| is_label(rest, b"._-"))
            && is_token(&self.os)
            && is_token(&self.arch)
            && is_token(&self.kind)
            && self.size > 0
            && self.sha256.len() == 64
            && self
                .sha256
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            && self.url.strip_prefix(downloads) == Some(self.name.as_str())
    }
}

fn is_label(value: &str, punctuation: &[u8]) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || punctuation.contains(&byte))
}

fn is_token(value: &str) -> bool {
    (1..=32).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}
