use std::sync::Arc;

use tauri::{App, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

use super::guard::Governance;

pub struct GovernanceModule {
    governance: Arc<Governance>,
}

impl GovernanceModule {
    pub fn new(governance: Governance) -> Self {
        Self {
            governance: Arc::new(governance),
        }
    }
}

impl<R: Runtime> WorkspaceModule<R> for GovernanceModule {
    fn id(&self) -> &'static str {
        "governance"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::clone(&self.governance));

        let rpc = super::rpc::GovernanceRpc(Arc::clone(&self.governance));
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
