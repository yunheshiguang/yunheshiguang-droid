mod api_keys;
mod assistant;
mod audit;
mod config;
mod input;
mod license;
mod market;
mod messages;
mod policy;
mod redroid;
mod scripts;
mod status;
mod team;
mod telemetry;
mod vendor;

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::ipc::CapabilityBuilder;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Runtime};
use yunheshiguang_sdk::documents::FolderDocumentStore;
use yunheshiguang_sdk::{
    ContributionConflict, Contributions, Extension, ExtensionError, WORKSPACE_WINDOWS,
};

pub use api_keys::{API_KEYS_AUTHENTICATOR_ID, EnterpriseApiKeys};
pub use assistant::{GENAI_PROVIDER_ID, GenaiAssistant};
pub use audit::TeamAuditStore;
pub use input::{ADB_KEYBOARD_CHANNEL_ID, AdbKeyboardChannel};
pub use license::{LicenseEntitlements, LicenseFile};
pub use market::{MARKET_CAPABILITY, MARKET_SOURCE_ID, MarketTemplates, NIGHTLY_PACKAGE_ID};
pub use policy::{Policy, PolicyAuthorization};
pub use redroid::{REDROID_PROVIDER_ID, RedroidProvider};
pub use scripts::{AUTOJS6_ENGINE_ID, AutoJs6Engine};
pub use status::{FileState, ReferenceStatus};
pub use team::TeamIdentity;
pub use telemetry::PrometheusTelemetry;
pub use vendor::BoardFarmVendor;

pub const EXTENSION_ID: &str = "reference";
pub const ENTERPRISE_CAPABILITY: &str = "reference.enterprise";
const PLUGIN_NAME: &str = "reference-extension";

#[derive(Clone, Debug)]
pub struct ReferencePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl ReferencePaths {
    pub fn resolve<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Self> {
        let folder = Path::new("extensions").join(EXTENSION_ID);

        Ok(Self {
            config_dir: app.path().app_config_dir()?.join(&folder),
            data_dir: app.path().app_data_dir()?.join(folder),
        })
    }

    pub fn policy_file(&self) -> PathBuf {
        self.config_dir.join("policy.cedar")
    }

    pub fn team_file(&self) -> PathBuf {
        self.config_dir.join("team.json")
    }

    pub fn license_file(&self) -> PathBuf {
        self.config_dir.join("license.json")
    }

    pub fn sync_file(&self) -> PathBuf {
        self.config_dir.join("sync.json")
    }

    pub fn assistant_file(&self) -> PathBuf {
        self.config_dir.join("assistant.json")
    }

    pub fn redroid_file(&self) -> PathBuf {
        self.config_dir.join("redroid.json")
    }

    pub fn local_documents(&self) -> PathBuf {
        self.data_dir.join("documents")
    }

    pub fn computer_id_file(&self) -> PathBuf {
        self.data_dir.join("computer-id")
    }

    pub fn vendors_file(&self) -> PathBuf {
        self.config_dir.join("vendors.json")
    }

    pub fn metrics_file(&self) -> PathBuf {
        self.data_dir.join("metrics.prom")
    }

