use crate::contract::yunheshiguang::governance::v1::{
    AuditEntry, GovernanceProvider, GovernedAction, License,
};
pub use crate::contract::yunheshiguang::governance::v1::{AuditPage, AuditQuery, Operator};
use crate::contract::yunheshiguang::workspace::v1::UiText;
use crate::text::ui_text;

#[non_exhaustive]
pub struct ActionContext<'a> {
    pub action: GovernedAction,
    pub actor: &'a str,
    pub targets: &'a [String],
    pub operator: Option<&'a Operator>,
    pub groups: &'a [String],
}

impl<'a> ActionContext<'a> {
    pub fn new(action: GovernedAction, actor: &'a str, targets: &'a [String]) -> Self {
        Self {
            action,
            actor,
            targets,
            operator: None,
            groups: &[],
        }
    }

    pub fn with_operator(mut self, operator: &'a Operator) -> Self {
        self.operator = Some(operator);
        self
    }

    pub fn with_groups(mut self, groups: &'a [String]) -> Self {
        self.groups = groups;
        self
    }
}

#[non_exhaustive]
pub enum Decision {
    Allow,
    Deny(UiText),
}

impl Decision {
    pub fn into_result(self) -> Result<(), UiText> {
        match self {
            Self::Allow => Ok(()),
            Self::Deny(reason) => Err(reason),
        }
    }
}

pub trait AuthorizationProvider: Send + Sync {
    fn describe(&self) -> GovernanceProvider;

    fn authorize(&self, context: &ActionContext<'_>) -> Decision;
}

pub trait EntitlementProvider: Send + Sync {
    fn describe(&self) -> GovernanceProvider;

    fn granted_capabilities(&self) -> Vec<String>;

    fn license(&self) -> Option<License> {
        None
    }
}

pub trait AuditSink: Send + Sync {
    fn describe(&self) -> GovernanceProvider;

    fn record(&self, entry: &AuditEntry);
}

pub trait AuditStore: Send + Sync {
    fn describe(&self) -> GovernanceProvider;

    fn append(&self, entry: &AuditEntry);

    fn query(&self, query: &AuditQuery) -> Result<AuditPage, UiText>;

    fn flush(&self) {}
}

pub trait IdentityProvider: Send + Sync {
    fn describe(&self) -> GovernanceProvider;

    fn current(&self) -> Operator;

    fn resolve(&self, id: &str) -> Option<Operator> {
        let _ = id;
        None
    }
}

pub fn actor_matches(filter: &str, actor: &str) -> bool {
    let filter = filter.trim();

    filter.is_empty()
        || actor == filter
        || actor
            .strip_prefix(filter)
            .is_some_and(|rest| rest.starts_with(':'))
}

pub fn timestamp_millis(timestamp: &crate::contract::wkt::Timestamp) -> i64 {
    timestamp
        .seconds
        .saturating_mul(1_000)
        .saturating_add(i64::from(timestamp.nanos / 1_000_000))
}

pub fn audit_matches(query: &AuditQuery, entry: &AuditEntry) -> bool {
    let operator_id = entry
        .operator
        .as_option()
        .map(|operator| operator.id.as_str())
        .unwrap_or_default();
    let at = entry.occurred_at.as_option().map_or(0, timestamp_millis);

    (query.actions.is_empty() || query.actions.contains(&entry.action))
        && (query.outcomes.is_empty() || query.outcomes.contains(&entry.outcome))
        && (query.operator_id.trim().is_empty() || query.operator_id.trim() == operator_id)
        && actor_matches(&query.actor, &entry.actor)
        && query
            .since
            .as_option()
            .is_none_or(|since| at >= timestamp_millis(since))
        && query
            .until
            .as_option()
            .is_none_or(|until| at < timestamp_millis(until))
}

pub fn provider_info(id: &str, label_key: &str, description_key: &str) -> GovernanceProvider {
    GovernanceProvider {
        id: id.to_string(),
        label: ui_text(label_key).into(),
        description: ui_text(description_key).into(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Nobody;

    impl IdentityProvider for Nobody {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn current(&self) -> Operator {
            Operator {
                id: "local".to_string(),
                ..Operator::default()
            }
        }
    }

    #[test]
    fn builds_contexts_with_and_without_the_operator() {
        let targets = vec!["serial".to_string()];
        let bare = ActionContext::new(GovernedAction::RunBatchAction, "main", &targets);
        assert!(bare.operator.is_none());
        assert!(bare.groups.is_empty());

        let operator = Nobody.current();
        let groups = vec!["shelf-a".to_string()];
        let full = ActionContext::new(GovernedAction::RunBatchAction, "main", &targets)
            .with_operator(&operator)
            .with_groups(&groups);
        assert_eq!(
            full.operator.map(|operator| operator.id.as_str()),
            Some("local")
        );
        assert_eq!(full.groups, ["shelf-a"]);
        assert_eq!(Nobody.resolve("ops-bot"), None);
    }

    #[test]
    fn matches_an_exact_actor_or_every_actor_on_a_channel() {
        assert!(actor_matches("", "main"));
        assert!(actor_matches("main", "main"));
        assert!(actor_matches("api", "api:ops-bot"));
        assert!(actor_matches(" schedule ", "schedule:nightly"));
        assert!(!actor_matches("api", "apis"));
        assert!(!actor_matches("api:ops", "api:ops-bot"));
    }
}
