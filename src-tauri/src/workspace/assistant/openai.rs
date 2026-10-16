use std::sync::Arc;
use std::time::Duration;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs, ResponseFormat,
    ResponseFormatJsonSchema,
};
use futures_util::StreamExt;
use yunheshiguang_sdk::assistant::{
    AssistantProvider, AssistantProviderInfo, Completion, CompletionFuture, CompletionProgress,
    CompletionRequest,
};

use crate::workspace::contract::{StructuredOutput, UiText};
use crate::workspace::loopback::{is_loopback_host, is_loopback_url};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::settings::{Endpoint, SettingsStore};

pub const OPENAI_COMPATIBLE_ID: &str = "core.openai-compatible";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

pub fn endpoint_problem(endpoint: &Endpoint) -> Option<UiText> {
    let Some(url) = reqwest::Url::parse(&endpoint.base_url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"))
    else {
        return Some(ui_text(ASSISTANT_BASE_URL_INVALID_KEY));
    };

    if endpoint.model.is_empty() {
        return Some(ui_text(ASSISTANT_MODEL_MISSING_KEY));
    }
    if endpoint.api_key.is_none() && !url.host_str().is_some_and(is_loopback_host) {
        return Some(ui_text(ASSISTANT_KEY_MISSING_KEY));
    }

    None
}

pub fn request_failed(message: impl ToString) -> UiText {
    ui_text_with_params(
        ASSISTANT_REQUEST_FAILED_KEY,
        "",
        [("message", message.to_string())],
    )
}

fn failure(error: OpenAIError) -> UiText {
    match error {
        OpenAIError::ApiError(response) if response.api_error.message.trim().is_empty() => {
            request_failed(format!("HTTP {}", response.status_code))
        }
        OpenAIError::ApiError(response) => request_failed(response.api_error.message),
        other => request_failed(other),
    }
}

pub struct OpenAiCompatible {
    settings: Arc<SettingsStore>,
}

impl OpenAiCompatible {
    pub fn new(settings: Arc<SettingsStore>) -> Self {
        Self { settings }
    }
}

fn client(endpoint: &Endpoint) -> Result<Client<OpenAIConfig>, UiText> {
    let mut http = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(IDLE_TIMEOUT);
    if is_loopback_url(&endpoint.base_url) {
        http = http.no_proxy();
    }
    let http = http.build().map_err(request_failed)?;
    let config = OpenAIConfig::new()
        .with_api_base(&endpoint.base_url)
        .with_api_key(endpoint.api_key.clone().unwrap_or_default());

    Ok(Client::build(http, config))
}

fn chat_request(
    endpoint: &Endpoint,
    request: CompletionRequest<'_>,
) -> Result<CreateChatCompletionRequest, UiText> {
    let format = match endpoint.structured_output {
        StructuredOutput::JsonObject => ResponseFormat::JsonObject,
        _ => ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: None,
                name: request.schema_name.to_string(),
                schema: request.schema.clone(),
                strict: Some(true),
            },
        },
    };
    CreateChatCompletionRequestArgs::default()
        .model(&endpoint.model)
        .messages([
            ChatCompletionRequestSystemMessage::from(request.system).into(),
            ChatCompletionRequestUserMessage::from(request.prompt).into(),
        ])
        .response_format(format)
        .build()
        .map_err(failure)
}

fn refused(refusal: String) -> UiText {
    ui_text_with_params(ASSISTANT_REFUSED_KEY, "", [("message", refusal)])
}

async fn complete_streaming(
    endpoint: Endpoint,
    request: CompletionRequest<'_>,
    progress: CompletionProgress<'_>,
) -> Result<Completion, UiText> {
    let chat = chat_request(&endpoint, request)?;
    let mut stream = client(&endpoint)?
        .chat()
        .create_stream(chat)
        .await
        .map_err(failure)?;
    let (mut json, mut refusal, mut model) = (String::new(), String::new(), String::new());

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(failure)?;
        if model.is_empty() {
            model = chunk.model;
        }
        for choice in chunk.choices.into_iter().filter(|choice| choice.index == 0) {
            if let Some(piece) = choice.delta.content.filter(|piece| !piece.is_empty()) {
                progress(&piece);
                json.push_str(&piece);
            }
            refusal.extend(choice.delta.refusal);
        }
    }

    if !refusal.trim().is_empty() {
        return Err(refused(refusal));
    }
    if json.trim().is_empty() {
        return Err(ui_text(ASSISTANT_EMPTY_ANSWER_KEY));
    }
    Ok(Completion::new(json, model))
}

async fn complete(
    endpoint: Endpoint,
    request: CompletionRequest<'_>,
) -> Result<Completion, UiText> {
    let chat = chat_request(&endpoint, request)?;
    let response = client(&endpoint)?
        .chat()
        .create(chat)
        .await
        .map_err(failure)?;
    let message = response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .ok_or_else(|| ui_text(ASSISTANT_EMPTY_ANSWER_KEY))?;

    if let Some(refusal) = message.refusal.filter(|refusal| !refusal.trim().is_empty()) {
        return Err(refused(refusal));
    }

    let json = message
        .content
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| ui_text(ASSISTANT_EMPTY_ANSWER_KEY))?;

    Ok(Completion::new(json, response.model))
}

impl AssistantProvider for OpenAiCompatible {
    fn describe(&self) -> AssistantProviderInfo {
        let problem = endpoint_problem(&self.settings.endpoint());

        AssistantProviderInfo {
            id: OPENAI_COMPATIBLE_ID.to_string(),
            label: ui_text(ASSISTANT_OPENAI_LABEL_KEY).into(),
            description: ui_text(ASSISTANT_OPENAI_DESCRIPTION_KEY).into(),
            configurable: true,
            ready: problem.is_none(),
            problem: problem.into(),
            ..Default::default()
        }
    }

    fn complete<'a>(&'a self, request: CompletionRequest<'a>) -> CompletionFuture<'a> {
        let endpoint = self.settings.endpoint();

        Box::pin(async move {
            if let Some(problem) = endpoint_problem(&endpoint) {
                return Err(problem);
            }

            complete(endpoint, request).await
        })
    }

    fn complete_streaming<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        progress: CompletionProgress<'a>,
    ) -> CompletionFuture<'a> {
        let endpoint = self.settings.endpoint();

        Box::pin(async move {
            if let Some(problem) = endpoint_problem(&endpoint) {
                return Err(problem);
            }

            complete_streaming(endpoint, request, progress).await
        })
    }
}
