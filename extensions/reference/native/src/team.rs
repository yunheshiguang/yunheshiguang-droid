use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;
use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::GovernanceProvider;
use yunheshiguang_sdk::governance::{IdentityProvider, Operator, provider_info};

use crate::config::read_json;
use crate::messages::*;
use crate::status::FileState;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TeamFile {
    members: Vec<Member>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    id: String,
    name: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    accounts: Vec<String>,
}

impl Member {
    fn operator(&self) -> Operator {
        Operator {
            id: self.id.clone(),
            name: self.name.clone(),
            roles: self.roles.clone(),
            ..Default::default()
        }
    }
}

pub struct TeamIdentity {
    members: Vec<Member>,
    account: String,
}

fn validate(file: TeamFile) -> Result<Vec<Member>, String> {
    let mut seen = HashSet::new();

    for member in &file.members {
        if member.id.trim().is_empty() {
            return Err("a member has an empty id".to_string());
        }
        if !seen.insert(member.id.as_str()) {
            return Err(format!("member {} is listed twice", member.id));
        }
    }

    Ok(file.members)
}

impl TeamIdentity {
    pub fn new(account: &str) -> Self {
        Self {
            members: Vec::new(),
            account: account.to_string(),
        }
    }

    pub fn load(path: &Path, account: &str) -> (Option<Self>, FileState) {
        let identity = |members| Self {
            members,
            account: account.to_string(),
        };

        match read_json::<TeamFile>(path).and_then(|file| file.map(validate).transpose()) {
            Ok(None) => (None, FileState::Missing),
            Ok(Some(members)) => (Some(identity(members)), FileState::Loaded),
            Err(reason) => (Some(identity(Vec::new())), FileState::Broken(reason)),
        }
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

impl IdentityProvider for TeamIdentity {
    fn describe(&self) -> GovernanceProvider {
        provider_info(
            "reference.team",
            REFERENCE_TEAM_LABEL_KEY,
            REFERENCE_TEAM_DESCRIPTION_KEY,
        )
    }

    fn current(&self) -> Operator {
        self.members
            .iter()
            .find(|member| member.accounts.contains(&self.account))
            .map_or_else(
                || Operator {
                    id: format!("guest:{}", self.account),
                    name: self.account.clone(),
                    roles: Vec::new(),
                    ..Default::default()
                },
                Member::operator,
            )
    }

    fn resolve(&self, id: &str) -> Option<Operator> {
        self.members
            .iter()
            .find(|member| member.id == id)
            .map(Member::operator)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn load(contents: &str, account: &str) -> (Option<TeamIdentity>, FileState) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("team.json");
        fs::write(&path, contents).unwrap();

        TeamIdentity::load(&path, account)
    }

    const TEAM: &str = r#"{ "members": [
        { "id": "alice", "name": "Alice", "roles": ["admin"], "accounts": ["alice", "a.wang"] },
        { "id": "ops-bot", "name": "Ops bot", "roles": ["operator"] }
    ] }"#;

    #[test]
    fn knows_this_computer_by_its_login_and_api_callers_by_their_id() {
        let (identity, state) = load(TEAM, "a.wang");
        let identity = identity.unwrap();

        assert_eq!(state, FileState::Loaded);
        assert_eq!(identity.member_count(), 2);
        assert_eq!(identity.current().id, "alice");
        assert_eq!(identity.current().roles, ["admin"]);
        assert_eq!(identity.resolve("ops-bot").unwrap().roles, ["operator"]);
        assert_eq!(identity.resolve("nobody"), None);
    }

    #[test]
    fn treats_strangers_as_guests_without_roles() {
        let stranger = load(TEAM, "bob").0.unwrap().current();

        assert_eq!(stranger.id, "guest:bob");
        assert!(stranger.roles.is_empty());
    }

    #[test]
    fn recognises_nobody_when_the_team_file_is_broken() {
        for contents in [
            "{",
            r#"{ "members": [{ "id": "", "name": "x" }] }"#,
            r#"{ "members": [{ "id": "a", "name": "A" }, { "id": "a", "name": "B" }] }"#,
        ] {
            let (identity, state) = load(contents, "alice");

            assert!(matches!(state, FileState::Broken(_)), "{contents}");
            assert!(identity.unwrap().current().roles.is_empty());
        }

        let folder = tempfile::tempdir().unwrap();
        let (identity, state) = TeamIdentity::load(&folder.path().join("team.json"), "alice");
        assert!(identity.is_none());
        assert_eq!(state, FileState::Missing);
    }
}
