mod openai;

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use http::Request;
use http_body_util::Full;
use tower::ServiceExt;
use yunheshiguang_contract::Message;

use tempfile::TempDir;
use yunheshiguang_sdk::assistant::{
    AssistantProvider, AssistantProviderInfo, Completion, CompletionFuture, CompletionRequest,
};
use yunheshiguang_sdk::documents::{
    Collection, Document, DocumentId, DocumentStore, FolderDocumentStore,
};

use super::drafts::{check_flow, check_script};
use super::openai::{OPENAI_COMPATIBLE_ID, OpenAiCompatible};
use super::prompts::MAX_DRAFT_STEPS;
use super::rpc::AssistantRpc;
use super::service::AssistantService;
use super::settings::SettingsStore;
use crate::workspace::contract::*;
use crate::workspace::governance::Governance;
use crate::workspace::messages::*;
use crate::workspace::rpc::{Caller, RpcRoutes};
use crate::workspace::scripts::SCRIPTS_COLLECTION;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

type Asked = Arc<Mutex<Vec<(String, String)>>>;

struct Canned {
    answer: String,
    asked: Asked,
}

impl Canned {
    fn answering(answer: &str) -> (Self, Asked) {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let canned = Self {
            answer: answer.to_string(),
            asked: Arc::clone(&asked),
        };

        (canned, asked)
    }
}

impl AssistantProvider for Canned {
    fn describe(&self) -> AssistantProviderInfo {
        AssistantProviderInfo {
            id: "test.canned".to_string(),
            ready: true,
            ..AssistantProviderInfo::default()
        }
    }

    fn complete<'a>(&'a self, request: CompletionRequest<'a>) -> CompletionFuture<'a> {
        self.asked
            .lock()
            .unwrap()
            .push((request.system.to_string(), request.prompt.to_string()));
        let answer = self.answer.clone();

        Box::pin(async move { Ok(Completion::new(answer, "canned-model")) })
    }
}

struct Harness {
    service: Arc<AssistantService>,
    governance: Arc<Governance>,
    settings: Arc<SettingsStore>,
    _documents: TempDir,
}

fn harness(extension: Vec<Box<dyn AssistantProvider>>, settings: SettingsStore) -> Harness {
    let directory = tempfile::tempdir().unwrap();
    let documents = Arc::new(FolderDocumentStore::new(directory.path()));
    documents
        .save(
            &Collection::parse(SCRIPTS_COLLECTION).unwrap(),
            &Document::new(
                DocumentId::parse("cleanup").unwrap(),
                r#"{"name":"Clean up","language":"SCRIPT_LANGUAGE_SHELL","body":"pm trim-caches 1G"}"#,
            )
            .unwrap(),
        )
        .unwrap();
    let settings = Arc::new(settings);
    let governance = Arc::new(Governance::core());
    let mut providers = extension;
    providers.push(Box::new(OpenAiCompatible::new(Arc::clone(&settings))));

    Harness {
        service: Arc::new(AssistantService {
            providers,
            settings: Arc::clone(&settings),
            documents,
            governance: Arc::clone(&governance),
            telemetry: Arc::new(Telemetry::assemble(Vec::new())),
        }),
        governance,
        settings,
        _documents: directory,
    }
}

impl Harness {
    fn draft_flow(&self, prompt: &str) -> DraftFlowResult {
        let request = DraftFlowRequest {
            prompt: prompt.to_string(),
            scope: Default::default(),
            ..Default::default()
        };

        tauri::async_runtime::block_on(
            self.service
                .draft_flow(request, &self.governance.principal("main")),
        )
    }

    fn audited(&self, action: GovernedAction) -> Vec<AuditEntry> {
        self.governance
            .state()
            .audit_trail
            .into_iter()
            .filter(|entry| entry.action == action)
            .collect()
    }
}

fn scripts() -> HashSet<String> {
    HashSet::from(["cleanup".to_string()])
}

