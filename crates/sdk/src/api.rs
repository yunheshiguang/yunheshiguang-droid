pub use crate::contract::yunheshiguang::api::v1::ApiAuthenticatorInfo;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiIdentity {
    pub name: String,
    pub roles: Vec<String>,
    pub vouched: bool,
}

impl ApiIdentity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            roles: Vec::new(),
            vouched: false,
        }
    }

    pub fn with_roles(mut self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    pub fn vouched(mut self) -> Self {
        self.vouched = true;
        self
    }
}

pub trait ApiAuthenticator: Send + Sync {
    fn describe(&self) -> ApiAuthenticatorInfo;

    fn authenticate(&self, token: &str) -> Option<ApiIdentity>;
}
