use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_sdk::assistant::{AssistantProvider, CompletionProgress, CompletionRequest};
use yunheshiguang_sdk::documents::DocumentStore;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::governance::{AuditTicket, Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::scripts::list_scripts;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::drafts::{MAX_SCRIPT_CHARS, check_flow, check_script};
use super::openai::endpoint_problem;
use super::prompts::*;
use super::settings::{Endpoint, SettingsStore};

const MAX_PROMPT_CHARS: usize = 4_000;
const MAX_SETTING_CHARS: usize = 1_000;

pub struct AssistantService {
    pub(super) providers: Vec<Box<dyn AssistantProvider>>,
    pub(super) settings: Arc<SettingsStore>,
    pub(super) documents: Arc<dyn DocumentStore>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
}

fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        ASSISTANT_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn check_prompt(prompt: &str) -> Result<String, UiText> {
    let prompt = prompt.trim();

    if prompt.is_empty() {
        return Err(ui_text(ASSISTANT_PROMPT_REQUIRED_KEY));
    }
    if prompt.chars().count() > MAX_PROMPT_CHARS {
        return Err(ui_text_with_params(
            ASSISTANT_PROMPT_TOO_LONG_KEY,
            "",
            [("max", MAX_PROMPT_CHARS.to_string())],
        ));
    }

    Ok(prompt.to_string())
}

impl AssistantService {
    pub fn state(&self) -> AssistantState {
        AssistantState {
            providers: self
                .providers
                .iter()
                .map(|provider| provider.describe())
                .collect(),
            settings: self.settings.settings().into(),
            ..Default::default()
        }
    }

    fn chosen(&self) -> Result<(String, &dyn AssistantProvider), UiText> {
        let wanted = self.settings.settings().provider_id;
        let described = self
            .providers
            .iter()
            .map(|provider| (provider.describe(), provider.as_ref()));
        let mut last_problem = None;

        for (info, provider) in described {
            if !wanted.is_empty() && info.id != wanted {
                continue;
            }
            if info.ready {
                return Ok((info.id, provider));
            }
            last_problem = info.problem.into_option();
            if !wanted.is_empty() {
                break;
            }
        }

        Err(last_problem.unwrap_or_else(|| {
            ui_text(if wanted.is_empty() {
                ASSISTANT_NO_PROVIDER_KEY
            } else {
                ASSISTANT_PROVIDER_UNKNOWN_KEY
            })
        }))
    }

    fn check_settings(
        &self,
        settings: &AssistantSettings,
        api_key: Option<&str>,
    ) -> Option<UiText> {
        let known = settings.provider_id.is_empty()
            || self
                .providers
                .iter()
                .any(|provider| provider.describe().id == settings.provider_id);
        let endpoint = Endpoint {
            base_url: settings.base_url.trim().to_string(),
            model: "model".to_string(),
            api_key: Some(String::new()),
            structured_output: StructuredOutput::JsonSchema,
        };
        let too_long = [
            settings.base_url.as_str(),
            &settings.model,
            api_key.unwrap_or_default(),
        ]
        .iter()
        .any(|value| value.chars().count() > MAX_SETTING_CHARS);
        let unusable_key = api_key.is_some_and(|key| {
            key.trim()
                .chars()
                .any(|character| !character.is_ascii_graphic())
        });

        if !known {
            Some(ui_text(ASSISTANT_PROVIDER_UNKNOWN_KEY))
        } else if unusable_key {
            Some(ui_text(ASSISTANT_KEY_INVALID_KEY))
        } else if too_long {
            Some(ui_text_with_params(
                ASSISTANT_SETTING_TOO_LONG_KEY,
                "",
                [("max", MAX_SETTING_CHARS.to_string())],
            ))
        } else {
            endpoint_problem(&endpoint)
        }
    }

    fn refused(&self, detail: UiText) -> AssistantResult {
        AssistantResult {
            accepted: false,
            detail: detail.into(),
            state: self.state().into(),
            ..Default::default()
        }
    }

    pub async fn update(
        &self,
        request: UpdateAssistantRequest,
        principal: &Principal,
    ) -> AssistantResult {
        let settings = request.settings.into_option().unwrap_or_default();

        if let Some(detail) = self.check_settings(&settings, request.api_key.as_deref()) {
            return self.refused(detail);
        }

        let ticket = match self
            .governance
            .begin(GovernedAction::ManageAssistant, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return self.refused(reason),
        };
        let store = Arc::clone(&self.settings);
        let api_key = request.api_key;
        let saved = run_blocking(move || Ok(store.save(settings, api_key).map_err(storage_failed)))
            .await
            .unwrap_or_else(|error| Err(storage_failed(error)));

        match saved {
            Ok(()) => {
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(ui_text(ASSISTANT_AUDIT_UPDATED_KEY)),
                );
                AssistantResult {
                    accepted: true,
                    detail: Default::default(),
                    state: self.state().into(),
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                self.refused(detail)
            }
        }
    }

    async fn ask(
        &self,
        principal: &Principal,
        system: &str,
        prompt: &str,
        schema_name: &str,
        schema: &serde_json::Value,
        progress: Option<CompletionProgress<'_>>,
    ) -> Result<
        (
            String,
            AuditTicket,
            yunheshiguang_sdk::assistant::Completion,
        ),
        UiText,
    > {
        let (provider_id, provider) = self.chosen()?;
        let ticket = self
            .governance
            .begin(GovernedAction::UseAssistant, principal, &[])?;
        let request = CompletionRequest::new(system, prompt, schema_name, schema);
        let completion = match progress {
            Some(progress) => provider.complete_streaming(request, progress).await,
            None => provider.complete(request).await,
        };

        match completion {
            Ok(completion) => Ok((provider_id, ticket, completion)),
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                Err(detail)
            }
        }
    }

    fn record(&self, started: Instant, succeeded: bool) {
        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(GovernedAction::UseAssistant, succeeded, started.elapsed())
                .with_targets(1, u32::from(!succeeded)),
        ));
    }

    fn finish<T>(
        &self,
        ticket: AuditTicket,
        outcome: &Result<T, UiText>,
        audit: impl FnOnce(&T) -> UiText,
    ) {
        match outcome {
            Ok(draft) => ticket.finish(AuditOutcome::Succeeded, Some(audit(draft))),
            Err(detail) => ticket.finish(AuditOutcome::Failed, Some(detail.clone())),
        }
    }

    #[cfg(test)]
    pub async fn draft_flow(
        &self,
        request: DraftFlowRequest,
        principal: &Principal,
    ) -> DraftFlowResult {
        self.draft_flow_with(request, principal, None).await
    }

    pub async fn draft_flow_with(
        &self,
        request: DraftFlowRequest,
        principal: &Principal,
        progress: Option<CompletionProgress<'_>>,
    ) -> DraftFlowResult {
        let started = Instant::now();
        let outcome = self.flow_draft(request, principal, progress).await;
        self.record(started, outcome.is_ok());

        match outcome {
            Ok(draft) => DraftFlowResult {
                accepted: true,
                detail: Default::default(),
                draft: draft.into(),
                ..Default::default()
            },
            Err(detail) => DraftFlowResult {
                accepted: false,
                detail: detail.into(),
                draft: Default::default(),
                ..Default::default()
            },
        }
    }

    async fn flow_draft(
        &self,
        request: DraftFlowRequest,
        principal: &Principal,
        progress: Option<CompletionProgress<'_>>,
    ) -> Result<FlowDraft, UiText> {
        let prompt = check_prompt(&request.prompt)?;
        let documents = Arc::clone(&self.documents);
        let scripts = run_blocking(move || Ok(list_scripts(documents.as_ref())))
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or_default();
        let script_ids = scripts
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<HashSet<_>>();
        let schema = flow_schema();
        let system = flow_system_prompt(&scripts, &schema);
        let (provider_id, ticket, completion) = self
            .ask(
                principal,
                &system,
                &prompt,
                FLOW_SCHEMA_NAME,
                &schema,
                progress,
            )
            .await?;
        let outcome = check_flow(&completion.json, &script_ids).map(|(flow, issues)| FlowDraft {
            flow: flow.into(),
            issues,
            provider_id,
            model: completion.model,
            ..Default::default()
        });

        self.finish(ticket, &outcome, |draft| {
            ui_text_with_params(
                ASSISTANT_AUDIT_FLOW_KEY,
                "",
                [("model", draft.model.clone())],
            )
        });
        outcome
    }

    #[cfg(test)]
    pub async fn draft_script(
        &self,
        request: DraftScriptRequest,
        principal: &Principal,
    ) -> DraftScriptResult {
        self.draft_script_with(request, principal, None).await
    }

    pub async fn draft_script_with(
        &self,
        request: DraftScriptRequest,
        principal: &Principal,
        progress: Option<CompletionProgress<'_>>,
    ) -> DraftScriptResult {
        let started = Instant::now();
        let outcome = self.script_draft(request, principal, progress).await;
        self.record(started, outcome.is_ok());

        match outcome {
            Ok(draft) => DraftScriptResult {
                accepted: true,
                detail: Default::default(),
                draft: draft.into(),
                ..Default::default()
            },
            Err(detail) => DraftScriptResult {
                accepted: false,
                detail: detail.into(),
                draft: Default::default(),
                ..Default::default()
            },
        }
    }

    async fn script_draft(
        &self,
        request: DraftScriptRequest,
        principal: &Principal,
        progress: Option<CompletionProgress<'_>>,
    ) -> Result<ScriptDraft, UiText> {
        let prompt = check_prompt(&request.prompt)?;
        if request.current_body.chars().count() > MAX_SCRIPT_CHARS {
            return Err(ui_text_with_params(
                ASSISTANT_SCRIPT_TOO_LONG_KEY,
                "",
                [("max", MAX_SCRIPT_CHARS.to_string())],
            ));
        }

        let language = match ScriptLanguage::known(request.language) {
            Some(ScriptLanguage::Javascript) => ScriptLanguage::Javascript,
            _ => ScriptLanguage::Shell,
        };
        let schema = script_schema();
        let system = script_system_prompt(language, &schema);
        let user = script_prompt(&prompt, &request.current_body);
        let (provider_id, ticket, completion) = self
            .ask(
                principal,
                &system,
                &user,
                SCRIPT_SCHEMA_NAME,
                &schema,
                progress,
            )
            .await?;
        let outcome =
            check_script(&completion.json, language).map(|(script, explanation)| ScriptDraft {
                script: script.into(),
                explanation,
                provider_id,
                model: completion.model,
                ..Default::default()
            });

        self.finish(ticket, &outcome, |draft| {
            ui_text_with_params(
                ASSISTANT_AUDIT_SCRIPT_KEY,
                "",
                [("model", draft.model.clone())],
            )
        });
        outcome
    }
}
