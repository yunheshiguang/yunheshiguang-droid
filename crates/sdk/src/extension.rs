use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};

use crate::api::ApiAuthenticator;
use crate::assistant::AssistantProvider;
use crate::cloud::CloudPhoneProvider;
use crate::connection::ConnectionProvider;
use crate::documents::DocumentStore;
use crate::governance::{
    AuditSink, AuditStore, AuthorizationProvider, EntitlementProvider, IdentityProvider,
};
use crate::input::InputChannel;
use crate::scripts::ScriptEngine;
use crate::telemetry::TelemetrySink;
use crate::templates::TemplateSource;
use crate::transport::TransportAdapter;
use crate::vendor::VendorAdapter;

pub type ExtensionError = Box<dyn Error + Send + Sync>;

pub const WORKSPACE_WINDOWS: &[&str] = &["main", "workspace_*"];

pub trait Extension<R: Runtime>: Send + Sync + 'static {
    fn id(&self) -> &'static str;

    fn plugins(&self) -> Vec<TauriPlugin<R>> {
        Vec::new()
    }

    fn contribute(
        &self,
        app: &AppHandle<R>,
        contributions: &mut Contributions,
    ) -> Result<(), ExtensionError> {
        let _ = (app, contributions);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Slot {
    Authorization,
    Entitlements,
    Connection,
    Documents,
    Identity,
    AuditStore,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ContributionConflict {
    pub slot: Slot,
}

impl fmt::Display for ContributionConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} is already provided by another extension",
            self.slot
        )
    }
}

impl Error for ContributionConflict {}

#[derive(Default)]
pub struct Contributions {
    authorization: Option<Box<dyn AuthorizationProvider>>,
    entitlements: Option<Box<dyn EntitlementProvider>>,
    audit_sinks: Vec<Box<dyn AuditSink>>,
    connection: Option<Arc<dyn ConnectionProvider>>,
    documents: Option<Arc<dyn DocumentStore>>,
    template_sources: Vec<Box<dyn TemplateSource>>,
    transports: Vec<Box<dyn TransportAdapter>>,
    telemetry_sinks: Vec<Box<dyn TelemetrySink>>,
    vendor_adapters: Vec<Box<dyn VendorAdapter>>,
    input_channels: Vec<Box<dyn InputChannel>>,
    script_engines: Vec<Box<dyn ScriptEngine>>,
    api_authenticators: Vec<Box<dyn ApiAuthenticator>>,
    identity: Option<Box<dyn IdentityProvider>>,
    audit_store: Option<Arc<dyn AuditStore>>,
    assistant_providers: Vec<Box<dyn AssistantProvider>>,
    cloud_providers: Vec<Box<dyn CloudPhoneProvider>>,
}

fn claim<T>(slot: &mut Option<T>, value: T, name: Slot) -> Result<(), ContributionConflict> {
    if slot.is_some() {
        return Err(ContributionConflict { slot: name });
    }

    *slot = Some(value);
    Ok(())
}

impl Contributions {
    pub fn set_authorization(
        &mut self,
        provider: impl AuthorizationProvider + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(
            &mut self.authorization,
            Box::new(provider),
            Slot::Authorization,
        )
    }

    pub fn set_entitlements(
        &mut self,
        provider: impl EntitlementProvider + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(
            &mut self.entitlements,
            Box::new(provider),
            Slot::Entitlements,
        )
    }

    pub fn add_audit_sink(&mut self, sink: impl AuditSink + 'static) {
        self.audit_sinks.push(Box::new(sink));
    }

    pub fn set_connection(
        &mut self,
        provider: impl ConnectionProvider + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(&mut self.connection, Arc::new(provider), Slot::Connection)
    }

    pub fn set_documents(
        &mut self,
        store: impl DocumentStore + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(&mut self.documents, Arc::new(store), Slot::Documents)
    }

    pub fn add_template_source(&mut self, source: impl TemplateSource + 'static) {
        self.template_sources.push(Box::new(source));
    }

    pub fn add_transport(&mut self, transport: impl TransportAdapter + 'static) {
        self.transports.push(Box::new(transport));
    }

    pub fn add_telemetry_sink(&mut self, sink: impl TelemetrySink + 'static) {
        self.telemetry_sinks.push(Box::new(sink));
    }

    pub fn add_vendor_adapter(&mut self, adapter: impl VendorAdapter + 'static) {
        self.vendor_adapters.push(Box::new(adapter));
    }

    pub fn add_input_channel(&mut self, channel: impl InputChannel + 'static) {
        self.input_channels.push(Box::new(channel));
    }

    pub fn add_script_engine(&mut self, engine: impl ScriptEngine + 'static) {
        self.script_engines.push(Box::new(engine));
    }

