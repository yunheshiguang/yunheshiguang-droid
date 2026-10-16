use std::error::Error;
use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::ContributionParts;

use super::api::ApiModule;
use super::assistant::AssistantModule;
use super::audit::AuditModule;
use super::automation::AutomationModule;
use super::batch::BatchModule;
use super::cloud::CloudModule;
use super::connection::ConnectionModule;
use super::documents::DocumentsModule;
use super::events::EventsModule;
use super::governance::{Governance, GovernanceModule, GovernanceParts};
use super::input::InputModule;
use super::packages::PackagesModule;
use super::profiles::ProfilesModule;
use super::scripts::ScriptsModule;
use super::snapshot::SnapshotModule;
use super::telemetry::{Telemetry, TelemetryModule};
use super::templates::TemplatesModule;
use super::transports::TransportsModule;
use super::updates::UpdatesModule;
use super::vendors::VendorsModule;

pub type ModuleSetupError = Box<dyn Error>;

pub trait WorkspaceModule<R: Runtime>: Send + Sync {
    fn id(&self) -> &'static str;

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError>;
}

pub fn dependency<T, R>(app: &App<R>) -> Result<Arc<T>, ModuleSetupError>
where
    T: ?Sized + Send + Sync + 'static,
    R: Runtime,
{
    app.try_state::<Arc<T>>()
        .map(|state| Arc::clone(state.inner()))
        .ok_or_else(|| format!("{} has to be set up first", std::any::type_name::<T>()).into())
}

pub fn core_modules<R: Runtime>(parts: ContributionParts) -> Vec<Box<dyn WorkspaceModule<R>>> {
    let governance = Governance::assemble(GovernanceParts {
        authorization: parts.authorization,
        entitlements: parts.entitlements,
        identity: parts.identity,
        audit_sinks: parts.audit_sinks,
        audit_store: parts.audit_store,
    });

    vec![
        Box::new(crate::window_lifecycle::WindowLifecycleModule),
        Box::new(super::lifecycle::ShutdownModule),
        Box::new(super::rpc::RpcModule),
        Box::new(EventsModule),
        Box::new(GovernanceModule::new(governance)),
        Box::new(AuditModule),
        Box::new(TelemetryModule::new(Telemetry::assemble(
            parts.telemetry_sinks,
        ))),
        Box::new(super::adb::AdbModule),
        Box::new(super::adb::DeviceWatchModule),
        Box::new(super::android_tools::AndroidToolsModule),
        Box::new(DocumentsModule::new(parts.documents)),
        Box::new(CloudModule::new(parts.cloud_providers)),
        Box::new(ConnectionModule::new(parts.connection)),
        Box::new(TransportsModule::new(parts.transports)),
        Box::new(TemplatesModule::new(parts.template_sources)),
        Box::new(VendorsModule::new(parts.vendor_adapters)),
        Box::new(InputModule::new(parts.input_channels)),
        Box::new(ScriptsModule::new(parts.script_engines)),
        Box::new(AssistantModule::new(parts.assistant_providers)),
        Box::new(PackagesModule),
        Box::new(ProfilesModule),
        Box::new(super::scrcpy::ScrcpyModule),
        Box::new(SnapshotModule),
        Box::new(BatchModule),
        Box::new(AutomationModule),
        Box::new(UpdatesModule),
        Box::new(ApiModule::new(parts.api_authenticators)),
    ]
}

pub fn setup_modules<R: Runtime>(
    app: &mut App<R>,
    modules: &[Box<dyn WorkspaceModule<R>>],
) -> Result<(), ModuleSetupError> {
    for module in modules {
        module
            .setup(app)
            .map_err(|error| -> ModuleSetupError { format!("{}: {error}", module.id()).into() })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tauri::Wry;
    use yunheshiguang_sdk::Contributions;

    use super::core_modules;

    #[test]
    fn sets_up_governance_first_and_connection_after_its_dependencies() {
        let modules = core_modules::<Wry>(Contributions::default().into_parts());
        let order: Vec<_> = modules.iter().map(|module| module.id()).collect();
        let position = |id: &str| order.iter().position(|candidate| *candidate == id).unwrap();

        assert!(position("governance") < position("adb"));
        assert!(position("audit") == position("governance") + 1);
        assert!(position("connection") > position("adb"));
        assert!(position("connection") > position("android-platform-tools"));
        assert!(position("transports") > position("adb"));
        assert!(position("transports") > position("android-platform-tools"));
        assert!(position("vendors") > position("adb"));
        assert!(position("profiles") > position("documents"));
        assert!(position("profiles") > position("adb"));
        assert!(position("events") < position("profiles"));
        assert!(position("events") < position("device-watch"));
        assert!(position("snapshot") > position("profiles"));
        assert!(position("snapshot") > position("transports"));
        assert!(position("batch") > position("scripts"));
        assert!(position("assistant") > position("documents"));
        assert!(position("assistant") > position("telemetry"));
        assert!(position("packages") > position("templates"));
        assert!(position("cloud") > position("documents"));
        assert!(position("cloud") > position("telemetry"));
        assert!(position("transports") > position("cloud"));
        assert!(position("snapshot") > position("cloud"));
        assert!(position("packages") > position("events"));
        assert!(position("batch") > position("input"));
        assert!(position("batch") > position("vendors"));
        assert!(position("automation") > position("batch"));
        assert!(position("automation") > position("snapshot"));
        assert!(position("api") > position("automation"));
    }
}
