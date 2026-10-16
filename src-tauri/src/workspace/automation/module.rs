use std::sync::Arc;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::documents::{DocumentStore, FolderDocumentStore};

use crate::workspace::batch::BatchService;
use crate::workspace::contract::GovernedAction;
use crate::workspace::documents::DocumentsService;
use crate::workspace::events::WorkspaceEvents;
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::lifecycle::Shutdown;
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};
use crate::workspace::snapshot::SnapshotService;

use super::flows::{FLOWS_COLLECTION, host_files_refusal};
use super::history::RunHistory;
use super::runner::FlowRunner;
use super::scheduler::run_scheduler;
use super::schedules::ScheduleBook;
use super::service::AutomationService;

const AUTOMATION_DIRECTORY: &str = "automation";

pub struct AutomationModule;

impl<R: Runtime> WorkspaceModule<R> for AutomationModule {
    fn id(&self) -> &'static str {
        "automation"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let directory = app.path().app_local_data_dir()?.join(AUTOMATION_DIRECTORY);
        let documents = dependency::<dyn DocumentStore, R>(app)?;
        let events = dependency::<dyn WorkspaceEvents, R>(app)?;
        let governance = dependency::<Governance, R>(app)?;
        let telemetry = dependency(app)?;
        dependency::<DocumentsService, R>(app)?
            .check_api_writes(FLOWS_COLLECTION, host_files_refusal);
        let history = Arc::new(RunHistory::load(directory.join("runs.json")));
        for run in history.take_interrupted() {
            governance.audit_interrupted(
                GovernedAction::RunAutomation,
                &Principal::new(
                    run.actor.clone(),
                    run.operator.clone().into_option().unwrap_or_default(),
                ),
                &run.device_ids,
            );
        }
        let schedules = Arc::new(ScheduleBook::open(Arc::new(FolderDocumentStore::new(
            directory.clone(),
        ))));
        let runner = Arc::new(FlowRunner::new(
            Box::new(dependency::<BatchService, R>(app)?),
            dependency::<SnapshotService, R>(app)?,
            Arc::clone(&documents),
            Arc::clone(&governance),
            Arc::clone(&telemetry),
            Arc::clone(&events),
            history,
        ));

        let stopping = Arc::clone(&runner);
        dependency::<Shutdown, R>(app)?.on_exit(move || stopping.interrupt_running());
        tauri::async_runtime::spawn(run_scheduler(
            Arc::clone(&schedules),
            Arc::clone(&runner),
            Arc::clone(&governance),
            Arc::clone(&events),
        ));
        app.manage(Arc::new(AutomationService {
            runner,
            schedules,
            documents,
            governance,
            telemetry,
            events,
        }));

        let rpc = super::rpc::AutomationRpc {
            service: dependency(app)?,
            governance: dependency(app)?,
        };
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}