    pub fn add_api_authenticator(&mut self, authenticator: impl ApiAuthenticator + 'static) {
        self.api_authenticators.push(Box::new(authenticator));
    }

    pub fn set_identity(
        &mut self,
        provider: impl IdentityProvider + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(&mut self.identity, Box::new(provider), Slot::Identity)
    }

    pub fn set_audit_store(
        &mut self,
        store: impl AuditStore + 'static,
    ) -> Result<(), ContributionConflict> {
        claim(&mut self.audit_store, Arc::new(store), Slot::AuditStore)
    }

    pub fn add_assistant_provider(&mut self, provider: impl AssistantProvider + 'static) {
        self.assistant_providers.push(Box::new(provider));
    }

    pub fn add_cloud_provider(&mut self, provider: impl CloudPhoneProvider + 'static) {
        self.cloud_providers.push(Box::new(provider));
    }

    pub fn into_parts(self) -> ContributionParts {
        ContributionParts {
            authorization: self.authorization,
            entitlements: self.entitlements,
            audit_sinks: self.audit_sinks,
            connection: self.connection,
            documents: self.documents,
            template_sources: self.template_sources,
            transports: self.transports,
            telemetry_sinks: self.telemetry_sinks,
            vendor_adapters: self.vendor_adapters,
            input_channels: self.input_channels,
            script_engines: self.script_engines,
            api_authenticators: self.api_authenticators,
            identity: self.identity,
            audit_store: self.audit_store,
            assistant_providers: self.assistant_providers,
            cloud_providers: self.cloud_providers,
        }
    }
}

#[non_exhaustive]
pub struct ContributionParts {
    pub authorization: Option<Box<dyn AuthorizationProvider>>,
    pub entitlements: Option<Box<dyn EntitlementProvider>>,
    pub audit_sinks: Vec<Box<dyn AuditSink>>,
    pub connection: Option<Arc<dyn ConnectionProvider>>,
    pub documents: Option<Arc<dyn DocumentStore>>,
    pub template_sources: Vec<Box<dyn TemplateSource>>,
    pub transports: Vec<Box<dyn TransportAdapter>>,
    pub telemetry_sinks: Vec<Box<dyn TelemetrySink>>,
    pub vendor_adapters: Vec<Box<dyn VendorAdapter>>,
    pub input_channels: Vec<Box<dyn InputChannel>>,
    pub script_engines: Vec<Box<dyn ScriptEngine>>,
    pub api_authenticators: Vec<Box<dyn ApiAuthenticator>>,
    pub identity: Option<Box<dyn IdentityProvider>>,
    pub audit_store: Option<Arc<dyn AuditStore>>,
    pub assistant_providers: Vec<Box<dyn AssistantProvider>>,
    pub cloud_providers: Vec<Box<dyn CloudPhoneProvider>>,
}

#[cfg(test)]
mod tests {
    use super::{Contributions, Slot};
    use crate::contract::yunheshiguang::governance::v1::{AuditEntry, GovernanceProvider};
    use crate::contract::yunheshiguang::workspace::v1::UiText;
    use crate::governance::{
        ActionContext, AuditPage, AuditQuery, AuditStore, AuthorizationProvider, Decision,
        IdentityProvider, Operator,
    };

    struct AllowAll;

    impl AuthorizationProvider for AllowAll {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn authorize(&self, _context: &ActionContext<'_>) -> Decision {
            Decision::Allow
        }
    }

    #[test]
    fn refuses_a_second_provider_for_the_same_contract() {
        let mut contributions = Contributions::default();

        contributions.set_authorization(AllowAll).unwrap();
        let conflict = contributions.set_authorization(AllowAll).unwrap_err();

        assert_eq!(conflict.slot, Slot::Authorization);
        assert!(conflict.to_string().contains("Authorization"));
        assert!(contributions.into_parts().authorization.is_some());
    }

    struct Local;

    impl IdentityProvider for Local {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn current(&self) -> Operator {
            Operator::default()
        }
    }

    struct Forget;

    impl AuditStore for Forget {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider::default()
        }

        fn append(&self, _entry: &AuditEntry) {}

        fn query(&self, _query: &AuditQuery) -> Result<AuditPage, UiText> {
            Ok(AuditPage::default())
        }
    }

    #[test]
    fn keeps_one_identity_and_one_audit_store() {
        let mut contributions = Contributions::default();

        contributions.set_identity(Local).unwrap();
        contributions.set_audit_store(Forget).unwrap();
        assert_eq!(
            contributions.set_identity(Local).unwrap_err().slot,
            Slot::Identity
        );
        assert_eq!(
            contributions.set_audit_store(Forget).unwrap_err().slot,
            Slot::AuditStore
        );

        let parts = contributions.into_parts();
        assert!(parts.identity.is_some());
        assert!(parts.audit_store.is_some());
    }
}
