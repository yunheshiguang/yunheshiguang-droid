use std::sync::{Arc, Mutex};

use tempfile::TempDir;
use yunheshiguang_contract::yunheshiguang::packages::v1::MarketRef;
use yunheshiguang_contract::yunheshiguang::templates::v1::{FlowTemplate, TemplateSourceInfo};
use yunheshiguang_sdk::documents::{
    Collection, Document, DocumentId, DocumentStore, FolderDocumentStore,
};
use yunheshiguang_sdk::governance::{EntitlementProvider, provider_info};
use yunheshiguang_sdk::templates::TemplateSource;

use super::format::{check_manifest, digest_of, parse_package, read_package};
use super::library::{Library, Plan, install, plan, preview, read_library};
use super::service::PackagesService;
use crate::workspace::automation::FLOWS_COLLECTION;
use crate::workspace::contract::*;
use crate::workspace::events::WorkspaceEvents;
use crate::workspace::governance::{Governance, GovernanceParts, Principal};
use crate::workspace::messages::*;
use crate::workspace::scripts::SCRIPTS_COLLECTION;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::templates::TemplateCatalog;
use crate::workspace::text::ui_text;

fn step(kind: BatchActionKind, value: &str) -> FlowStep {
    FlowStep {
        id: format!("step-{value}"),
        kind: kind.into(),
        value: value.to_string(),
        ..Default::default()
    }
}

fn flow(name: &str, script: &str) -> Flow {
    Flow {
        name: name.to_string(),
        steps: vec![
            step(BatchActionKind::LaunchApp, "com.android.settings"),
            step(BatchActionKind::RunScript, script),
        ],
        ..Default::default()
    }
}

fn script(name: &str) -> Script {
    Script {
        name: name.to_string(),
        language: ScriptLanguage::Shell.into(),
        body: "dumpsys battery".to_string(),
        ..Default::default()
    }
}

fn manifest(id: &str, version: &str) -> PackageManifest {
    PackageManifest {
        id: id.to_string(),
        name: "Nightly".to_string(),
        version: version.to_string(),
        ..PackageManifest::default()
    }
}

fn package() -> TemplatePackage {
    TemplatePackage {
        manifest: manifest("com.example.nightly", "1.2.0").into(),
        flows: vec![PackageFlow {
            id: "nightly".to_string(),
            flow: flow("Nightly", "battery").into(),
            ..Default::default()
        }],
        scripts: vec![PackageScript {
            id: "battery".to_string(),
            script: script("Battery").into(),
            ..Default::default()
        }],
        ..TemplatePackage::default()
    }
}

#[test]
fn digests_the_raw_json_regardless_of_key_order_and_new_fields() {
    let ordered: serde_json::Value =
        serde_json::from_str(r#"{"manifest":{"id":"a","version":"1.0.0"},"flows":[]}"#).unwrap();
    let shuffled: serde_json::Value = serde_json::from_str(
        r#"{"flows":[],"digest":"ignored","manifest":{"version":"1.0.0","id":"a"}}"#,
    )
    .unwrap();
    let newer: serde_json::Value = serde_json::from_str(
        r#"{"manifest":{"id":"a","version":"1.0.0","license":"MIT"},"flows":[]}"#,
    )
    .unwrap();

    assert_eq!(digest_of(&ordered), digest_of(&shuffled));
    assert_ne!(digest_of(&ordered), digest_of(&newer));
    assert_eq!(digest_of(&ordered).len(), 64);
}

#[test]
fn verifies_package_files_and_refuses_tampered_ones() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nightly.json");
    let keys = super::signing::SigningKeys::new(directory.path().join("keys"));
    super::format::write_package(&path, &package(), &keys, "Alice").unwrap();
    let written = std::fs::read_to_string(&path).unwrap();

    let loaded = parse_package(&written).unwrap();
    assert!(loaded.signer.is_some());
    assert!(
        !loaded.verified,
        "reading a file never vouches for its signer"
    );
    assert_eq!(loaded.package.flows[0].id, "nightly");

    let tampered = written.replace("dumpsys battery", "reboot");
    assert_eq!(
        parse_package(&tampered).err(),
        Some(ui_text(PACKAGES_DIGEST_MISMATCH_KEY))
    );

    let mut unsigned: serde_json::Value = serde_json::from_str(&written).unwrap();
    unsigned.as_object_mut().unwrap().remove("digest");
    let handmade = parse_package(&unsigned.to_string()).unwrap();
    assert!(
        !handmade.verified,
        "hand-written packages install but are not verified"
    );
    assert_eq!(
        preview(&handmade.package, &Library::default(), handmade.verified).problems,
        [ui_text(PACKAGES_UNVERIFIED_KEY)]
    );
}

