use std::collections::{BTreeSet, HashMap, HashSet};

use yunheshiguang_sdk::documents::{Collection, Document, DocumentId, DocumentStore};

use crate::workspace::automation::{FLOWS_COLLECTION, list_flows};
use crate::workspace::contract::*;
use crate::workspace::messages::*;
use crate::workspace::scripts::{SCRIPTS_COLLECTION, list_scripts};
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::format::storage_failed;

#[derive(Default)]
pub struct Library {
    pub flows: HashMap<String, Flow>,
    pub scripts: HashMap<String, Script>,
    pub unreadable: HashSet<(&'static str, String)>,
}

impl Library {
    fn holds(&self, collection: &'static str, id: &str) -> bool {
        let readable = match collection {
            FLOWS_COLLECTION => self.flows.contains_key(id),
            _ => self.scripts.contains_key(id),
        };
        readable || self.unreadable.contains(&(collection, id.to_string()))
    }

    fn taken(&self, collection: &'static str) -> HashSet<String> {
        let readable = match collection {
            FLOWS_COLLECTION => self.flows.keys().cloned().collect::<Vec<_>>(),
            _ => self.scripts.keys().cloned().collect(),
        };
        readable
            .into_iter()
            .chain(
                self.unreadable
                    .iter()
                    .filter(|(kind, _)| *kind == collection)
                    .map(|(_, id)| id.clone()),
            )
            .collect()
    }
}

fn stored_ids(store: &dyn DocumentStore, collection: &str) -> Result<Vec<String>, UiText> {
    let collection = Collection::parse(collection).expect("collection names are valid");

    Ok(store
        .list(&collection)
        .map_err(storage_failed)?
        .into_iter()
        .map(|document| document.id.as_str().to_string())
        .collect())
}

pub fn read_library(store: &dyn DocumentStore) -> Result<Library, UiText> {
    let mut library = Library {
        flows: list_flows(store)?
            .flows
            .into_iter()
            .map(|entry| (entry.id, entry.flow.into_option().unwrap_or_default()))
            .collect(),
        scripts: list_scripts(store)?.into_iter().collect(),
        unreadable: HashSet::new(),
    };
    for collection in [FLOWS_COLLECTION, SCRIPTS_COLLECTION] {
        for id in stored_ids(store, collection)? {
            if !library.holds(collection, &id) {
                library.unreadable.insert((collection, id));
            }
        }
    }

    Ok(library)
}

fn scripts_run_by(flow: &Flow) -> impl Iterator<Item = &str> {
    flow.steps
        .iter()
        .filter(|step| step.kind == BatchActionKind::RunScript)
        .map(|step| step.value.as_str())
        .filter(|id| !id.is_empty())
}

pub fn collect(
    library: &Library,
    flow_ids: &[String],
) -> Result<(Vec<PackageFlow>, Vec<PackageScript>), UiText> {
    let mut flows = Vec::new();
    let mut script_ids = BTreeSet::new();

    for id in flow_ids.iter().collect::<BTreeSet<_>>() {
        let flow = library.flows.get(id).ok_or_else(|| {
            ui_text_with_params(PACKAGES_FLOW_MISSING_KEY, "", [("id", id.clone())])
        })?;

        for script in scripts_run_by(flow) {
            if !library.scripts.contains_key(script) {
                return Err(ui_text_with_params(
                    PACKAGES_SCRIPT_MISSING_KEY,
                    "",
                    [("flow", flow.name.clone()), ("script", script.to_string())],
                ));
            }
            script_ids.insert(script.to_string());
        }
        flows.push(PackageFlow {
            id: id.clone(),
            flow: flow.clone().into(),
            ..Default::default()
        });
    }

    let scripts = script_ids
        .into_iter()
        .map(|id| PackageScript {
            script: library.scripts.get(&id).cloned().into(),
            id,
            ..Default::default()
        })
        .collect();

    Ok((flows, scripts))
}

pub fn preview(package: &TemplatePackage, library: &Library, verified: bool) -> PackagePreview {
    let included = package
        .scripts
        .iter()
        .map(|item| item.id.as_str())
        .collect::<HashSet<_>>();
    let mut problems = Vec::new();

    if !verified {
        problems.push(ui_text(PACKAGES_UNVERIFIED_KEY));
    }
    for item in &package.flows {
        let flow = item.flow.clone().into_option().unwrap_or_default();
        for script in scripts_run_by(&flow) {
            if !included.contains(script) && !library.scripts.contains_key(script) {
                problems.push(ui_text_with_params(
                    PACKAGES_SCRIPT_NOT_INCLUDED_KEY,
                    "",
                    [("flow", flow.name.clone()), ("script", script.to_string())],
                ));
            }
        }
    }
    for item in &package.scripts {
        if item
            .script
            .as_option()
            .is_none_or(|script| script.body.trim().is_empty())
        {
            problems.push(ui_text_with_params(
                PACKAGES_SCRIPT_EMPTY_KEY,
                "",
                [("id", item.id.clone())],
            ));
        }
    }

    PackagePreview {
        signer: Default::default(),
        manifest: package.manifest.clone(),
        flows: package
            .flows
            .iter()
            .map(|item| PackageItem {
                id: item.id.clone(),
                name: item
                    .flow
                    .as_option()
                    .map(|flow| flow.name.clone())
                    .unwrap_or_default(),
                exists: library.holds(FLOWS_COLLECTION, &item.id),
                ..Default::default()
            })
            .collect(),
        scripts: package
            .scripts
            .iter()
            .map(|item| PackageItem {
                id: item.id.clone(),
                name: item
                    .script
                    .as_option()
                    .map(|script| script.name.clone())
                    .unwrap_or_default(),
                exists: library.holds(SCRIPTS_COLLECTION, &item.id),
                ..Default::default()
            })
            .collect(),
        problems,
        verified,
        ..Default::default()
    }
}

fn unique(id: &str, taken: &HashSet<String>) -> String {
    (2..)
        .map(|number| format!("{id}-{number}"))
        .find(|candidate| !taken.contains(candidate))
        .expect("an unused suffix exists")
}

fn place(id: &str, taken: &mut HashSet<String>, policy: ConflictPolicy) -> Option<String> {
    let placed = if !taken.contains(id) {
        Some(id.to_string())
    } else {
        match policy {
            ConflictPolicy::Replace => Some(id.to_string()),
            ConflictPolicy::Skip => None,
            ConflictPolicy::KeepBoth | ConflictPolicy::Unspecified => Some(unique(id, taken)),
        }
    };
    if let Some(placed) = &placed {
        taken.insert(placed.clone());
    }

    placed
}

pub struct Plan {
    pub flows: Vec<(String, Flow)>,
    pub scripts: Vec<(String, Script)>,
}

pub fn plan(package: &TemplatePackage, library: &Library, policy: ConflictPolicy) -> Plan {
    let mut taken_scripts = library.taken(SCRIPTS_COLLECTION);
    let mut taken_flows = library.taken(FLOWS_COLLECTION);
    let mut renamed = HashMap::new();
    let mut scripts = Vec::new();

    for item in &package.scripts {
        if let Some(id) = place(&item.id, &mut taken_scripts, policy) {
            if id != item.id {
                renamed.insert(item.id.clone(), id.clone());
            }
            scripts.push((id, item.script.clone().into_option().unwrap_or_default()));
        }
    }

    let flows = package
        .flows
        .iter()
        .filter_map(|item| {
            let id = place(&item.id, &mut taken_flows, policy)?;
            let mut flow = item.flow.clone().into_option().unwrap_or_default();
            for step in &mut flow.steps {
                if step.kind == BatchActionKind::RunScript
                    && let Some(renamed) = renamed.get(&step.value)
                {
                    step.value.clone_from(renamed);
                }
            }
            Some((id, flow))
        })
        .collect();

    Plan { flows, scripts }
}

fn document<T: serde::Serialize>(
    collection: &str,
    id: &str,
    value: &T,
) -> Result<(Collection, Document), UiText> {
    let collection = Collection::parse(collection).expect("collection names are valid");
    let body = serde_json::to_string(value).map_err(storage_failed)?;
    let document = DocumentId::parse(id)
        .and_then(|id| Document::new(id, body))
        .map_err(storage_failed)?;

    Ok((collection, document))
}

pub fn install(store: &dyn DocumentStore, plan: &Plan) -> Result<(), UiText> {
    let documents = plan
        .scripts
        .iter()
        .map(|(id, script)| document(SCRIPTS_COLLECTION, id, script))
        .chain(
            plan.flows
                .iter()
                .map(|(id, flow)| document(FLOWS_COLLECTION, id, flow)),
        )
        .collect::<Result<Vec<_>, _>>()?;

    for (collection, document) in &documents {
        store.save(collection, document).map_err(storage_failed)?;
    }

    Ok(())
}