#[test]
fn keeps_known_steps_and_flags_what_needs_attention() {
    let answer = r#"```json
{"name": "  早班巡检 ", "steps": [
  {"kind": "BATCH_ACTION_KIND_LAUNCH_APP", "value": " com.android.settings "},
  {"kind": "BATCH_ACTION_KIND_INPUT_TEXT", "value": " hi\n"},
  {"kind": "BATCH_ACTION_KIND_TAP", "value": "100,200"},
  {"kind": "BATCH_ACTION_KIND_INSTALL_APK", "value": ""},
  {"kind": "BATCH_ACTION_KIND_RUN_SCRIPT", "value": "cleanup"},
  {"kind": "BATCH_ACTION_KIND_RUN_SCRIPT", "value": "invented"}
]}
```"#;

    let (flow, issues) = check_flow(answer, &scripts()).unwrap();

    assert_eq!(flow.name, "早班巡检");
    let values = flow
        .steps
        .iter()
        .map(|step| step.value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        ["com.android.settings", " hi\n", "", "cleanup", "invented"],
        "text stays as written, other values are trimmed, unknown actions are dropped"
    );
    let flagged = issues
        .iter()
        .map(|issue| (issue.step_id.clone(), issue.message.clone().unwrap().key))
        .collect::<Vec<_>>();
    assert_eq!(
        flagged,
        [
            (
                String::new(),
                ASSISTANT_DRAFT_UNKNOWN_ACTION_KEY.to_string()
            ),
            (
                flow.steps[2].id.clone(),
                VALIDATION_NEED_PATH_KEY.to_string()
            ),
            (
                flow.steps[4].id.clone(),
                ASSISTANT_DRAFT_SCRIPT_MISSING_KEY.to_string()
            ),
        ]
    );
    assert_eq!(
        issues[0].message,
        ui_text_with_params(
            ASSISTANT_DRAFT_UNKNOWN_ACTION_KEY,
            "",
            [("step", "3".to_string())]
        )
        .into()
    );
}

#[test]
fn caps_long_drafts_and_explains_empty_or_unreadable_ones() {
    let step = r#"{"kind":"BATCH_ACTION_KIND_RECOVER_CONNECTION","value":""}"#;
    let long = format!(
        r#"{{"name":"n","steps":[{}]}}"#,
        vec![step; MAX_DRAFT_STEPS + 5].join(",")
    );
    let (flow, issues) = check_flow(&long, &scripts()).unwrap();
    assert_eq!(flow.steps.len(), MAX_DRAFT_STEPS);
    assert_eq!(
        issues[0].message.as_option().unwrap().key,
        ASSISTANT_DRAFT_TOO_LONG_KEY
    );

    let (_, issues) = check_flow(r#"{"name":"n","steps":[]}"#, &scripts()).unwrap();
    assert_eq!(issues[0].message, ui_text(ASSISTANT_DRAFT_EMPTY_KEY).into());

    let unreadable = check_flow("I cannot help with that.", &scripts()).unwrap_err();
    assert_eq!(unreadable.key, ASSISTANT_UNREADABLE_KEY);
}

#[test]
fn reads_script_drafts_and_refuses_empty_ones() {
    let (script, explanation) = check_script(
        r#"{"name":"电量","body":"dumpsys battery | grep level\n\n","explanation":"打印电量"}"#,
        ScriptLanguage::Shell,
    )
    .unwrap();
    assert_eq!(script.body, "dumpsys battery | grep level");
    assert_eq!(script.language, ScriptLanguage::Shell);
    assert_eq!(explanation, "打印电量");

    assert_eq!(
        check_script(
            r#"{"name":"x","body":"  ","explanation":""}"#,
            ScriptLanguage::Shell
        )
        .unwrap_err(),
        ui_text(ASSISTANT_SCRIPT_EMPTY_KEY)
    );
}

#[test]
fn drafts_through_the_first_ready_provider_and_audits_the_model() {
    let (canned, asked) = Canned::answering(
        r#"{"name":"Clean","steps":[{"kind":"BATCH_ACTION_KIND_RUN_SCRIPT","value":"cleanup"}]}"#,
    );
    let harness = harness(vec![Box::new(canned)], SettingsStore::in_memory(None));

    let result = harness.draft_flow("清理缓存");

    assert!(result.accepted, "{:?}", result.detail);
    let draft = result.draft.unwrap();
    assert_eq!(draft.provider_id, "test.canned");
    assert_eq!(draft.model, "canned-model");
    assert!(draft.issues.is_empty());
    let (system, prompt) = asked.lock().unwrap()[0].clone();
    assert_eq!(prompt, "清理缓存");
    assert!(
        system.contains("- cleanup: Clean up"),
        "saved scripts are offered"
    );
    assert!(system.contains("BATCH_ACTION_KIND_LAUNCH_APP"));

    let entries = harness.audited(GovernedAction::UseAssistant);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].outcome, AuditOutcome::Succeeded);
    assert_eq!(
        entries[0].detail,
        ui_text_with_params(
            ASSISTANT_AUDIT_FLOW_KEY,
            "",
            [("model", "canned-model".to_string())]
        )
        .into()
    );
}

#[test]
fn explains_what_the_chosen_provider_lacks_before_asking_anything() {
    let (canned, asked) = Canned::answering("{}");
    let harness = harness(vec![Box::new(canned)], SettingsStore::in_memory(None));
    let choose_core = AssistantSettings {
        provider_id: OPENAI_COMPATIBLE_ID.to_string(),
        ..harness.settings.settings()
    };
    harness.settings.save(choose_core, None).unwrap();

    let result = harness.draft_flow("清理缓存");

    assert_eq!(result.detail, ui_text(ASSISTANT_MODEL_MISSING_KEY).into());
    assert!(asked.lock().unwrap().is_empty());
    assert!(harness.audited(GovernedAction::UseAssistant).is_empty());
    assert_eq!(
        harness.draft_flow("   ").detail,
        ui_text(ASSISTANT_PROMPT_REQUIRED_KEY).into()
    );
}

#[test]
fn drafts_scripts_with_the_current_body_as_context() {
    let (canned, asked) = Canned::answering(
        r#"{"name":"Open","body":"app.launch(\"com.android.settings\");","explanation":"opens"}"#,
    );
    let harness = harness(vec![Box::new(canned)], SettingsStore::in_memory(None));
    let request = DraftScriptRequest {
        prompt: "also open settings".to_string(),
        language: ScriptLanguage::Javascript.into(),
        current_body: "toast(\"hi\");".to_string(),
        scope: Default::default(),
        ..Default::default()
    };

    let result = tauri::async_runtime::block_on(
        harness
            .service
            .draft_script(request, &harness.governance.principal("main")),
    );

    let draft = result.draft.unwrap();
    assert_eq!(draft.script.unwrap().language, ScriptLanguage::Javascript);
    assert_eq!(draft.explanation, "opens");
    let (system, prompt) = asked.lock().unwrap()[0].clone();
    assert!(system.contains("AutoJs6"));
    assert!(prompt.contains("toast(\"hi\");"));
    assert_eq!(
        harness.audited(GovernedAction::UseAssistant)[0]
            .detail
            .as_option()
            .unwrap()
            .key,
        ASSISTANT_AUDIT_SCRIPT_KEY
    );
}

struct Hanging {
    started: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}

struct SetOnDrop(Arc<AtomicBool>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl AssistantProvider for Hanging {
    fn describe(&self) -> AssistantProviderInfo {
        AssistantProviderInfo {
            id: "test.hanging".to_string(),
            ready: true,
            ..AssistantProviderInfo::default()
        }
    }

    fn complete<'a>(&'a self, _request: CompletionRequest<'a>) -> CompletionFuture<'a> {
        self.started.store(true, Ordering::SeqCst);
        let dropped = SetOnDrop(Arc::clone(&self.dropped));

        Box::pin(async move {
            let _dropped = dropped;
            std::future::pending().await
        })
    }
}

#[test]
fn stops_asking_the_model_once_the_caller_stops_listening() {
    let started = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicBool::new(false));
    let harness = harness(
        vec![Box::new(Hanging {
            started: Arc::clone(&started),
            dropped: Arc::clone(&dropped),
        })],
        SettingsStore::in_memory(None),
    );
    let routes = RpcRoutes::new();
    let rpc = Arc::new(AssistantRpc {
        service: Arc::clone(&harness.service),
        governance: Arc::clone(&harness.governance),
    });
    routes.add(|router| router.add_service(rpc));
    let message = DraftFlowRequest {
        prompt: "reboot every phone".to_string(),
        ..Default::default()
    }
    .encode_to_vec();
    let mut framed = vec![0];
    framed.extend(u32::try_from(message.len()).unwrap().to_be_bytes());
    framed.extend(message);
    let mut request = Request::builder()
        .method("POST")
        .uri("/yunheshiguang.assistant.v1.AssistantService/DraftFlow")
        .header("content-type", "application/connect+proto")
        .body(Full::new(Bytes::from(framed)))
        .unwrap();
    request
        .extensions_mut()
        .insert(Caller::Window("main".to_string()));

    tauri::async_runtime::block_on(async {
        let response = routes.service().oneshot(request).await.unwrap();
        for _ in 0..200 {
            if started.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(started.load(Ordering::SeqCst));

        drop(response);
        for _ in 0..200 {
            if dropped.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    assert!(dropped.load(Ordering::SeqCst));
}
