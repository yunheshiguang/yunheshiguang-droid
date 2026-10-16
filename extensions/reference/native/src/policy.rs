use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;
use yunheshiguang_sdk::contract::Enumeration;

use cedar_policy::{
    AuthorizationError, Authorizer, Context, Decision as CedarDecision, Effect, Entities, Entity,
    EntityId, EntityTypeName, EntityUid, PolicySet, Request, RestrictedExpression,
};
use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::{
    GovernanceProvider, GovernedAction,
};
use yunheshiguang_sdk::governance::{
    ActionContext, AuthorizationProvider, Decision, Operator, provider_info,
};
use yunheshiguang_sdk::text::{ui_text, ui_text_with_params};

use crate::messages::*;
use crate::status::FileState;

const ACTION_PREFIX: &str = "GOVERNED_ACTION_";

#[derive(Debug)]
pub enum Policy {
    Missing,
    Enforced(Box<PolicySet>),
    Broken(String),
}

impl Policy {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Err(error) if error.kind() == ErrorKind::NotFound => Self::Missing,
            Err(error) => Self::Broken(error.to_string()),
            Ok(text) => Self::parse(&text),
        }
    }

    pub fn parse(text: &str) -> Self {
        PolicySet::from_str(text).map_or_else(
            |error| Self::Broken(error.to_string()),
            |policies| Self::Enforced(Box::new(policies)),
        )
    }

    pub fn file_state(&self) -> FileState {
        match self {
            Self::Missing => FileState::Missing,
            Self::Enforced(_) => FileState::Loaded,
            Self::Broken(reason) => FileState::Broken(reason.clone()),
        }
    }

    pub fn rule_count(&self) -> usize {
        match self {
            Self::Enforced(policies) => policies.policies().count(),
            Self::Missing | Self::Broken(_) => 0,
        }
    }
}

fn action_name(action: GovernedAction) -> &'static str {
    let name = action.proto_name();
    name.strip_prefix(ACTION_PREFIX).unwrap_or(name)
}

fn uid(kind: &str, id: &str) -> Result<EntityUid, String> {
    let kind = EntityTypeName::from_str(kind).map_err(|error| error.to_string())?;

    Ok(EntityUid::from_type_name_and_id(kind, EntityId::new(id)))
}

fn request(context: &ActionContext<'_>) -> Result<(Request, Entities), String> {
    let unknown = Operator {
        id: "unknown".to_string(),
        ..Operator::default()
    };
    let operator = context.operator.unwrap_or(&unknown);
    let roles = operator
        .roles
        .iter()
        .map(|role| uid("Role", role))
        .collect::<Result<HashSet<_>, _>>()?;
    let principal = uid("Operator", &operator.id)?;
    let entities = Entities::from_entities(
        std::iter::once(Entity::new_no_attrs(principal.clone(), roles.clone())).chain(
            roles
                .into_iter()
                .map(|role| Entity::new_no_attrs(role, HashSet::new())),
        ),
        None,
    )
    .map_err(|error| error.to_string())?;
    let details = Context::from_pairs([
        (
            "actor".to_string(),
            RestrictedExpression::new_string(context.actor.to_string()),
        ),
        (
            "groups".to_string(),
            RestrictedExpression::new_set(
                context
                    .groups
                    .iter()
                    .map(|group| RestrictedExpression::new_string(group.clone())),
            ),
        ),
        (
            "targetCount".to_string(),
            RestrictedExpression::new_long(
                i64::try_from(context.targets.len()).unwrap_or(i64::MAX),
            ),
        ),
    ])
    .map_err(|error| error.to_string())?;
    let request = Request::new(
        principal,
        uid("Action", action_name(context.action))?,
        uid("Workspace", "local")?,
        details,
        None,
    )
    .map_err(|error| error.to_string())?;

    Ok((request, entities))
}

fn breaks_a_forbid(policies: &PolicySet, error: &AuthorizationError) -> bool {
    let AuthorizationError::PolicyEvaluationError(error) = error;

    policies
        .policy(error.policy_id())
        .is_none_or(|policy| policy.effect() == Effect::Forbid)
}

fn evaluate(policies: &PolicySet, context: &ActionContext<'_>) -> Decision {
    let (request, entities) = match request(context) {
        Ok(prepared) => prepared,
        Err(reason) => {
            log::warn!("policy: could not build a request: {reason}");
            return Decision::Deny(ui_text(REFERENCE_POLICY_BROKEN_KEY));
        }
    };
    let response = Authorizer::new().is_authorized(&request, policies, &entities);
    let broken = response
        .diagnostics()
        .errors()
        .filter(|error| breaks_a_forbid(policies, error))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if !broken.is_empty() {
        log::warn!(
            "policy: a forbid rule could not be evaluated: {}",
            broken.join("; ")
        );
        return Decision::Deny(ui_text(REFERENCE_POLICY_BROKEN_KEY));
    }

    match response.decision() {
        CedarDecision::Allow => Decision::Allow,
        CedarDecision::Deny => {
            let rules = response
                .diagnostics()
                .reason()
                .map(|id| {
                    policies
                        .policy(id)
                        .and_then(|policy| policy.annotation("id"))
                        .map_or_else(|| id.to_string(), str::to_string)
                })
                .collect::<Vec<_>>();

            let action = action_name(context.action).to_string();

            Decision::Deny(if rules.is_empty() {
                ui_text_with_params(REFERENCE_POLICY_DENIED_KEY, "", [("action", action)])
            } else {
                ui_text_with_params(
                    REFERENCE_POLICY_FORBIDDEN_KEY,
                    "",
                    [("action", action), ("rules", rules.join(", "))],
                )
            })
        }
    }
}

pub struct PolicyAuthorization {
    policy: Policy,
}

