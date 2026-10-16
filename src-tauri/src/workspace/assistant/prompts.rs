use serde_json::{Value, json};
use yunheshiguang_contract::Enumeration;

use crate::workspace::contract::{BatchActionKind, Script, ScriptLanguage};

pub const FLOW_SCHEMA_NAME: &str = "flow_draft";
pub const SCRIPT_SCHEMA_NAME: &str = "script_draft";
pub const MAX_DRAFT_STEPS: usize = 30;

pub const FLOW_ACTIONS: [BatchActionKind; 12] = [
    BatchActionKind::LaunchApp,
    BatchActionKind::StopApp,
    BatchActionKind::ClearAppData,
    BatchActionKind::UninstallApp,
    BatchActionKind::InputText,
    BatchActionKind::ClipboardWrite,
    BatchActionKind::RunScript,
    BatchActionKind::Screenshot,
    BatchActionKind::ScreenRecord,
    BatchActionKind::InstallApk,
    BatchActionKind::UploadFile,
    BatchActionKind::RecoverConnection,
];

fn explain(kind: BatchActionKind) -> &'static str {
    match kind {
        BatchActionKind::LaunchApp => {
            "open an app; value: its Android package name, such as com.android.settings"
        }
        BatchActionKind::StopApp => "force-stop an app; value: its Android package name",
        BatchActionKind::ClearAppData => {
            "erase an app's data and cache, as if it were just installed; value: its Android \
             package name"
        }
        BatchActionKind::UninstallApp => {
            "uninstall an app and its data; value: its Android package name"
        }
        BatchActionKind::InputText => "type text into the field that has focus; value: the text",
        BatchActionKind::ClipboardWrite => "put text on the clipboard; value: the text",
        BatchActionKind::RunScript => {
            "run one of the saved scripts listed below; value: the script id"
        }
        BatchActionKind::Screenshot => {
            "take a screenshot; value: a folder on this computer to save it in, or \"\""
        }
        BatchActionKind::ScreenRecord => {
            "record the screen for a while; value: a folder on this computer to save it in, or \"\""
        }
        BatchActionKind::InstallApk => {
            "install an app; value: the path of an .apk file on this computer, or \"\""
        }
        BatchActionKind::UploadFile => {
            "copy a file to the devices; value: the path of the file on this computer, or \"\""
        }
        BatchActionKind::RecoverConnection => "reconnect devices that dropped off; value: \"\"",
        BatchActionKind::Unspecified => "",
    }
}

fn reply_with(schema: &Value) -> String {
    format!(
        "Reply with a single JSON object, and nothing else, that matches this JSON Schema:\n{}",
        serde_json::to_string_pretty(schema).unwrap_or_default()
    )
}

pub fn flow_schema() -> Value {
    let kinds = FLOW_ACTIONS
        .iter()
        .map(|kind| kind.proto_name())
        .collect::<Vec<_>>();

    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "A short name for the flow" },
            "steps": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "enum": kinds },
                        "value": { "type": "string" }
                    },
                    "required": ["kind", "value"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["name", "steps"],
        "additionalProperties": false
    })
}

pub fn flow_system_prompt(scripts: &[(String, Script)], schema: &Value) -> String {
    let actions = FLOW_ACTIONS
        .iter()
        .map(|kind| format!("- {}: {}", kind.proto_name(), explain(*kind)))
        .collect::<Vec<_>>()
        .join("\n");
    let scripts = if scripts.is_empty() {
        "There are no saved scripts, so do not use BATCH_ACTION_KIND_RUN_SCRIPT.".to_string()
    } else {
        let listed = scripts
            .iter()
            .map(|(id, script)| format!("- {id}: {}", script.name))
            .collect::<Vec<_>>()
            .join("\n");
        format!("Saved scripts (id: name):\n{listed}")
    };

    format!(
        "You draft automation flows for an Android device farm. A flow runs its steps in \
         order, on every selected device at the same time. Each step is one action with one \
         string value.\n\nActions:\n{actions}\n\n{scripts}\n\nRules:\n\
         - Use only the actions above, at most {MAX_DRAFT_STEPS} steps.\n\
         - There is no wait, tap, or swipe action. When the request needs one, use a saved \
         script that does it, or leave that part out.\n\
         - Never invent paths or script ids; leave the value empty when you do not know it.\n\
         - Write the flow name in the language of the request.\n\n{}",
        reply_with(schema)
    )
}

pub fn script_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "A short name for the script" },
            "body": { "type": "string", "description": "The complete script" },
            "explanation": { "type": "string", "description": "What the script does" }
        },
        "required": ["name", "body", "explanation"],
        "additionalProperties": false
    })
}

pub fn script_system_prompt(language: ScriptLanguage, schema: &Value) -> String {
    let environment = match language {
        ScriptLanguage::Javascript => {
            "You write JavaScript for AutoJs6 on Android devices. The script is saved on the \
             device and started in AutoJs6, which provides globals such as app.launch, click, \
             swipe, text, id, desc, sleep, toast, and log. It runs in the background; nothing \
             it prints comes back to the operator."
        }
        _ => {
            "You write shell scripts for Android devices. The script is pushed to the device \
             and run by sh through adb shell, as the shell user, without root. Toybox provides \
             the common commands, and input, am, pm, settings, dumpsys, getprop, screencap, \
             and svc are available. The operator sees what the script prints and its exit \
             code; exit with a non-zero code when it fails."
        }
    };

    format!(
        "{environment}\n\nRules:\n\
         - Only use what a stock Android device provides. Do not assume root, busybox, or \
         Python.\n\
         - Keep the script short, with brief comments.\n\
         - Write the name, the comments, and the explanation in the language of the \
         request.\n\n{}",
        reply_with(schema)
    )
}

pub fn script_prompt(prompt: &str, current_body: &str) -> String {
    if current_body.trim().is_empty() {
        prompt.to_string()
    } else {
        format!("{prompt}\n\nThe script currently reads:\n```\n{current_body}\n```")
    }
}
