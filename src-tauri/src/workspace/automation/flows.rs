use serde_json::Value;
use yunheshiguang_sdk::documents::{Collection, DocumentStore};

use crate::workspace::batch::uses_host_files;
use crate::workspace::contract::{Flow, FlowEntry, Flows, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

pub const FLOWS_COLLECTION: &str = "automation-flows";
const KIND_PREFIX: &str = "BATCH_ACTION_KIND_";

pub fn parse_flow(body: &str) -> Option<Flow> {
    let mut value = serde_json::from_str::<Value>(body).ok()?;

    if let Some(steps) = value.get_mut("steps").and_then(Value::as_array_mut) {
        for kind in steps.iter_mut().filter_map(|step| step.get_mut("kind")) {
            if let Some(name) = kind.as_str().filter(|name| !name.starts_with(KIND_PREFIX)) {
                *kind = Value::String(format!("{KIND_PREFIX}{name}"));
            }
        }
    }

    yunheshiguang_contract::json::from_value(value).ok()
}

pub fn host_files_refusal(body: &str) -> Option<UiText> {
    parse_flow(body)
        .is_some_and(|flow| {
            flow.steps
                .iter()
                .any(|step| uses_host_files(step.kind.to_i32()))
        })
        .then(|| ui_text(API_HOST_FILES_KEY))
}

fn read_all(store: &dyn DocumentStore) -> Result<Vec<(String, Option<Flow>)>, UiText> {
    let collection = Collection::parse(FLOWS_COLLECTION).expect("collection name is valid");
    let documents = store.list(&collection).map_err(|error| {
        ui_text_with_params(
            AUTOMATION_LOAD_FAILED_KEY,
            "",
            [("message", error.to_string())],
        )
    })?;

    Ok(documents
        .into_iter()
        .map(|document| (document.id.as_str().to_string(), parse_flow(&document.body)))
        .collect())
}

pub fn list_flows(store: &dyn DocumentStore) -> Result<Flows, UiText> {
    Ok(Flows {
        flows: read_all(store)?
            .into_iter()
            .filter_map(|(id, flow)| {
                flow.map(|flow| FlowEntry {
                    id,
                    flow: flow.into(),
                    ..Default::default()
                })
            })
            .collect(),
        ..Default::default()
    })
}

pub fn load_flow(store: &dyn DocumentStore, id: &str) -> Result<Flow, UiText> {
    let flow = read_all(store)?
        .into_iter()
        .find(|(candidate, _)| candidate == id)
        .ok_or_else(|| ui_text(AUTOMATION_FLOW_MISSING_KEY))?
        .1
        .ok_or_else(|| ui_text(AUTOMATION_FLOW_UNREADABLE_KEY))?;

    if flow.steps.is_empty() {
        return Err(ui_text(AUTOMATION_FLOW_EMPTY_KEY));
    }

    Ok(flow)
}
