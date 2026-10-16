use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::governance::Governance;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::sqlite::SqliteAuditStore;

const AUDIT_DATABASE: [&str; 2] = ["audit", "audit.sqlite3"];

pub struct AuditModule;

impl<R: Runtime> WorkspaceModule<R> for AuditModule {
    fn id(&self) -> &'static str {
        "audit"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let governance = dependency::<Governance, R>(app)?;
        let rpc = super::rpc::AuditRpc(Arc::clone(&governance));
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        if governance.has_audit_store() {
            return Ok(());
        }

        let path = AUDIT_DATABASE
            .iter()
            .fold(app.path().app_local_data_dir()?, |path, part| {
                path.join(part)
            });

        match SqliteAuditStore::open(&path) {
            Ok(store) => {
                governance.install_audit_store(Arc::new(store));
            }
            Err(error) => {
                log::error!("audit store {} is unusable: {error}", path.display());
            }
        }

        Ok(())
    }
}
