use std::path::Path;
use std::time::Duration;

use futures_util::StreamExt;
use genai::chat::{
    ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, ChatStreamEvent, JsonSpec,
};
use genai::resolver::{AuthData, Endpoint};
use genai::{ModelIden, ServiceTarget};
use serde::Deserialize;
use yunheshiguang_sdk::assistant::{
    AssistantProvider, AssistantProviderInfo, Completion, CompletionFuture, CompletionProgress,
    CompletionRequest,
};
use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::UiText;
use yunheshiguang_sdk::text::{ui_text, ui_text_with_params};

use crate::config::read_json;
use crate::messages::*;
use crate::status::FileState;

pub const GENAI_PROVIDER_ID: &str = "reference.genai";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AssistantFile {
    model: String,
    #[serde(default)]
    api_key_env: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
}

pub struct GenaiAssistant {
    config: Result<AssistantFile, String>,
    client: genai::Client,
}

fn is_local(endpoint: &str) -> bool {
    endpoint.contains("://localhost")
        || endpoint.contains("://127.0.0.1")
        || endpoint.contains("://[::1]")
}

fn client(config: &AssistantFile) -> genai::Client {
    let mut http = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(IDLE_TIMEOUT);
    if config.endpoint.as_deref().is_some_and(is_local) {
        http = http.no_proxy();
    }
    let mut builder = genai::Client::builder();
    if let Ok(http) = http.build() {
        builder = builder.with_reqwest(http);
    }
    if let Some(variable) = config.api_key_env.clone() {
        builder = builder.with_auth_resolver_fn(move |_model: ModelIden| {
            Ok(Some(AuthData::from_env(variable.clone())))
        });
    }
    if let Some(endpoint) = config.endpoint.clone() {
        let endpoint = if endpoint.ends_with('/') {
            endpoint
        } else {
            format!("{endpoint}/")
        };
        builder = builder.with_service_target_resolver_fn(move |target: ServiceTarget| {
            Ok(ServiceTarget {
                endpoint: Endpoint::from_owned(endpoint.clone()),
                ..target
            })
        });
    }

    builder.build()
}

impl GenaiAssistant {
    pub fn load(path: &Path) -> (Option<Self>, FileState) {
        match read_json::<AssistantFile>(path) {
            Ok(None) => (None, FileState::Missing),
            Ok(Some(config)) => (Some(Self::new(Ok(config))), FileState::Loaded),
            Err(reason) => (
                Some(Self::new(Err(reason.clone()))),
                FileState::Broken(reason),
            ),
        }
    }

    fn new(config: Result<AssistantFile, String>) -> Self {
        let client = config
            .as_ref()
            .map_or_else(|_| genai::Client::default(), client);

        Self { config, client }
    }

    fn problem(&self) -> Option<UiText> {
        match &self.config {
            Err(reason) => Some(ui_text_with_params(
                REFERENCE_ASSISTANT_BROKEN_KEY,
                "",
                [("detail", reason.clone())],
            )),
            Ok(config) if config.model.trim().is_empty() => {
                Some(ui_text(REFERENCE_ASSISTANT_MODEL_MISSING_KEY))
            }
            Ok(config) => config
                .api_key_env
                .as_ref()
                .filter(|variable| std::env::var(variable).is_err())
                .map(|variable| {
                    ui_text_with_params(
                        REFERENCE_ASSISTANT_KEY_MISSING_KEY,
                        "",
                        [("variable", variable.clone())],
                    )
                }),
        }
    }
}

fn failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        REFERENCE_ASSISTANT_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

impl GenaiAssistant {
    fn chat(
        &self,
        request: &CompletionRequest<'_>,
    ) -> Result<(String, ChatRequest, ChatOptions), UiText> {
        if let Some(problem) = self.problem() {
            return Err(problem);
        }
        let model = self
            .config
            .as_ref()
            .map(|config| config.model.clone())
            .unwrap_or_default();
        let chat = ChatRequest::new(vec![
            ChatMessage::system(request.system),
            ChatMessage::user(request.prompt),
        ]);
        let options = ChatOptions::default().with_response_format(ChatResponseFormat::JsonSpec(
            JsonSpec::new(request.schema_name, request.schema.clone()),
        ));

        Ok((model, chat, options))
    }
}

impl AssistantProvider for GenaiAssistant {
    fn describe(&self) -> AssistantProviderInfo {
        let problem = self.problem();

        AssistantProviderInfo {
            id: GENAI_PROVIDER_ID.to_string(),
            label: ui_text(REFERENCE_ASSISTANT_LABEL_KEY).into(),
            description: ui_text(REFERENCE_ASSISTANT_DESCRIPTION_KEY).into(),
            configurable: false,
            ready: problem.is_none(),
            problem: problem.into(),
            ..Default::default()
        }
    }

    fn complete<'a>(&'a self, request: CompletionRequest<'a>) -> CompletionFuture<'a> {
        Box::pin(async move {
            let (model, chat, options) = self.chat(&request)?;
            let response = self
                .client
                .exec_chat(model.as_str(), chat, Some(&options))
                .await
                .map_err(failed)?;
            let answered = response.provider_model_iden.model_name.to_string();
            let text = response
                .into_first_text()
                .filter(|text| !text.trim().is_empty())
                .ok_or_else(|| ui_text(REFERENCE_ASSISTANT_EMPTY_KEY))?;

            Ok(Completion::new(text, answered))
        })
    }

