use serde::Serialize;
use tauri::State;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "detail", rename_all = "camelCase")]
pub enum FileState {
    Missing,
    Loaded,
    Broken(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceStatus {
    pub policy_file: String,
    pub policy: FileState,
    pub policy_rules: u32,
    pub team_file: String,
    pub team: FileState,
    pub team_members: u32,
    pub license_file: String,
    pub license: FileState,
    pub audit_folder: String,
    pub sync_file: String,
    pub sync: FileState,
    pub sync_folder: String,
    pub assistant_file: String,
    pub assistant: FileState,
    pub redroid_file: String,
    pub redroid: FileState,
    pub vendors_file: String,
    pub vendors: FileState,
    pub metrics_file: String,
    pub api_keys_file: String,
    pub api_keys: FileState,
}

#[tauri::command]
pub(crate) async fn describe(
    status: State<'_, ReferenceStatus>,
) -> Result<ReferenceStatus, String> {
    Ok(status.inner().clone())
}