    pub fn api_keys_file(&self) -> PathBuf {
        self.config_dir.join("api-keys.json")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncFile {
    folder: PathBuf,
}

fn documents_folder(paths: &ReferencePaths) -> (PathBuf, FileState) {
    match config::read_json::<SyncFile>(&paths.sync_file()) {
        Ok(Some(sync)) if sync.folder.is_absolute() => (sync.folder, FileState::Loaded),
        Ok(Some(sync)) => {
            let reason = format!(
                "the folder must be an absolute path, not {}",
                sync.folder.display()
            );
            log::error!("{} is unusable: {reason}", paths.sync_file().display());
            (paths.local_documents(), FileState::Broken(reason))
        }
        Ok(None) => (paths.local_documents(), FileState::Missing),
        Err(reason) => {
            log::error!("{} is unusable: {reason}", paths.sync_file().display());
            (paths.local_documents(), FileState::Broken(reason))
        }
    }
}

fn computer_id(paths: &ReferencePaths) -> String {
    let path = paths.computer_id_file();
    let valid = |id: &str| {
        !id.is_empty()
            && id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
    };

    if let Ok(existing) = std::fs::read_to_string(&path)
        && valid(existing.trim())
    {
        return existing.trim().to_string();
    }

    let generated = uuid::Uuid::now_v7().to_string();
    if let Err(error) =
        std::fs::create_dir_all(&paths.data_dir).and_then(|()| std::fs::write(&path, &generated))
    {
        log::warn!("{} is not writable: {error}", path.display());
    }
    generated
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn count(items: usize) -> u32 {
    u32::try_from(items).unwrap_or(u32::MAX)
}

pub fn contribute_with(
    paths: &ReferencePaths,
    account: &str,
    contributions: &mut Contributions,
) -> Result<ReferenceStatus, ContributionConflict> {
    let policy = Policy::load(&paths.policy_file());
    let (team, team_state) = TeamIdentity::load(&paths.team_file(), account);
    let entitlements = LicenseEntitlements::load(&paths.license_file());
    let (folder, sync) = documents_folder(paths);
    let audit = TeamAuditStore::open(folder.join("audit"), &computer_id(paths));
    let (assistant, assistant_state) = GenaiAssistant::load(&paths.assistant_file());
    let (redroid, redroid_state) = RedroidProvider::load(&paths.redroid_file());
    let board_farm = BoardFarmVendor::load(&paths.vendors_file());
    let api_keys = EnterpriseApiKeys::load(&paths.api_keys_file());
    let status = ReferenceStatus {
        policy_file: shown(&paths.policy_file()),
        policy: policy.file_state(),
        policy_rules: count(policy.rule_count()),
        team_file: shown(&paths.team_file()),
        team: team_state,
        team_members: count(team.as_ref().map_or(0, TeamIdentity::member_count)),
        license_file: shown(&paths.license_file()),
        license: entitlements.state().clone(),
        audit_folder: shown(audit.folder()),
        sync_file: shown(&paths.sync_file()),
        sync,
        sync_folder: shown(&folder),
        assistant_file: shown(&paths.assistant_file()),
        assistant: assistant_state,
        redroid_file: shown(&paths.redroid_file()),
        redroid: redroid_state,
        vendors_file: shown(&paths.vendors_file()),
        vendors: board_farm.state().clone(),
        metrics_file: shown(&paths.metrics_file()),
        api_keys_file: shown(&paths.api_keys_file()),
        api_keys: api_keys.state().clone(),
    };

    contributions.set_authorization(PolicyAuthorization::new(policy))?;
    if let Some(team) = team {
        contributions.set_identity(team)?;
    }
    contributions.set_entitlements(entitlements)?;
    contributions.set_audit_store(audit)?;
    contributions.set_documents(FolderDocumentStore::new(folder))?;
    contributions.add_template_source(MarketTemplates);
    if let Some(assistant) = assistant {
        contributions.add_assistant_provider(assistant);
    }
    if let Some(redroid) = redroid {
        contributions.add_cloud_provider(redroid);
    }
    contributions.add_telemetry_sink(PrometheusTelemetry::new(paths.metrics_file()));
    contributions.add_vendor_adapter(board_farm);
    contributions.add_input_channel(AdbKeyboardChannel);
    contributions.add_script_engine(AutoJs6Engine);
    contributions.add_api_authenticator(api_keys);
    Ok(status)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceExtension;

impl<R: Runtime> Extension<R> for ReferenceExtension {
    fn id(&self) -> &'static str {
        EXTENSION_ID
    }

    fn plugins(&self) -> Vec<TauriPlugin<R>> {
        vec![plugin()]
    }

    fn contribute(
        &self,
        app: &AppHandle<R>,
        contributions: &mut Contributions,
    ) -> Result<(), ExtensionError> {
        let account = whoami::username().unwrap_or_default();
        let status = contribute_with(&ReferencePaths::resolve(app)?, &account, contributions)?;
        app.manage(status);

        app.add_capability(
            CapabilityBuilder::new(PLUGIN_NAME)
                .windows(WORKSPACE_WINDOWS.iter().copied())
                .permission(format!("{PLUGIN_NAME}:default")),
        )?;
        Ok(())
    }
}

fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![status::describe])
        .build()
}

#[cfg(test)]
mod tests;