impl PolicyAuthorization {
    pub fn new(policy: Policy) -> Self {
        Self { policy }
    }
}

impl AuthorizationProvider for PolicyAuthorization {
    fn describe(&self) -> GovernanceProvider {
        provider_info(
            "reference.cedar-policy",
            REFERENCE_POLICY_LABEL_KEY,
            REFERENCE_POLICY_DESCRIPTION_KEY,
        )
    }

    fn authorize(&self, context: &ActionContext<'_>) -> Decision {
        match &self.policy {
            Policy::Missing => Decision::Allow,
            Policy::Enforced(policies) => evaluate(policies, context),
            Policy::Broken(_) => Decision::Deny(ui_text(REFERENCE_POLICY_BROKEN_KEY)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEAM_POLICY: &str = r#"
        @id("admins")
        permit (principal in Role::"admin", action, resource);

        @id("operators-on-shelf-a")
        permit (
            principal in Role::"operator",
            action in [Action::"RUN_BATCH_ACTION", Action::"RUN_AUTOMATION"],
            resource
        ) when { ["shelf-a"].containsAll(context.groups) };

        @id("no-settings-over-api")
        forbid (principal, action == Action::"MANAGE_API", resource)
        when { context.actor like "api:*" };
    "#;

    fn operator(id: &str, roles: &[&str]) -> Operator {
        Operator {
            id: id.to_string(),
            name: id.to_string(),
            roles: roles.iter().map(ToString::to_string).collect(),
            ..Default::default()
        }
    }

    fn decide(
        policy: &PolicyAuthorization,
        action: GovernedAction,
        actor: &str,
        who: &Operator,
        groups: &[&str],
    ) -> Result<(), UiTextKey> {
        let groups = groups.iter().map(ToString::to_string).collect::<Vec<_>>();
        let targets = vec!["serial".to_string()];
        let context = ActionContext::new(action, actor, &targets)
            .with_operator(who)
            .with_groups(&groups);

        policy
            .authorize(&context)
            .into_result()
            .map_err(|reason| UiTextKey(reason.key, reason.params.get("rules").cloned()))
    }

    #[derive(Debug, PartialEq)]
    struct UiTextKey(String, Option<String>);

    #[test]
    fn authorizes_by_role_and_by_the_groups_of_the_targets() {
        let policy = PolicyAuthorization::new(Policy::parse(TEAM_POLICY));
        let admin = operator("alice", &["admin"]);
        let crew = operator("bob", &["operator"]);
        let guest = operator("guest:carol", &[]);

        assert_eq!(
            decide(&policy, GovernedAction::PairDevice, "main", &admin, &[]),
            Ok(())
        );
        assert_eq!(
            decide(
                &policy,
                GovernedAction::RunBatchAction,
                "main",
                &crew,
                &["shelf-a"]
            ),
            Ok(())
        );
        let outside = decide(
            &policy,
            GovernedAction::RunBatchAction,
            "main",
            &crew,
            &["shelf-a", "shelf-b"],
        );
        assert_eq!(
            outside,
            Err(UiTextKey(REFERENCE_POLICY_DENIED_KEY.to_string(), None)),
            "nothing permits touching shelf-b"
        );
        assert!(
            decide(
                &policy,
                GovernedAction::OrganizeDevices,
                "main",
                &guest,
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn names_the_rule_that_forbade_the_action() {
        let policy = PolicyAuthorization::new(Policy::parse(TEAM_POLICY));
        let admin = operator("alice", &["admin"]);

        assert_eq!(
            decide(&policy, GovernedAction::ManageApi, "api:alice", &admin, &[]),
            Err(UiTextKey(
                REFERENCE_POLICY_FORBIDDEN_KEY.to_string(),
                Some("no-settings-over-api".to_string())
            ))
        );
        assert_eq!(
            decide(&policy, GovernedAction::ManageApi, "main", &admin, &[]),
            Ok(())
        );
    }

    #[test]
    fn denies_when_a_forbid_rule_cannot_be_evaluated() {
        let policy = PolicyAuthorization::new(Policy::parse(
            r#"
            permit (principal in Role::"admin", action, resource);

            forbid (principal, action == Action::"RUN_BATCH_ACTION", resource)
            when { resource.shelf == "prod" };
            "#,
        ));
        let admin = operator("alice", &["admin"]);

        assert_eq!(
            decide(&policy, GovernedAction::RunBatchAction, "main", &admin, &[]),
            Err(UiTextKey(REFERENCE_POLICY_BROKEN_KEY.to_string(), None))
        );
        assert_eq!(
            decide(&policy, GovernedAction::PairDevice, "main", &admin, &[]),
            Ok(()),
            "a forbid rule that does not apply cannot break anything"
        );
    }

    #[test]
    fn allows_everything_without_a_file_and_nothing_with_a_broken_one() {
        let folder = tempfile::tempdir().unwrap();
        let missing = Policy::load(&folder.path().join("policy.cedar"));
        assert_eq!(missing.file_state(), FileState::Missing);
        let anyone = operator("x", &[]);
        assert_eq!(
            decide(
                &PolicyAuthorization::new(missing),
                GovernedAction::PairDevice,
                "main",
                &anyone,
                &[]
            ),
            Ok(())
        );

        let broken = Policy::parse("permit (principal, action, resource");
        assert!(matches!(broken.file_state(), FileState::Broken(_)));
        assert_eq!(
            decide(
                &PolicyAuthorization::new(broken),
                GovernedAction::PairDevice,
                "main",
                &anyone,
                &[]
            ),
            Err(UiTextKey(REFERENCE_POLICY_BROKEN_KEY.to_string(), None))
        );
        assert_eq!(Policy::parse(TEAM_POLICY).rule_count(), 3);
    }
}
