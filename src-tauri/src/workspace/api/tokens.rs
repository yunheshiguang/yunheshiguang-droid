use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;
use yunheshiguang_contract::wkt::Timestamp;

use crate::workspace::clock;
use crate::workspace::contract::{ApiToken, UiText};
use crate::workspace::local_state::{read_json, write_json};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

pub const TOKEN_PREFIX: &str = "yhsg_";
pub const MAX_TOKENS: usize = 20;
pub const MAX_TOKEN_NAME_CHARS: usize = 32;
const LAST_USED_RESOLUTION_SECONDS: i64 = 60;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredToken {
    id: String,
    name: String,
    digest: String,
    created_at: Option<Timestamp>,
    last_used_at: Option<Timestamp>,
}

impl StoredToken {
    fn public(&self) -> ApiToken {
        ApiToken {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at: self.created_at.clone().into(),
            last_used_at: self.last_used_at.clone().into(),
            ..Default::default()
        }
    }
}

fn digest(secret: &str) -> [u8; 32] {
    Sha256::digest(secret.as_bytes()).into()
}

pub struct TokenStore {
    path: Option<PathBuf>,
    tokens: Mutex<Vec<StoredToken>>,
}

impl TokenStore {
    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self {
            path: None,
            tokens: Mutex::new(Vec::new()),
        }
    }

    pub fn load(path: PathBuf) -> Self {
        let tokens = read_json::<Vec<StoredToken>>(&path).unwrap_or_default();

        Self {
            path: Some(path),
            tokens: Mutex::new(tokens),
        }
    }

    fn tokens(&self) -> MutexGuard<'_, Vec<StoredToken>> {
        self.tokens.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn persist(&self, tokens: &[StoredToken]) -> Result<(), UiText> {
        let Some(path) = &self.path else {
            return Ok(());
        };

        write_json(path, &tokens).map_err(|error| {
            ui_text_with_params(API_STORAGE_FAILED_KEY, "", [("message", error.to_string())])
        })
    }

    pub fn list(&self) -> Vec<ApiToken> {
        self.tokens().iter().map(StoredToken::public).collect()
    }

    pub fn create(&self, name: &str) -> Result<(ApiToken, String), UiText> {
        let name = name.trim();

        if name.is_empty() {
            return Err(ui_text(API_TOKEN_NAME_REQUIRED_KEY));
        }
        if name.chars().count() > MAX_TOKEN_NAME_CHARS {
            return Err(ui_text_with_params(
                API_TOKEN_NAME_TOO_LONG_KEY,
                "",
                [("max", MAX_TOKEN_NAME_CHARS.to_string())],
            ));
        }

        let mut tokens = self.tokens();
        if tokens
            .iter()
            .any(|token| token.name.to_lowercase() == name.to_lowercase())
        {
            return Err(ui_text(API_TOKEN_NAME_TAKEN_KEY));
        }
        if tokens.len() >= MAX_TOKENS {
            return Err(ui_text_with_params(
                API_TOKEN_LIMIT_KEY,
                "",
                [("max", MAX_TOKENS.to_string())],
            ));
        }

        let mut random = [0_u8; 32];
        getrandom::fill(&mut random).map_err(|error| {
            ui_text_with_params(API_STORAGE_FAILED_KEY, "", [("message", error.to_string())])
        })?;
        let secret = format!("{TOKEN_PREFIX}{}", URL_SAFE_NO_PAD.encode(random));
        let token = StoredToken {
            id: Uuid::now_v7().to_string(),
            name: name.to_string(),
            digest: STANDARD.encode(digest(&secret)),
            created_at: Some(clock::now()),
            last_used_at: None,
        };

        tokens.push(token.clone());
        if let Err(problem) = self.persist(&tokens) {
            tokens.pop();
            return Err(problem);
        }
        Ok((token.public(), secret))
    }

    pub fn revoke(&self, id: &str) -> Result<Option<String>, UiText> {
        let mut tokens = self.tokens();
        let Some(index) = tokens.iter().position(|token| token.id == id) else {
            return Ok(None);
        };
        let removed = tokens.remove(index);

        if let Err(problem) = self.persist(&tokens) {
            tokens.insert(index, removed);
            return Err(problem);
        }
        Ok(Some(removed.name))
    }

    pub fn authenticate(&self, secret: &str) -> Option<String> {
        let presented = digest(secret);
        let mut tokens = self.tokens();
        let mut matched = None;

        for (index, token) in tokens.iter().enumerate() {
            let stored = STANDARD.decode(&token.digest).unwrap_or_default();

            if bool::from(stored.ct_eq(&presented)) {
                matched = Some(index);
            }
        }

        let token = &mut tokens[matched?];
        let now = clock::now();
        let stale = token
            .last_used_at
            .as_ref()
            .is_none_or(|last| now.seconds - last.seconds >= LAST_USED_RESOLUTION_SECONDS);
        let name = token.name.clone();

        if stale {
            token.last_used_at = Some(now);
            if let Err(problem) = self.persist(&tokens) {
                log::warn!("api: token use not recorded: {}", problem.key);
            }
        }
        Some(name)
    }
}
