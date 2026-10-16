use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use super::*;

fn completion(content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "created": 1,
        "model": "test-model-2026",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": "stop"
        }]
    })
}

fn configured(server: &MockServer, structured_output: StructuredOutput) -> SettingsStore {
    let settings = SettingsStore::in_memory(None);
    settings
        .save(
            AssistantSettings {
                base_url: format!("{}/v1", server.uri()),
                model: "test-model".to_string(),
                structured_output: structured_output.into(),
                ..AssistantSettings::default()
            },
            Some("sk-test".to_string()),
        )
        .unwrap();
    settings
}

fn body(request: &Request) -> serde_json::Value {
    serde_json::from_slice(&request.body).unwrap()
}

#[test]
fn asks_for_the_flow_schema_and_reads_the_answer() {
    tauri::async_runtime::block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(completion(
                r#"{"name":"Morning","steps":[{"kind":"BATCH_ACTION_KIND_LAUNCH_APP","value":"com.android.settings"}]}"#,
            )))
            .expect(1)
            .mount(&server)
            .await;
        let harness = harness(
            Vec::new(),
            configured(&server, StructuredOutput::JsonSchema),
        );
        let principal = harness.governance.principal("main");

        let result = harness
            .service
            .draft_flow(
                DraftFlowRequest {
                    prompt: "open settings".to_string(),
                    scope: Default::default(),
                    ..Default::default()
                },
                &principal,
            )
            .await;

        assert!(result.accepted, "{:?}", result.detail);
        let draft = result.draft.unwrap();
        assert_eq!(draft.provider_id, OPENAI_COMPATIBLE_ID);
        assert_eq!(draft.model, "test-model-2026");
        assert_eq!(draft.flow.unwrap().steps[0].value, "com.android.settings");

        let sent = body(&server.received_requests().await.unwrap()[0]);
        assert_eq!(sent["model"], "test-model");
        assert_eq!(sent["response_format"]["type"], "json_schema");
        assert_eq!(sent["response_format"]["json_schema"]["name"], "flow_draft");
        assert_eq!(sent["response_format"]["json_schema"]["strict"], true);
        assert_eq!(sent["messages"][0]["role"], "system");
        assert_eq!(sent["messages"][1]["content"], "open settings");
    });
}

#[test]
fn falls_back_to_json_mode_for_endpoints_without_schemas() {
    tauri::async_runtime::block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(completion(
                "```json\n{\"name\":\"Battery\",\"body\":\"dumpsys battery\",\"explanation\":\"\"}\n```",
            )))
            .mount(&server)
            .await;
        let harness = harness(
            Vec::new(),
            configured(&server, StructuredOutput::JsonObject),
        );

        let result = harness
            .service
            .draft_script(
                DraftScriptRequest {
                    prompt: "print the battery".to_string(),
                    ..DraftScriptRequest::default()
                },
                &harness.governance.principal("main"),
            )
            .await;

        assert_eq!(
            result.draft.unwrap().script.unwrap().body,
            "dumpsys battery"
        );
        let sent = body(&server.received_requests().await.unwrap()[0]);
        assert_eq!(sent["response_format"]["type"], "json_object");
        assert!(
            sent["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("JSON Schema"),
            "JSON mode still gets the schema in the prompt"
        );
    });
}

#[test]
fn reports_what_the_endpoint_said_and_audits_the_failure() {
    tauri::async_runtime::block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {
                    "message": "Incorrect API key provided",
                    "type": "invalid_request_error",
                    "param": null,
                    "code": "invalid_api_key"
                }
            })))
            .mount(&server)
            .await;
        let harness = harness(
            Vec::new(),
            configured(&server, StructuredOutput::JsonSchema),
        );

        let result = harness
            .service
            .draft_flow(
                DraftFlowRequest {
                    prompt: "open settings".to_string(),
                    scope: Default::default(),
                    ..Default::default()
                },
                &harness.governance.principal("main"),
            )
            .await;

        assert_eq!(
            result.detail,
            ui_text_with_params(
                ASSISTANT_REQUEST_FAILED_KEY,
                "",
                [("message", "Incorrect API key provided".to_string())]
            )
            .into()
        );
        assert_eq!(
            harness.audited(GovernedAction::UseAssistant)[0].outcome,
            AuditOutcome::Failed
        );
    });
}

#[test]
fn needs_a_key_only_for_endpoints_off_this_computer() {
    let provider = |base_url: &str, key: Option<&str>| {
        let settings = SettingsStore::in_memory(key);
        settings
            .save(
                AssistantSettings {
                    base_url: base_url.to_string(),
                    model: "qwen2.5".to_string(),
                    ..AssistantSettings::default()
                },
                None,
            )
            .unwrap();
        OpenAiCompatible::new(Arc::new(settings)).describe()
    };

    assert!(provider("http://localhost:11434/v1", None).ready);
    assert!(provider("http://[::1]:11434/v1", None).ready);
    assert_eq!(
        provider("https://api.deepseek.com/v1", None).problem,
        ui_text(ASSISTANT_KEY_MISSING_KEY).into()
    );
    assert!(
        provider("https://api.deepseek.com/v1", Some("from-environment")).ready,
        "a key in the environment counts"
    );
    assert_eq!(
        provider("ftp://example.com", Some("k")).problem,
        ui_text(ASSISTANT_BASE_URL_INVALID_KEY).into()
    );
}