    fn complete_streaming<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        progress: CompletionProgress<'a>,
    ) -> CompletionFuture<'a> {
        Box::pin(async move {
            let (model, chat, options) = self.chat(&request)?;
            let response = self
                .client
                .exec_chat_stream(model.as_str(), chat, Some(&options))
                .await
                .map_err(failed)?;
            let answered = response.model_iden.model_name.to_string();
            let mut stream = response.stream;
            let mut text = String::new();

            while let Some(event) = stream.next().await {
                if let ChatStreamEvent::Chunk(chunk) = event.map_err(failed)?
                    && !chunk.content.is_empty()
                {
                    progress(&chunk.content);
                    text.push_str(&chunk.content);
                }
            }

            if text.trim().is_empty() {
                return Err(ui_text(REFERENCE_ASSISTANT_EMPTY_KEY));
            }
            Ok(Completion::new(text, answered))
        })
    }
}

#[cfg(test)]
fn adapter_of(model: &str) -> Option<genai::adapter::AdapterKind> {
    genai::adapter::AdapterKind::from_model(model).ok()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn load(contents: &str) -> (Option<GenaiAssistant>, FileState) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("assistant.json");
        fs::write(&path, contents).unwrap();

        GenaiAssistant::load(&path)
    }

    #[test]
    fn answers_through_the_native_anthropic_api() {
        tauri::async_runtime::block_on(async {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/messages"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "id": "msg_1",
                    "type": "message",
                    "role": "assistant",
                    "model": "claude-test-2026",
                    "content": [{ "type": "text", "text": "{\"name\":\"n\",\"steps\":[]}" }],
                    "stop_reason": "end_turn",
                    "usage": { "input_tokens": 1, "output_tokens": 1 }
                })))
                .mount(&server)
                .await;
            unsafe { std::env::set_var("REFERENCE_TEST_ANTHROPIC_KEY", "sk-test") };
            let config = serde_json::json!({
                "model": "claude-sonnet-4-5",
                "apiKeyEnv": "REFERENCE_TEST_ANTHROPIC_KEY",
                "endpoint": format!("{}/v1", server.uri()),
            });
            let (assistant, state) = load(&config.to_string());
            let assistant = assistant.unwrap();
            assert_eq!(state, FileState::Loaded);
            assert!(assistant.describe().ready);

            let schema = serde_json::json!({ "type": "object" });
            let completion = assistant
                .complete(CompletionRequest::new(
                    "system",
                    "prompt",
                    "flow_draft",
                    &schema,
                ))
                .await
                .unwrap();

            assert_eq!(completion.json, "{\"name\":\"n\",\"steps\":[]}");
            assert_eq!(completion.model, "claude-test-2026");
            let sent = &server.received_requests().await.unwrap()[0];
            assert_eq!(sent.headers["x-api-key"], "sk-test");
        });
    }

    #[test]
    fn streams_what_the_anthropic_api_writes() {
        tauri::async_runtime::block_on(async {
            let server = MockServer::start().await;
            let events = [
                r#"{"type":"message_start","message":{"id":"msg_1","type":"message","role":"assistant","model":"claude-test-2026","content":[],"stop_reason":null,"usage":{"input_tokens":1,"output_tokens":1}}}"#,
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
                r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"{\"name\":"}}"#,
                r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"\"n\"}"}}"#,
                r#"{"type":"content_block_stop","index":0}"#,
                r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":2}}"#,
                r#"{"type":"message_stop"}"#,
            ];
            let body = events
                .iter()
                .map(|data| {
                    let event = serde_json::from_str::<serde_json::Value>(data).unwrap();
                    format!(
                        "event: {}\ndata: {data}\n\n",
                        event["type"].as_str().unwrap()
                    )
                })
                .collect::<String>();
            Mock::given(method("POST"))
                .and(path("/v1/messages"))
                .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
                .mount(&server)
                .await;
            unsafe { std::env::set_var("REFERENCE_TEST_ANTHROPIC_STREAM_KEY", "sk-test") };
            let config = serde_json::json!({
                "model": "claude-sonnet-4-5",
                "apiKeyEnv": "REFERENCE_TEST_ANTHROPIC_STREAM_KEY",
                "endpoint": format!("{}/v1", server.uri()),
            });
            let assistant = load(&config.to_string()).0.unwrap();
            let schema = serde_json::json!({ "type": "object" });
            let written = std::sync::Mutex::new(Vec::new());
            let progress = |piece: &str| written.lock().unwrap().push(piece.to_string());

            let completion = assistant
                .complete_streaming(
                    CompletionRequest::new("system", "prompt", "flow_draft", &schema),
                    &progress,
                )
                .await
                .unwrap();

            assert_eq!(completion.json, "{\"name\":\"n\"}");
            assert_eq!(*written.lock().unwrap(), ["{\"name\":", "\"n\"}"]);
        });
    }

    #[test]
    fn explains_why_it_cannot_answer() {
        let (broken, state) = load("{ \"modle\": \"x\" }");
        assert!(matches!(state, FileState::Broken(_)));
        let problem = broken.unwrap().describe().problem.unwrap();
        assert_eq!(problem.key, REFERENCE_ASSISTANT_BROKEN_KEY);

        let (keyless, _) =
            load(r#"{ "model": "gemini-2.5-flash", "apiKeyEnv": "REFERENCE_TEST_UNSET" }"#);
        let described = keyless.unwrap().describe();
        assert!(!described.ready);
        assert_eq!(
            described.problem.unwrap().params["variable"],
            "REFERENCE_TEST_UNSET"
        );
        assert_eq!(
            adapter_of("gemini-2.5-flash"),
            Some(genai::adapter::AdapterKind::Gemini)
        );
        assert_eq!(
            adapter_of("claude-sonnet-4-5"),
            Some(genai::adapter::AdapterKind::Anthropic)
        );

        let folder = tempfile::tempdir().unwrap();
        let (missing, state) = GenaiAssistant::load(&folder.path().join("assistant.json"));
        assert!(missing.is_none());
        assert_eq!(state, FileState::Missing);
    }
}
