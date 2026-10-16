use yunheshiguang_sdk::documents::{Collection, Document, DocumentStore};

use crate::workspace::contract::{Script, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

pub const SCRIPTS_COLLECTION: &str = "scripts";

fn documents(store: &dyn DocumentStore) -> Result<Vec<Document>, UiText> {
    let collection = Collection::parse(SCRIPTS_COLLECTION).expect("collection name is valid");

    store.list(&collection).map_err(|error| {
        ui_text_with_params(SCRIPT_LOAD_FAILED_KEY, "", [("message", error.to_string())])
    })
}

pub fn load_script(store: &dyn DocumentStore, id: &str) -> Result<Script, UiText> {
    let documents = documents(store)?;
    let document = documents
        .iter()
        .find(|document| document.id.as_str() == id)
        .ok_or_else(|| ui_text(SCRIPT_MISSING_KEY))?;
    let script = yunheshiguang_contract::json::from_str::<Script>(&document.body)
        .map_err(|_| ui_text(SCRIPT_UNREADABLE_KEY))?;

    if script.body.trim().is_empty() {
        return Err(ui_text(SCRIPT_EMPTY_KEY));
    }

    Ok(script)
}

pub fn list_scripts(store: &dyn DocumentStore) -> Result<Vec<(String, Script)>, UiText> {
    let mut scripts = documents(store)?
        .into_iter()
        .filter_map(|document| {
            yunheshiguang_contract::json::from_str::<Script>(&document.body)
                .ok()
                .map(|script| (document.id.as_str().to_string(), script))
        })
        .collect::<Vec<_>>();
    scripts.sort_by(|left, right| left.0.cmp(&right.0));

    Ok(scripts)
}
