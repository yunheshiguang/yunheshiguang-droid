use super::*;

#[derive(Default)]
struct RecordingEvents(Mutex<Vec<(String, serde_json::Value)>>);

impl WorkspaceEvents for RecordingEvents {
    fn emit(&self, event: &'static str, payload: serde_json::Value) {
        self.0.lock().unwrap().push((event.to_string(), payload));
    }
}

struct Grants(&'static str);

impl EntitlementProvider for Grants {
    fn describe(&self) -> GovernanceProvider {
        provider_info("test.grants", "test", "test")
    }

    fn granted_capabilities(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

struct Market;

impl TemplateSource for Market {
    fn describe(&self) -> TemplateSourceInfo {
        TemplateSourceInfo {
            id: "test.market".to_string(),
            ..TemplateSourceInfo::default()
        }
    }

    fn required_capability(&self) -> Option<&str> {
        Some("test.market")
    }

    fn templates(&self) -> Vec<FlowTemplate> {
        Vec::new()
    }

    fn packages(&self) -> Vec<TemplatePackage> {
        vec![package()]
    }
}

struct Harness {
    service: Arc<PackagesService>,
    governance: Arc<Governance>,
    events: Arc<RecordingEvents>,
    documents: Arc<FolderDocumentStore>,
    directory: TempDir,
}

fn harness(granted: &'static str) -> Harness {
    let directory = tempfile::tempdir().unwrap();
    let documents = Arc::new(FolderDocumentStore::new(directory.path().join("documents")));
    let governance = Arc::new(Governance::assemble(GovernanceParts {
        entitlements: Some(Box::new(Grants(granted))),
        ..GovernanceParts::default()
    }));
    let events = Arc::new(RecordingEvents::default());

    Harness {
        service: Arc::new(PackagesService {
            documents: Arc::clone(&documents) as Arc<dyn DocumentStore>,
            catalog: Arc::new(TemplateCatalog::new(vec![Box::new(Market)])),
            governance: Arc::clone(&governance),
            telemetry: Arc::new(Telemetry::assemble(Vec::new())),
            events: Arc::clone(&events) as Arc<dyn WorkspaceEvents>,
            keys: Arc::new(super::super::signing::SigningKeys::new(
                directory.path().join("keys"),
            )),
        }),
        governance,
        events,
        documents,
        directory,
    }
}

fn save(store: &dyn DocumentStore, collection: &str, id: &str, value: &impl serde::Serialize) {
    store
        .save(
            &Collection::parse(collection).unwrap(),
            &Document::new(
                DocumentId::parse(id).unwrap(),
                serde_json::to_string(value).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
}

#[test]
fn exports_flows_with_their_scripts_and_installs_them_elsewhere() {
    let source = harness("none");
    save(
        source.documents.as_ref(),
        SCRIPTS_COLLECTION,
        "battery",
        &script("Battery"),
    );
    save(
        source.documents.as_ref(),
        FLOWS_COLLECTION,
        "nightly",
        &flow("Nightly", "battery"),
    );
    let path = source.directory.path().join("nightly.json");
    let principal = source.governance.principal("main");

    let exported = tauri::async_runtime::block_on(source.service.export(
        ExportPackageRequest {
            manifest: manifest(" com.example.nightly ", "1.2.0").into(),
            flow_ids: vec!["nightly".to_string()],
            path: path.display().to_string(),
            scope: Default::default(),
            ..Default::default()
        },
        &principal,
    ));
    assert!(exported.accepted, "{:?}", exported.detail);
    assert_eq!((exported.flows, exported.scripts), (1, 1));

    let target = harness("none");
    let origin = Some(PackageOrigin {
        origin: Some(package_origin::Origin::Path(path.display().to_string())),
        ..Default::default()
    });
    let inspected = tauri::async_runtime::block_on(target.service.inspect(InspectPackageRequest {
        origin: origin.clone().into(),
        scope: Default::default(),
        ..Default::default()
    }));
    let shown = inspected.preview.unwrap();
    assert!(!shown.verified, "a signer this computer does not trust");
    assert_eq!(shown.problems, [ui_text(PACKAGES_UNVERIFIED_KEY)]);
    assert_eq!(shown.manifest.unwrap().id, "com.example.nightly");

    let signer = shown.signer.expect("an exported package is signed");
    assert_eq!(signer.name, principal.operator.name);
    assert!(!signer.trusted);
    let trusted = tauri::async_runtime::block_on(target.service.trust_signer(
        TrustPackageSignerRequest {
            public_key: signer.public_key,
            name: "Source".to_string(),
            scope: Default::default(),
            ..Default::default()
        },
        &target.governance.principal("main"),
    ));
    assert!(trusted.accepted, "{:?}", trusted.detail);
    let again = tauri::async_runtime::block_on(target.service.inspect(InspectPackageRequest {
        origin: origin.clone().into(),
        scope: Default::default(),
        ..Default::default()
    }));
    let again = again.preview.unwrap();
    assert!(again.verified && again.problems.is_empty());
    let signer = again.signer.unwrap();
    assert!(signer.trusted);
    assert_eq!(signer.name, "Source", "the name it was trusted under");

    let installed = tauri::async_runtime::block_on(target.service.install(
        InstallPackageRequest {
            origin: origin.into(),
            on_conflict: ConflictPolicy::KeepBoth.into(),
            scope: Default::default(),
            ..Default::default()
        },
        &target.governance.principal("main"),
    ));
    assert!(installed.accepted, "{:?}", installed.detail);
    let library = read_library(target.documents.as_ref()).unwrap();
    assert_eq!(library.flows["nightly"].steps[1].value, "battery");
    assert_eq!(library.scripts["battery"].body, "dumpsys battery");
    assert_eq!(
        target.events.0.lock().unwrap().len(),
        2,
        "both collections changed"
    );
    for harness in [&source, &target] {
        let entries = harness.governance.state().audit_trail;
        assert_eq!(entries[0].action, GovernedAction::ManagePackages);
        assert_eq!(entries[0].outcome, AuditOutcome::Succeeded);
    }
}

#[test]
fn refuses_exports_that_would_leave_something_behind() {
    let source = harness("none");
    save(
        source.documents.as_ref(),
        FLOWS_COLLECTION,
        "nightly",
        &flow("Nightly", "gone"),
    );
    let export = |flow_ids: Vec<String>, version: &str| {
        tauri::async_runtime::block_on(
            source.service.export(
                ExportPackageRequest {
                    manifest: manifest("com.example.nightly", version).into(),
                    flow_ids,
                    path: source
                        .directory
                        .path()
                        .join("out.json")
                        .display()
                        .to_string(),
                    scope: Default::default(),
                    ..Default::default()
                },
                &source.governance.principal("main"),
            ),
        )
        .detail
        .unwrap()
        .key
    };

    assert_eq!(
        export(vec!["nightly".to_string()], "1"),
        PACKAGES_VERSION_INVALID_KEY
    );
    assert_eq!(export(Vec::new(), "1.0.0"), PACKAGES_NO_FLOWS_KEY);
    assert!(source.governance.state().audit_trail.is_empty());
    assert_eq!(
        export(vec!["nightly".to_string()], "1.0.0"),
        PACKAGES_SCRIPT_MISSING_KEY
    );
    assert_eq!(
        export(vec!["absent".to_string()], "1.0.0"),
        PACKAGES_FLOW_MISSING_KEY
    );
}

#[test]
fn offers_market_packages_only_when_their_capability_is_granted() {
    let locked = harness("none");
    assert!(locked.service.market().packages.is_empty());

    let unlocked = harness("test.market");
    let listed = unlocked.service.market().packages;
    assert_eq!(listed.len(), 1);
    assert_eq!((listed[0].flows, listed[0].scripts), (1, 1));

    let installed = tauri::async_runtime::block_on(
        unlocked.service.install(
            InstallPackageRequest {
                origin: PackageOrigin {
                    origin: Some(package_origin::Origin::Market(Box::new(MarketRef {
                        source_id: "test.market".to_string(),
                        package_id: "com.example.nightly".to_string(),
                        ..Default::default()
                    }))),
                    ..Default::default()
                }
                .into(),
                on_conflict: ConflictPolicy::Unspecified.into(),
                scope: Default::default(),
                ..Default::default()
            },
            &unlocked.governance.principal("main"),
        ),
    );
    assert!(installed.accepted, "{:?}", installed.detail);
    assert_eq!(installed.flows, 1);

    let missing = tauri::async_runtime::block_on(
        locked.service.inspect(InspectPackageRequest {
            origin: PackageOrigin {
                origin: Some(package_origin::Origin::Market(Box::new(MarketRef {
                    source_id: "test.market".to_string(),
                    package_id: "com.example.nightly".to_string(),
                    ..Default::default()
                }))),
                ..Default::default()
            }
            .into(),
            scope: Default::default(),
            ..Default::default()
        }),
    );
    assert_eq!(missing.detail, ui_text(PACKAGES_MARKET_MISSING_KEY).into());
}

#[test]
fn refuses_local_api_packages_whose_flows_touch_files_on_this_computer() {
    let target = harness("none");
    let mut leaking = flow("Leak", "battery");
    leaking.steps.push(step(
        BatchActionKind::UploadFile,
        "/home/me/.ssh/id_ed25519",
    ));
    let mut bundle = package();
    bundle.flows[0].flow = leaking.into();
    let text = serde_json::to_string(&bundle).unwrap();
    let request = || InstallPackageRequest {
        origin: PackageOrigin {
            origin: Some(package_origin::Origin::Json(text.clone())),
            ..Default::default()
        }
        .into(),
        on_conflict: ConflictPolicy::KeepBoth.into(),
        ..Default::default()
    };

    let refused = tauri::async_runtime::block_on(
        target
            .service
            .install(request(), &Principal::new("api:robot", Default::default())),
    );
    assert!(!refused.accepted);
    assert_eq!(refused.detail.unwrap().key, API_HOST_FILES_KEY);
    assert!(
        read_library(target.documents.as_ref())
            .unwrap()
            .flows
            .is_empty()
    );
    assert_eq!(
        target.governance.state().audit_trail[0].outcome,
        AuditOutcome::Denied
    );

    let installed = tauri::async_runtime::block_on(
        target
            .service
            .install(request(), &target.governance.principal("main")),
    );
    assert!(installed.accepted, "{:?}", installed.detail);
}
