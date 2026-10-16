use std::collections::HashSet;
use yunheshiguang_contract::Enumeration;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::workspace::batch::step_problem;
use crate::workspace::contract::{
    BatchActionKind, DraftIssue, Flow, FlowStep, Script, ScriptLanguage, UiText,
};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::prompts::{FLOW_ACTIONS, MAX_DRAFT_STEPS};

const MAX_NAME_CHARS: usize = 80;
pub const MAX_SCRIPT_CHARS: usize = 64 * 1024;

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawFlow {
    name: String,
    steps: Vec<RawStep>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawStep {
    kind: String,
    value: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawScript {
    name: String,
    body: String,
    explanation: String,
}

fn object_text(answer: &str) -> &str {
    match (answer.find('{'), answer.rfind('}')) {
        (Some(start), Some(end)) if start < end => &answer[start..=end],
        _ => answer,
    }
}

fn parse<T: DeserializeOwned>(answer: &str) -> Result<T, UiText> {
    serde_json::from_str(object_text(answer)).map_err(|error| {
        ui_text_with_params(
            ASSISTANT_UNREADABLE_KEY,
            "",
            [("message", error.to_string())],
        )
    })
}

fn name(raw: &str) -> String {
    raw.trim().chars().take(MAX_NAME_CHARS).collect()
}

fn about_the_draft(message: UiText) -> DraftIssue {
    DraftIssue {
        step_id: String::new(),
        message: message.into(),
        ..Default::default()
    }
}

fn step_value(kind: BatchActionKind, value: String) -> String {
    match kind {
        BatchActionKind::InputText | BatchActionKind::ClipboardWrite => value,
        _ => value.trim().to_string(),
    }
}

pub fn check_flow(
    answer: &str,
    scripts: &HashSet<String>,
) -> Result<(Flow, Vec<DraftIssue>), UiText> {
    let raw = parse::<RawFlow>(answer)?;
    let mut issues = Vec::new();
    let mut steps = Vec::new();

    if raw.steps.len() > MAX_DRAFT_STEPS {
        issues.push(about_the_draft(ui_text_with_params(
            ASSISTANT_DRAFT_TOO_LONG_KEY,
            "",
            [("max", MAX_DRAFT_STEPS.to_string())],
        )));
    }

    for (index, raw_step) in raw.steps.into_iter().take(MAX_DRAFT_STEPS).enumerate() {
        let Some(kind) = <BatchActionKind as Enumeration>::from_proto_name(raw_step.kind.trim())
            .filter(|kind| FLOW_ACTIONS.contains(kind))
        else {
            issues.push(about_the_draft(ui_text_with_params(
                ASSISTANT_DRAFT_UNKNOWN_ACTION_KEY,
                "",
                [("step", (index + 1).to_string())],
            )));
            continue;
        };
        let step = FlowStep {
            id: Uuid::now_v7().to_string(),
            kind: kind.into(),
            value: step_value(kind, raw_step.value),
            ..Default::default()
        };
        let problem = step_problem(kind, &step.value).or_else(|| {
            (kind == BatchActionKind::RunScript && !scripts.contains(&step.value))
                .then(|| ui_text(ASSISTANT_DRAFT_SCRIPT_MISSING_KEY))
        });

        if let Some(message) = problem {
            issues.push(DraftIssue {
                step_id: step.id.clone(),
                message: message.into(),
                ..Default::default()
            });
        }
        steps.push(step);
    }

    if steps.is_empty() {
        issues.push(about_the_draft(ui_text(ASSISTANT_DRAFT_EMPTY_KEY)));
    }

    Ok((
        Flow {
            name: name(&raw.name),
            steps,
            ..Default::default()
        },
        issues,
    ))
}

pub fn check_script(answer: &str, language: ScriptLanguage) -> Result<(Script, String), UiText> {
    let raw = parse::<RawScript>(answer)?;
    let body = raw.body.trim_end().to_string();

    if body.trim().is_empty() {
        return Err(ui_text(ASSISTANT_SCRIPT_EMPTY_KEY));
    }
    if body.chars().count() > MAX_SCRIPT_CHARS {
        return Err(ui_text_with_params(
            ASSISTANT_SCRIPT_TOO_LONG_KEY,
            "",
            [("max", MAX_SCRIPT_CHARS.to_string())],
        ));
    }

    Ok((
        Script {
            name: name(&raw.name),
            language: language.into(),
            body,
            ..Default::default()
        },
        raw.explanation.trim().to_string(),
    ))
}