#[test]
fn sends_each_key_only_to_the_service_it_belongs_to() {
    let at = |settings: &SettingsStore, base_url: &str, api_key: Option<&str>| {
        settings
            .save(
                AssistantSettings {
                    base_url: base_url.to_string(),
                    model: "m".to_string(),
                    ..AssistantSettings::default()
                },
                api_key.map(str::to_string),
            )
            .unwrap();
        settings.endpoint().api_key
    };

    let openai_only = SettingsStore::in_memory_with(None, Some("sk-openai"));
    assert_eq!(
        at(&openai_only, "https://api.openai.com/v1", None).as_deref(),
        Some("sk-openai")
    );
    assert_eq!(at(&openai_only, "http://192.168.1.20:11434/v1", None), None);
    assert!(!openai_only.settings().api_key_set);

    let app = SettingsStore::in_memory_with(Some("app-key"), Some("sk-openai"));
    assert_eq!(
        at(&app, "https://api.deepseek.com/v1", None).as_deref(),
        Some("app-key")
    );

    let stored = SettingsStore::in_memory(None);
    assert_eq!(
        at(&stored, "https://api.deepseek.com/v1", Some("sk-deepseek")).as_deref(),
        Some("sk-deepseek")
    );
    assert_eq!(
        at(&stored, "https://api.deepseek.com/beta", None).as_deref(),
        Some("sk-deepseek"),
        "the same service keeps its key"
    );
    assert_eq!(
        at(&stored, "https://evil.example/v1", None),
        None,
        "another service never receives it"
    );
}

#[test]
fn keeps_the_key_in_the_backend_and_audits_setting_changes() {
    let directory = tempfile::tempdir().unwrap();
    let harness = harness(
        Vec::new(),
        SettingsStore::load(directory.path().to_path_buf()),
    );
    let update = |settings: AssistantSettings, api_key: Option<&str>| {
        tauri::async_runtime::block_on(harness.service.update(
            UpdateAssistantRequest {
                settings: settings.into(),
                api_key: api_key.map(str::to_string),
                scope: Default::default(),
                ..Default::default()
            },
            &harness.governance.principal("main"),
        ))
    };
    let settings = AssistantSettings {
        base_url: "https://api.deepseek.com/v1".to_string(),
        model: "deepseek-chat".to_string(),
        structured_output: StructuredOutput::JsonObject.into(),
        ..AssistantSettings::default()
    };

    let saved = update(settings.clone(), Some(" sk-secret "));
    assert!(saved.accepted, "{:?}", saved.detail);
    let state = saved.state.unwrap();
    assert!(!format!("{state:?}").contains("sk-secret"));
    assert!(state.settings.unwrap().api_key_set);
    assert!(state.providers.last().unwrap().ready);

    let reloaded = SettingsStore::load(directory.path().to_path_buf());
    assert_eq!(reloaded.endpoint().api_key.as_deref(), Some("sk-secret"));
    assert_eq!(reloaded.settings().model, "deepseek-chat");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(directory.path().join("credentials.json"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "only this user may read the key");
    }

    assert!(update(settings.clone(), None).accepted);
    assert!(
        harness.settings.settings().api_key_set,
        "absent keeps the key"
    );
    assert!(update(settings.clone(), Some("")).accepted);
    assert!(!harness.settings.settings().api_key_set, "empty removes it");
    assert_eq!(
        update(settings.clone(), Some("sk-\nsecret")).detail,
        ui_text(ASSISTANT_KEY_INVALID_KEY).into()
    );

    let refused = update(
        AssistantSettings {
            base_url: "ftp://example.com".to_string(),
            ..settings.clone()
        },
        None,
    );
    assert_eq!(
        refused.detail,
        ui_text(ASSISTANT_BASE_URL_INVALID_KEY).into()
    );
    let unknown = update(
        AssistantSettings {
            provider_id: "nobody".to_string(),
            ..settings
        },
        None,
    );
    assert_eq!(
        unknown.detail,
        ui_text(ASSISTANT_PROVIDER_UNKNOWN_KEY).into()
    );
    assert_eq!(
        harness.audited(GovernedAction::ManageAssistant).len(),
        3,
        "refused requests never reach governance"
    );
}

fn chunk(content: &str, finish: Option<&str>) -> String {
    let chunk = serde_json::json!({
        "id": "chatcmpl-1",
        "object": "chat.completion.chunk",
        "created": 1,
        "model": "test-model-2026",
        "choices": [{ "index": 0, "delta": { "content": content }, "finish_reason": finish }]
    });
    format!("data: {chunk}\n\n")
}

#[test]
fn streams_the_answer_while_the_model_writes_it() {
    tauri::async_runtime::block_on(async {
        let server = MockServer::start().await;
        let events = [
            chunk(
                r#"{"name":"Morning","steps":[{"kind":"BATCH_ACTION_KIND_LAUNCH_APP","#,
                None,
            ),
            chunk(r#""value":"com.android.settings"}]}"#, Some("stop")),
            "data: [DONE]\n\n".to_string(),
        ]
        .concat();
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(events),
            )
            .expect(1)
            .mount(&server)
            .await;
        let harness = harness(
            Vec::new(),
            configured(&server, StructuredOutput::JsonSchema),
        );
        let pieces = std::sync::Mutex::new(Vec::new());
        let progress = |piece: &str| pieces.lock().unwrap().push(piece.to_string());

        let result = harness
            .service
            .draft_flow_with(
                DraftFlowRequest {
                    prompt: "open settings".to_string(),
                    scope: Default::default(),
                    ..Default::default()
                },
                &harness.governance.principal("main"),
                Some(&progress),
            )
            .await;

        assert!(result.accepted, "{:?}", result.detail);
        assert_eq!(
            result.draft.unwrap().flow.unwrap().steps[0].value,
            "com.android.settings"
        );
        let pieces = pieces.into_inner().unwrap();
        assert_eq!(pieces.len(), 2);
        assert!(pieces[0].starts_with(r#"{"name":"Morning""#));
        let sent = body(&server.received_requests().await.unwrap()[0]);
        assert_eq!(sent["stream"], true);
    });
}
