use yunheshiguang_sdk::governance::{IdentityProvider, provider_info as provider};

use crate::workspace::contract::{GovernanceProvider, Operator};
use crate::workspace::messages::*;

pub const OWNER_ROLE: &str = "owner";

pub struct LocalUserIdentity {
    operator: Operator,
}

impl LocalUserIdentity {
    pub fn detect() -> Self {
        let account = whoami::username()
            .ok()
            .filter(|name| !name.trim().is_empty());
        let name = whoami::realname()
            .ok()
            .filter(|name| !name.trim().is_empty())
            .or_else(|| account.clone())
            .unwrap_or_default();

        Self::new(account.as_deref().unwrap_or_default(), &name)
    }

    pub fn new(account: &str, name: &str) -> Self {
        let id = if account.is_empty() {
            "local".to_string()
        } else {
            format!("local:{account}")
        };

        Self {
            operator: Operator {
                name: if name.is_empty() {
                    id.clone()
                } else {
                    name.to_string()
                },
                id,
                roles: vec![OWNER_ROLE.to_string()],
                ..Default::default()
            },
        }
    }
}

impl IdentityProvider for LocalUserIdentity {
    fn describe(&self) -> GovernanceProvider {
        provider(
            "core.local-user",
            GOVERNANCE_LOCAL_USER_LABEL_KEY,
            GOVERNANCE_LOCAL_USER_DESCRIPTION_KEY,
        )
    }

    fn current(&self) -> Operator {
        self.operator.clone()
    }

    fn resolve(&self, id: &str) -> Option<Operator> {
        (id == self.operator.id).then(|| self.operator.clone())
    }
}
