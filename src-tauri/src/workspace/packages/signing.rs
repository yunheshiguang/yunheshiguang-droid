use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use minisign::{KeyPair, PublicKey, SecretKey, SecretKeyBox, SignatureBox};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::workspace::contract::{PackageSignature, UiText};
use crate::workspace::local_state::{load_json, write_json};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::format::signed_text;

const SIGNER_PREFIX: &str = "signer:";
const UNTRUSTED_COMMENT: &str = "yunheshiguang template package";
const KEY_FILE: &str = "signing-key.json";
const TRUSTED_FILE: &str = "trusted-signers.json";
const MAX_SIGNER_CHARS: usize = 80;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signed {
    pub key_id: String,
    pub name: String,
    pub public_key: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredKey {
    secret_key: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrustedSigner {
    public_key: String,
    name: String,
}

fn invalid(error: impl ToString) -> UiText {
    ui_text_with_params(
        PACKAGES_SIGNATURE_INVALID_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn unavailable(error: impl ToString) -> UiText {
    ui_text_with_params(
        PACKAGES_SIGNING_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn key_id(public_key: &PublicKey) -> String {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&public_key.keynum()[..8]);
    format!("{:016X}", u64::from_le_bytes(bytes))
}

pub fn verify(package: &Value) -> Result<Option<Signed>, UiText> {
    let Some(signature) = package.get("signature").filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let field = |name: &str| {
        signature
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
    };
    let public_key = PublicKey::from_base64(field("publicKey")).map_err(invalid)?;
    let minisig = SignatureBox::from_string(field("minisig")).map_err(invalid)?;
    let data = Cursor::new(signed_text(package).into_bytes());

    minisign::verify(&public_key, &minisig, data, true, false, false).map_err(invalid)?;
    let comment = minisig.trusted_comment().map_err(invalid)?;

    Ok(Some(Signed {
        key_id: key_id(&public_key),
        name: comment
            .strip_prefix(SIGNER_PREFIX)
            .unwrap_or_default()
            .to_string(),
        public_key: public_key.to_base64(),
    }))
}

pub struct SigningKeys {
    directory: PathBuf,
    own: Mutex<Option<KeyPair>>,
    trusting: Mutex<()>,
}

impl SigningKeys {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            own: Mutex::new(None),
            trusting: Mutex::new(()),
        }
    }

    fn own(&self) -> Result<KeyPair, UiText> {
        let mut own = self.own.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(pair) = own.as_ref() {
            return Ok(pair.clone());
        }

        let path = self.directory.join(KEY_FILE);
        let pair = match load_json::<StoredKey>(&path).map_err(unavailable)? {
            Some(stored) => {
                let secret = SecretKeyBox::from_string(&stored.secret_key)
                    .and_then(SecretKey::from_unencrypted_box)
                    .map_err(unavailable)?;
                let public = PublicKey::from_secret_key(&secret).map_err(unavailable)?;
                KeyPair {
                    pk: public,
                    sk: secret,
                }
            }
            None => {
                let pair = KeyPair::generate_unencrypted_keypair().map_err(unavailable)?;
                let secret_key = pair
                    .sk
                    .to_box(Some("yunheshiguang package signing key"))
                    .map_err(unavailable)?
                    .into_string();
                write_json(&path, &StoredKey { secret_key }).map_err(unavailable)?;
                pair
            }
        };

        *own = Some(pair.clone());
        Ok(pair)
    }

    pub fn sign(&self, package: &Value, signer: &str) -> Result<PackageSignature, UiText> {
        let pair = self.own()?;
        let signer = signer
            .trim()
            .chars()
            .take(MAX_SIGNER_CHARS)
            .collect::<String>();
        let comment = format!("{SIGNER_PREFIX}{signer}");
        let data = Cursor::new(signed_text(package).into_bytes());
        let minisig = minisign::sign(
            Some(&pair.pk),
            &pair.sk,
            data,
            Some(&comment),
            Some(UNTRUSTED_COMMENT),
        )
        .map_err(unavailable)?;

        Ok(PackageSignature {
            public_key: pair.pk.to_base64(),
            minisig: minisig.into_string(),
            ..Default::default()
        })
    }

    fn trusted(&self) -> Result<Vec<TrustedSigner>, UiText> {
        load_json(&self.directory.join(TRUSTED_FILE))
            .map(Option::unwrap_or_default)
            .map_err(unavailable)
    }

    pub fn trusted_name(&self, public_key: &str) -> Option<String> {
        self.trusted()
            .ok()?
            .into_iter()
            .find(|signer| signer.public_key == public_key)
            .map(|signer| signer.name)
            .filter(|name| !name.is_empty())
    }

    pub fn is_trusted(&self, public_key: &str) -> bool {
        self.own()
            .is_ok_and(|pair| pair.pk.to_base64() == public_key)
            || self
                .trusted()
                .inspect_err(|reason| log::warn!("packages: trusted signers: {}", reason.key))
                .unwrap_or_default()
                .iter()
                .any(|signer| signer.public_key == public_key)
    }

    pub fn trust(&self, public_key: &str, name: &str) -> Result<(), UiText> {
        let public_key = PublicKey::from_base64(public_key.trim())
            .map_err(|_| ui_text(PACKAGES_SIGNER_INVALID_KEY))?
            .to_base64();
        let _trusting = self.trusting.lock().unwrap_or_else(PoisonError::into_inner);
        let mut trusted = self.trusted()?;
        trusted.retain(|signer| signer.public_key != public_key);
        trusted.push(TrustedSigner {
            public_key,
            name: name.trim().chars().take(MAX_SIGNER_CHARS).collect(),
        });

        write_json(&self.directory.join(TRUSTED_FILE), &trusted).map_err(unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Value {
        serde_json::json!({ "manifest": { "id": "com.example.a", "version": "1.0.0" }, "flows": [] })
    }

    fn signed(keys: &SigningKeys, signer: &str) -> Value {
        let mut value = package();
        let signature = keys.sign(&value, signer).unwrap();
        value["signature"] = serde_json::to_value(signature).unwrap();
        value
    }

    #[test]
    fn signs_with_a_key_it_keeps_and_names_the_signer() {
        let folder = tempfile::tempdir().unwrap();
        let keys = SigningKeys::new(folder.path().to_path_buf());

        let value = signed(&keys, "Alice");
        let checked = verify(&value).unwrap().unwrap();

        assert_eq!(checked.name, "Alice");
        assert_eq!(checked.key_id.len(), 16);
        assert!(keys.is_trusted(&checked.public_key));

        let reopened = SigningKeys::new(folder.path().to_path_buf());
        assert_eq!(
            verify(&signed(&reopened, "Alice")).unwrap().unwrap().key_id,
            checked.key_id
        );
    }

    #[test]
    fn refuses_a_package_changed_after_signing() {
        let folder = tempfile::tempdir().unwrap();
        let keys = SigningKeys::new(folder.path().to_path_buf());
        let mut value = signed(&keys, "Alice");

        value["manifest"]["version"] = Value::String("1.0.1".to_string());

        assert_eq!(
            verify(&value).unwrap_err().key,
            PACKAGES_SIGNATURE_INVALID_KEY
        );
        assert_eq!(verify(&package()), Ok(None));
    }

    #[test]
    fn trusts_other_signers_only_once_the_operator_says_so() {
        let mine = tempfile::tempdir().unwrap();
        let theirs = tempfile::tempdir().unwrap();
        let keys = SigningKeys::new(mine.path().to_path_buf());
        let other = verify(&signed(
            &SigningKeys::new(theirs.path().to_path_buf()),
            "Bob",
        ))
        .unwrap()
        .unwrap();

        assert!(!keys.is_trusted(&other.public_key));
        keys.trust(&other.public_key, "Bob").unwrap();
        keys.trust(&other.public_key, "Bob").unwrap();
        assert!(keys.is_trusted(&other.public_key));
        assert_eq!(keys.trusted().unwrap().len(), 1);
        assert_eq!(
            keys.trust("not a key", "x").unwrap_err().key,
            PACKAGES_SIGNER_INVALID_KEY
        );
    }

    #[test]
    fn refuses_to_replace_a_key_or_trust_list_it_cannot_read() {
        let folder = tempfile::tempdir().unwrap();
        let theirs = tempfile::tempdir().unwrap();
        let other = verify(&signed(
            &SigningKeys::new(theirs.path().to_path_buf()),
            "Bob",
        ))
        .unwrap()
        .unwrap();
        std::fs::write(folder.path().join(KEY_FILE), "{").unwrap();
        std::fs::write(folder.path().join(TRUSTED_FILE), "[").unwrap();
        let keys = SigningKeys::new(folder.path().to_path_buf());

        assert_eq!(
            keys.sign(&package(), "Alice").unwrap_err().key,
            PACKAGES_SIGNING_FAILED_KEY
        );
        assert_eq!(
            keys.trust(&other.public_key, "Bob").unwrap_err().key,
            PACKAGES_SIGNING_FAILED_KEY
        );
        assert_eq!(
            std::fs::read_to_string(folder.path().join(KEY_FILE)).unwrap(),
            "{"
        );
        assert_eq!(
            std::fs::read_to_string(folder.path().join(TRUSTED_FILE)).unwrap(),
            "["
        );
    }
}
