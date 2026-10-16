use std::collections::HashSet;
use std::sync::Arc;

use yunheshiguang_sdk::api::{ApiAuthenticator, ApiAuthenticatorInfo, ApiIdentity};

use crate::workspace::messages::*;
use crate::workspace::text::ui_text;

use super::tokens::TokenStore;

pub const CORE_TOKENS_ID: &str = "core.tokens";

pub struct CoreTokens(pub Arc<TokenStore>);

impl ApiAuthenticator for CoreTokens {
    fn describe(&self) -> ApiAuthenticatorInfo {
        ApiAuthenticatorInfo {
            id: CORE_TOKENS_ID.to_string(),
            label: ui_text(API_CORE_TOKENS_LABEL_KEY).into(),
            description: ui_text(API_CORE_TOKENS_DESCRIPTION_KEY).into(),
            ..Default::default()
        }
    }

    fn authenticate(&self, token: &str) -> Option<ApiIdentity> {
        self.0.authenticate(token).map(ApiIdentity::new)
    }
}

pub struct Authenticators {
    chain: Vec<Box<dyn ApiAuthenticator>>,
}

impl Authenticators {
    pub fn new(
        contributed: Vec<Box<dyn ApiAuthenticator>>,
        core: Vec<Box<dyn ApiAuthenticator>>,
    ) -> Self {
        let mut seen = HashSet::new();
        let chain = contributed
            .into_iter()
            .chain(core)
            .filter(|authenticator| {
                let id = authenticator.describe().id;
                let first = seen.insert(id.clone());

                if !first {
                    log::warn!("api authenticator {id} is installed twice; keeping the first");
                }
                first
            })
            .collect();

        Self { chain }
    }

    pub fn describe(&self) -> Vec<ApiAuthenticatorInfo> {
        self.chain
            .iter()
            .map(|authenticator| authenticator.describe())
            .collect()
    }

    pub fn identify(&self, token: &str) -> Option<ApiIdentity> {
        if token.is_empty() {
            return None;
        }

        self.chain
            .iter()
            .find_map(|authenticator| authenticator.authenticate(token))
    }
}
