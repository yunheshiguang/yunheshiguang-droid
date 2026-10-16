use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use yunheshiguang_sdk::api::{ApiAuthenticator, ApiAuthenticatorInfo, ApiIdentity};
use yunheshiguang_sdk::text::ui_text;

use crate::config;
use crate::messages::*;
use crate::status::FileState;

pub const API_KEYS_AUTHENTICATOR_ID: &str = "reference.api-keys";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeysFile {
    keys: Vec<KeyEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyEntry {
    name: String,
    #[serde(default)]
    roles: Vec<String>,
    sha256: String,
}

fn parse_hex(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }

    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(digest)
}

pub struct EnterpriseApiKeys {
    keys: Vec<(ApiIdentity, [u8; 32])>,
    state: FileState,
}

impl EnterpriseApiKeys {
    pub fn load(path: &Path) -> Self {
        let parsed = config::read_json::<KeysFile>(path).and_then(|file| {
            file.map(|file| {
                file.keys
                    .into_iter()
                    .map(|entry| {
                        parse_hex(&entry.sha256)
                            .map(|digest| {
                                let identity = ApiIdentity::new(entry.name.trim())
                                    .with_roles(entry.roles.clone());
                                let identity = if entry.roles.is_empty() {
                                    identity.vouched()
                                } else {
                                    identity
                                };
                                (identity, digest)
                            })
                            .ok_or_else(|| format!("{}: sha256 is not 64 hex digits", entry.name))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
        });

        match parsed {
            Ok(Some(keys)) => Self {
                keys,
                state: FileState::Loaded,
            },
            Ok(None) => Self {
                keys: Vec::new(),
                state: FileState::Missing,
            },
            Err(reason) => {
                log::error!("{} is unusable: {reason}", path.display());
                Self {
                    keys: Vec::new(),
                    state: FileState::Broken(reason),
                }
            }
        }
    }

    pub fn state(&self) -> &FileState {
        &self.state
    }
}

impl ApiAuthenticator for EnterpriseApiKeys {
    fn describe(&self) -> ApiAuthenticatorInfo {
        ApiAuthenticatorInfo {
            id: API_KEYS_AUTHENTICATOR_ID.to_string(),
            label: ui_text(REFERENCE_API_KEYS_LABEL_KEY).into(),
            description: ui_text(REFERENCE_API_KEYS_DESCRIPTION_KEY).into(),
            ..Default::default()
        }
    }

    fn authenticate(&self, token: &str) -> Option<ApiIdentity> {
        let presented: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let mut matched = None;

        for (identity, digest) in &self.keys {
            if bool::from(digest.ct_eq(&presented)) {
                matched = Some(identity);
            }
        }
        matched.cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const TEST_DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    #[test]
    fn recognizes_keys_by_their_digest() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("api-keys.json");
        fs::write(
            &path,
            format!(
                r#"{{"keys":[{{"name":" ops-bot ","roles":["operator"],"sha256":"{TEST_DIGEST}"}}]}}"#
            ),
        )
        .unwrap();

        let keys = EnterpriseApiKeys::load(&path);

        assert_eq!(keys.state(), &FileState::Loaded);
        let caller = keys.authenticate("test").unwrap();
        assert_eq!(caller.name, "ops-bot");
        assert_eq!(caller.roles, ["operator"]);
        assert!(
            !caller.vouched,
            "a key with its own roles acts with exactly those"
        );
        assert!(keys.authenticate("Test").is_none());
    }

    #[test]
    fn lets_a_key_without_roles_act_as_the_team_member_it_is_named_after() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("api-keys.json");
        fs::write(
            &path,
            format!(r#"{{"keys":[{{"name":"alice","sha256":"{TEST_DIGEST}"}}]}}"#),
        )
        .unwrap();

        assert!(
            EnterpriseApiKeys::load(&path)
                .authenticate("test")
                .unwrap()
                .vouched
        );
    }

    #[test]
    fn recognizes_nothing_without_a_usable_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("api-keys.json");

        let missing = EnterpriseApiKeys::load(&path);
        assert_eq!(missing.state(), &FileState::Missing);
        assert!(missing.authenticate("test").is_none());

        fs::write(&path, r#"{"keys":[{"name":"x","sha256":"abc"}]}"#).unwrap();
        let broken = EnterpriseApiKeys::load(&path);
        assert!(matches!(broken.state(), FileState::Broken(reason) if reason.contains("64 hex")));
        assert!(broken.authenticate("test").is_none());
    }
}
