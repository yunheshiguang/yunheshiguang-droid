use std::sync::Arc;
use std::time::Duration;

use tauri::{App, Manager, Runtime};

use crate::workspace::events::WorkspaceEvents;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};

use super::build_info::BuildInfo;
use super::checker::Updates;
use super::install::InstallTarget;
use super::rpc::UpdatesRpc;

const UPDATES_DIRECTORY: &str = "updates";
const SETTINGS_FILE: &str = "settings.json";
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

pub struct UpdatesModule;

impl<R: Runtime> WorkspaceModule<R> for UpdatesModule {
    fn id(&self) -> &'static str {
        "updates"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let settings = app
            .path()
            .app_local_data_dir()?
            .join(UPDATES_DIRECTORY)
            .join(SETTINGS_FILE);
        let updates = Arc::new(Updates::new(
            BuildInfo::current(),
            InstallTarget::current(),
            settings,
            dependency::<dyn WorkspaceEvents, R>(app)?,
        ));
        app.manage(Arc::clone(&updates));

        if updates.follows_a_channel() {
            let scheduled = Arc::clone(&updates);
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(FIRST_CHECK).await;
                loop {
                    if scheduled.checks_automatically() {
                        scheduled.check(true).await;
                    }
                    tokio::time::sleep(CHECK_INTERVAL).await;
                }
            });
        }

        let rpc = UpdatesRpc(updates);
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
