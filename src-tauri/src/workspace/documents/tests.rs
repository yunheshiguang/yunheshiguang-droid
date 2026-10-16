use yunheshiguang_contract::yunheshiguang::documents::v1::{
    Document, ListDocumentsRequest, RemoveDocumentRequest, SaveDocumentRequest,
};
use yunheshiguang_sdk::documents::FolderDocumentStore;

use super::store;

fn save_request(collection: &str, id: &str, body: &str) -> SaveDocumentRequest {
    SaveDocumentRequest {
        collection: collection.to_string(),
        document: Document {
            id: id.to_string(),
            body: body.to_string(),
            ..Default::default()
        }
        .into(),
        scope: Default::default(),
        ..Default::default()
    }
}

fn list_request(collection: &str) -> ListDocumentsRequest {
    ListDocumentsRequest {
        collection: collection.to_string(),
        scope: Default::default(),
        ..Default::default()
    }
}

#[test]
fn round_trips_documents_through_the_store() {
    let root = tempfile::tempdir().unwrap();
    let folder = FolderDocumentStore::new(root.path());

    let changed = store::save(
        &folder,
        &save_request("automation-flows", "flow-1", r#"{"name":"Recover"}"#),
    )
    .unwrap();
    assert_eq!(changed.as_str(), "automation-flows");

    let listed = store::list(&folder, &list_request("automation-flows")).unwrap();
    assert_eq!(listed.collection, "automation-flows");
    assert_eq!(listed.documents.len(), 1);
    assert_eq!(listed.documents[0].id, "flow-1");
    assert_eq!(listed.documents[0].body, r#"{"name":"Recover"}"#);

    store::remove(
        &folder,
        &RemoveDocumentRequest {
            collection: "automation-flows".to_string(),
            id: "flow-1".to_string(),
            scope: Default::default(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        store::list(&folder, &list_request("automation-flows"))
            .unwrap()
            .documents
            .is_empty()
    );
}

#[test]
fn rejects_paths_and_bodies_that_could_escape_or_corrupt_the_store() {
    let root = tempfile::tempdir().unwrap();
    let folder = FolderDocumentStore::new(root.path());

    assert!(store::list(&folder, &list_request("../outside")).is_err());
    assert!(store::save(&folder, &save_request("automation-flows", "../x", "{}")).is_err());
    assert!(store::save(&folder, &save_request("automation-flows", "flow-1", "{")).is_err());
    assert!(
        store::save(
            &folder,
            &SaveDocumentRequest {
                collection: "automation-flows".to_string(),
                document: Default::default(),
                scope: Default::default(),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        std::fs::read_dir(root.path()).unwrap().next().is_none(),
        "nothing was written"
    );
}

mod governed {
    use std::sync::{Arc, Mutex};

    use yunheshiguang_contract::yunheshiguang::documents::v1::{
        Document, ListDocumentsRequest, RemoveDocumentRequest, SaveDocumentRequest,
    };
    use yunheshiguang_sdk::documents::FolderDocumentStore;
    use yunheshiguang_sdk::governance::{
        ActionContext, AuthorizationProvider, Decision, EntitlementProvider,
    };

    use crate::workspace::contract::{AuditOutcome, GovernanceProvider, GovernedAction};
    use crate::workspace::documents::{DocumentsError, DocumentsService};
    use crate::workspace::events::WorkspaceEvents;
    use crate::workspace::governance::{Governance, Principal};
    use crate::workspace::text::ui_text;

    #[derive(Default)]
    struct Events(Mutex<Vec<&'static str>>);

    impl WorkspaceEvents for Events {
        fn emit(&self, event: &'static str, _payload: serde_json::Value) {
            self.0.lock().unwrap().push(event);
        }
    }

    struct RefuseEdits;

    impl AuthorizationProvider for RefuseEdits {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn authorize(&self, context: &ActionContext<'_>) -> Decision {
            if context.action == GovernedAction::EditDocuments {
                Decision::Deny(ui_text("test.refused"))
            } else {
                Decision::Allow
            }
        }
    }

    struct NoGrants;

    impl EntitlementProvider for NoGrants {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn granted_capabilities(&self) -> Vec<String> {
            Vec::new()
        }
    }

    fn save(collection: &str, id: &str) -> SaveDocumentRequest {
        SaveDocumentRequest {
            collection: collection.to_string(),
            document: Document {
                id: id.to_string(),
                body: "{}".to_string(),
                ..Default::default()
            }
            .into(),
            scope: Default::default(),
            ..Default::default()
        }
    }

    fn service(governance: Governance) -> (tempfile::TempDir, Arc<Governance>, DocumentsService) {
        let root = tempfile::tempdir().unwrap();
        let governance = Arc::new(governance);
        let service = DocumentsService::new(
            Arc::new(FolderDocumentStore::new(root.path())),
            Arc::clone(&governance),
            Arc::new(Events::default()),
        );
        service.reserve("device-profiles");
        (root, governance, service)
    }

    #[test]
    fn audits_every_saved_and_removed_document() {
        tauri::async_runtime::block_on(async {
            let (_root, governance, service) = service(Governance::core());
            let principal = governance.principal("main");

            service
                .save(save("scripts", "check"), &principal)
                .await
                .unwrap();
            service
                .remove(
                    RemoveDocumentRequest {
                        collection: "scripts".to_string(),
                        id: "check".to_string(),
                        scope: Default::default(),
                        ..Default::default()
                    },
                    &principal,
                )
                .await
                .unwrap();

            service
                .remove(
                    RemoveDocumentRequest {
                        collection: "scripts".to_string(),
                        id: "check".to_string(),
                        scope: Default::default(),
                        ..Default::default()
                    },
                    &principal,
                )
                .await
                .unwrap();

            let trail = governance.state().audit_trail;
            assert_eq!(
                trail.len(),
                2,
                "removing what is already gone changes nothing"
            );
            assert!(trail.iter().all(|entry| {
                entry.action == GovernedAction::EditDocuments
                    && entry.outcome == AuditOutcome::Succeeded
            }));
        });
    }

    #[test]
    fn refuses_and_audits_local_api_writes_a_collection_check_rejects() {
        tauri::async_runtime::block_on(async {
            let (root, governance, service) = service(Governance::core());
            service.check_api_writes("scripts", |_| Some(ui_text("test.hostFiles")));
            let api = Principal::new("api:robot", Default::default());

            let refused = service.save(save("scripts", "check"), &api).await;
            assert!(
                matches!(refused, Err(DocumentsError::Denied(reason)) if reason.key == "test.hostFiles")
            );
            assert!(std::fs::read_dir(root.path()).unwrap().next().is_none());
            assert_eq!(
                governance.state().audit_trail[0].outcome,
                AuditOutcome::Denied
            );

            service
                .save(save("scripts", "check"), &governance.principal("main"))
                .await
                .unwrap();
        });
    }

    #[test]
    fn refuses_edits_the_policy_denies_and_collections_other_modules_own() {
        tauri::async_runtime::block_on(async {
            let (root, governance, service) =
                service(Governance::new(Box::new(RefuseEdits), Box::new(NoGrants)));
            let principal = governance.principal("main");

            let denied = service.save(save("scripts", "check"), &principal).await;
            assert!(
                matches!(denied, Err(DocumentsError::Denied(reason)) if reason.key == "test.refused")
            );

            let reserved = service
                .save(save("device-profiles", "device-1"), &principal)
                .await;
            assert!(matches!(reserved, Err(DocumentsError::Invalid(_))));
            let listed = service
                .list(ListDocumentsRequest {
                    collection: "device-profiles".to_string(),
                    scope: Default::default(),
                    ..Default::default()
                })
                .await;
            assert!(matches!(listed, Err(DocumentsError::Invalid(_))));

            assert!(std::fs::read_dir(root.path()).unwrap().next().is_none());
            assert_eq!(
                governance.state().audit_trail[0].outcome,
                AuditOutcome::Denied
            );
        });
    }
}
