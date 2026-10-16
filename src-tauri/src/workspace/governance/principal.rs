use crate::workspace::contract::Operator;

pub const API_ACTOR_PREFIX: &str = "api:";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Principal {
    pub actor: String,
    pub operator: Operator,
}

impl Principal {
    pub fn new(actor: impl Into<String>, operator: Operator) -> Self {
        Self {
            actor: actor.into(),
            operator,
        }
    }

    pub fn is_api(&self) -> bool {
        self.actor.starts_with(API_ACTOR_PREFIX)
    }
}