#[test]
fn requires_an_id_a_name_and_a_semantic_version() {
    assert_eq!(
        check_manifest(&manifest("com.example.nightly", "1.2.0")),
        Ok(())
    );
    assert_eq!(
        check_manifest(&manifest("com.example.nightly", "2.0.0-beta.1")),
        Ok(())
    );

    for (id, version, key) in [
        ("", "1.0.0", PACKAGES_ID_INVALID_KEY),
        (".hidden", "1.0.0", PACKAGES_ID_INVALID_KEY),
        ("a/b", "1.0.0", PACKAGES_ID_INVALID_KEY),
        ("ok", "1.0", PACKAGES_VERSION_INVALID_KEY),
        ("ok", "latest", PACKAGES_VERSION_INVALID_KEY),
    ] {
        assert_eq!(
            check_manifest(&manifest(id, version)),
            Err(ui_text(key)),
            "{id} {version}"
        );
    }
    assert_eq!(
        check_manifest(&PackageManifest {
            name: "  ".to_string(),
            ..manifest("ok", "1.0.0")
        }),
        Err(ui_text(PACKAGES_NAME_REQUIRED_KEY))
    );
}

fn library_with_conflicts() -> Library {
    Library {
        flows: [("nightly".to_string(), flow("Mine", "battery"))].into(),
        scripts: [("battery".to_string(), script("Mine"))].into(),
        ..Library::default()
    }
}

#[test]
fn reads_package_files_but_not_folders_or_devices() {
    let directory = tempfile::tempdir().unwrap();

    assert_eq!(
        read_package(directory.path())
            .err()
            .map(|reason| reason.key),
        Some(PACKAGES_READ_FAILED_KEY.to_string())
    );
}

#[test]
fn writes_nothing_when_any_item_of_an_install_is_unusable() {
    let directory = tempfile::tempdir().unwrap();
    let store = FolderDocumentStore::new(directory.path());
    let unusable = Plan {
        scripts: vec![
            ("battery".to_string(), script("Battery")),
            ("not/an id".to_string(), script("Broken")),
        ],
        flows: vec![("nightly".to_string(), flow("Nightly", "battery"))],
    };

    assert!(install(&store, &unusable).is_err());
    let library = read_library(&store).unwrap();
    assert!(library.scripts.is_empty() && library.flows.is_empty());
}

#[test]
fn treats_unreadable_documents_as_taken() {
    let directory = tempfile::tempdir().unwrap();
    let store = FolderDocumentStore::new(directory.path());
    store
        .save(
            &Collection::parse(FLOWS_COLLECTION).unwrap(),
            &Document::new(DocumentId::parse("nightly").unwrap(), "[]".to_string()).unwrap(),
        )
        .unwrap();
    let library = read_library(&store).unwrap();

    assert!(preview(&package(), &library, true).flows[0].exists);
    assert!(
        plan(&package(), &library, ConflictPolicy::Skip)
            .flows
            .is_empty()
    );
    assert_eq!(
        plan(&package(), &library, ConflictPolicy::KeepBoth).flows[0].0,
        "nightly-2"
    );
}

#[test]
fn resolves_conflicts_by_the_chosen_policy() {
    let library = library_with_conflicts();

    let replaced = plan(&package(), &library, ConflictPolicy::Replace);
    assert_eq!(replaced.scripts[0].0, "battery");
    assert_eq!(replaced.flows[0].0, "nightly");

    let skipped = plan(&package(), &library, ConflictPolicy::Skip);
    assert!(skipped.scripts.is_empty() && skipped.flows.is_empty());

    let kept = plan(&package(), &library, ConflictPolicy::Unspecified);
    assert_eq!(kept.scripts[0].0, "battery-2");
    assert_eq!(kept.flows[0].0, "nightly-2");
    assert_eq!(
        kept.flows[0].1.steps[1].value, "battery-2",
        "the flow runs the copy it came with"
    );
}

#[test]
fn keeps_both_when_a_renamed_item_meets_one_of_the_same_name_in_the_package() {
    let mut incoming = package();
    incoming.scripts.push(PackageScript {
        id: "battery-2".to_string(),
        script: script("Second").into(),
        ..Default::default()
    });
    incoming.flows.push(PackageFlow {
        id: "second".to_string(),
        flow: flow("Second", "battery-2").into(),
        ..Default::default()
    });

    let kept = plan(
        &incoming,
        &library_with_conflicts(),
        ConflictPolicy::KeepBoth,
    );
    let ids = kept
        .scripts
        .iter()
        .map(|(id, script)| (id.as_str(), script.name.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(ids, [("battery-2", "Battery"), ("battery-2-2", "Second")]);
    assert_eq!(kept.flows[0].1.steps[1].value, "battery-2");
    assert_eq!(kept.flows[1].1.steps[1].value, "battery-2-2");
}

#[test]
fn previews_conflicts_and_scripts_nobody_provides() {
    let mut package = package();
    package.flows[0].flow = flow("Nightly", "elsewhere").into();

    let shown = preview(&package, &library_with_conflicts(), true);

    assert!(shown.flows[0].exists && shown.scripts[0].exists);
    assert_eq!(shown.problems.len(), 1);
    assert_eq!(shown.problems[0].key, PACKAGES_SCRIPT_NOT_INCLUDED_KEY);
    assert_eq!(shown.problems[0].params["script"], "elsewhere");
}

mod service;
