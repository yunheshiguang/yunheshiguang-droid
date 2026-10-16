use std::fs;
use std::path::Path;

use yunheshiguang_sdk::Slot;
use yunheshiguang_sdk::documents::{Collection, Document, DocumentId};
use yunheshiguang_sdk::governance::AuditQuery;

use super::*;

fn paths(root: &Path) -> ReferencePaths {
    let paths = ReferencePaths {
        config_dir: root.join("config"),
        data_dir: root.join("data"),
    };
    fs::create_dir_all(&paths.config_dir).unwrap();
    paths
}

#[test]
fn fills_every_commercial_slot_from_its_own_files() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());
    let synced = root.path().join("Dropbox/yunheshiguang");
    fs::write(
        paths.policy_file(),
        r#"@id("admins") permit (principal in Role::"admin", action, resource);"#,
    )
    .unwrap();
    fs::write(
        paths.team_file(),
        r#"{ "members": [{ "id": "alice", "name": "Alice", "roles": ["admin"], "accounts": ["alice"] }] }"#,
    )
    .unwrap();
    fs::write(
        paths.sync_file(),
        serde_json::json!({ "folder": synced }).to_string(),
    )
    .unwrap();
    fs::write(
        paths.redroid_file(),
        r#"{ "docker": "tcp://127.0.0.1:2375", "adbHost": "127.0.0.1" }"#,
    )
    .unwrap();

    let mut contributions = Contributions::default();
    let status = contribute_with(&paths, "alice", &mut contributions).unwrap();
    let parts = contributions.into_parts();

    assert!(parts.authorization.is_some());
    assert!(parts.entitlements.is_some());
    assert_eq!(parts.identity.unwrap().current().id, "alice");
    assert!(
        parts.audit_sinks.is_empty(),
        "the team store replaces the sink"
    );
    assert_eq!(parts.template_sources.len(), 1);
    assert!(
        parts.transports.is_empty(),
        "cloud phones come from a provider now"
    );
    assert_eq!(parts.cloud_providers[0].describe().id, REDROID_PROVIDER_ID);
    assert!(
        parts.assistant_providers.is_empty(),
        "no assistant.json, no provider"
    );
    assert_eq!(parts.telemetry_sinks.len(), 1);
    assert_eq!(parts.vendor_adapters.len(), 1);
    assert_eq!(parts.input_channels.len(), 1);
    assert_eq!(parts.script_engines[0].describe().id, AUTOJS6_ENGINE_ID);
    assert_eq!(
        parts.api_authenticators[0].describe().id,
        API_KEYS_AUTHENTICATOR_ID
    );
    assert_eq!(status.policy, FileState::Loaded);
    assert_eq!(status.policy_rules, 1);
    assert_eq!(status.team, FileState::Loaded);
    assert_eq!(status.team_members, 1);
    assert_eq!(status.redroid, FileState::Loaded);
    assert_eq!(status.assistant, FileState::Missing);
    assert_eq!(status.api_keys, FileState::Missing);
    assert_eq!(status.vendors, FileState::Missing);
    assert_eq!(status.license, FileState::Missing);
    assert_eq!(status.sync, FileState::Loaded);
    assert_eq!(status.sync_folder, synced.display().to_string());
    assert_eq!(
        status.audit_folder,
        synced.join("audit").display().to_string()
    );
    assert!(status.metrics_file.ends_with("metrics.prom"));

    let collection = Collection::parse("automation-flows").unwrap();
    let document = Document::new(DocumentId::parse("flow-1").unwrap(), "{}").unwrap();
    parts
        .documents
        .unwrap()
        .save(&collection, &document)
        .unwrap();
    assert!(synced.join("automation-flows/flow-1.json").is_file());

    let store = parts.audit_store.unwrap();
    assert!(
        store
            .query(&AuditQuery::default())
            .unwrap()
            .entries
            .is_empty()
    );
}

#[test]
fn keeps_the_core_identity_and_one_computer_id_without_a_team() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());

    let mut contributions = Contributions::default();
    let status = contribute_with(&paths, "alice", &mut contributions).unwrap();
    let first = fs::read_to_string(paths.computer_id_file()).unwrap();
    contribute_with(&paths, "alice", &mut Contributions::default()).unwrap();

    assert_eq!(status.team, FileState::Missing);
    assert_eq!(status.policy, FileState::Missing);
    assert!(contributions.into_parts().identity.is_none());
    assert_eq!(
        fs::read_to_string(paths.computer_id_file()).unwrap(),
        first,
        "the audit file name survives restarts"
    );
}

#[test]
fn keeps_documents_on_this_computer_when_the_sync_file_is_broken() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());
    fs::write(paths.sync_file(), r#"{ "directory": "/elsewhere" }"#).unwrap();

    let status = contribute_with(&paths, "alice", &mut Contributions::default()).unwrap();

    assert!(matches!(status.sync, FileState::Broken(reason) if reason.contains("directory")));
    assert_eq!(
        status.sync_folder,
        paths.local_documents().display().to_string()
    );
}

#[test]
fn keeps_documents_on_this_computer_when_the_sync_folder_is_relative() {
    let root = tempfile::tempdir().unwrap();
    let paths = paths(root.path());
    fs::write(paths.sync_file(), r#"{ "folder": "Dropbox/team" }"#).unwrap();

    let status = contribute_with(&paths, "alice", &mut Contributions::default()).unwrap();

    assert!(matches!(status.sync, FileState::Broken(reason) if reason.contains("absolute")));
    assert_eq!(
        status.sync_folder,
        paths.local_documents().display().to_string()
    );
}

#[test]
fn refuses_a_slot_another_extension_already_filled() {
    let root = tempfile::tempdir().unwrap();
    let mut contributions = Contributions::default();
    contributions
        .set_authorization(PolicyAuthorization::new(Policy::Missing))
        .unwrap();

    let conflict = contribute_with(&paths(root.path()), "alice", &mut contributions).unwrap_err();

    assert_eq!(conflict.slot, Slot::Authorization);
}
