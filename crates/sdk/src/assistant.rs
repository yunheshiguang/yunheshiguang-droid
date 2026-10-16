use std::future::Future;
use std::pin::Pin;

pub use crate::contract::yunheshiguang::assistant::v1::AssistantProviderInfo;
use crate::contract::yunheshiguang::workspace::v1::UiText;

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub struct CompletionRequest<'a> {
    pub system: &'a str,
    pub prompt: &'a str,
    pub schema_name: &'a str,
    pub schema: &'a serde_json::Value,
}

impl<'a> CompletionRequest<'a> {
    pub fn new(
        system: &'a str,
        prompt: &'a str,
        schema_name: &'a str,
        schema: &'a serde_json::Value,
    ) -> Self {
        Self {
            system,
            prompt,
            schema_name,
            schema,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub json: String,
    pub model: String,
}

impl Completion {
    pub fn new(json: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            json: json.into(),
            model: model.into(),
        }
    }
}

pub type CompletionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Completion, UiText>> + Send + 'a>>;

pub type CompletionProgress<'a> = &'a (dyn Fn(&str) + Send + Sync);

pub trait AssistantProvider: Send + Sync {
    fn describe(&self) -> AssistantProviderInfo;

    fn complete<'a>(&'a self, request: CompletionRequest<'a>) -> CompletionFuture<'a>;

    fn complete_streaming<'a>(
        &'a self,
        request: CompletionRequest<'a>,
        progress: CompletionProgress<'a>,
    ) -> CompletionFuture<'a> {
        let _ = progress;
        self.complete(request)
    }
}
