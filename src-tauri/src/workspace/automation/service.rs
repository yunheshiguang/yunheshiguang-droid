use std::sync::Arc;
use std::time::Instant;

use yunheshiguang_sdk::documents::DocumentStore;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::*;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::flows::list_flows;
use super::runner::{AUTOMATION_CHANGED_EVENT, FlowRunner, StartRun};
use super::schedules::{ScheduleBook, check_schedule};

pub struct AutomationService {
    pub(super) runner: Arc<FlowRunner>,
    pub(super) schedules: Arc<ScheduleBook>,
    pub(super) documents: Arc<dyn DocumentStore>,
    pub(super) governance: Arc<Governance>,
    pub(super) telemetry: Arc<Telemetry>,
    pub(super) events: Arc<dyn WorkspaceEvents>,
}

fn refused(detail: UiText) -> ScheduleResult {
    ScheduleResult {
        accepted: false,
        detail: detail.into(),
        ..Default::default()
    }
}

impl AutomationService {
    pub async fn start(
        &self,
        request: StartFlowRunRequest,
        trigger: RunTrigger,
        principal: &Principal,
    ) -> FlowRunResult {
        self.runner
            .start(StartRun::requested(request, trigger), principal)
            .await
    }

    pub fn stop(&self, run_id: &str, principal: &Principal) -> FlowRunResult {
        self.runner.stop(run_id, principal)
    }

    pub fn runs(&self) -> FlowRuns {
        self.runner.runs()
    }

    pub fn run(&self, id: &str) -> Option<FlowRun> {
        self.runner.history().get(id)
    }

    pub fn schedules(&self) -> Schedules {
        self.schedules.states(self.runner.history())
    }

    pub async fn flows(&self) -> Result<Flows, UiText> {
        let documents = Arc::clone(&self.documents);

        run_blocking(move || Ok(list_flows(documents.as_ref())))
            .await
            .unwrap_or_else(|error| {
                Err(ui_text_with_params(
                    AUTOMATION_LOAD_FAILED_KEY,
                    "",
                    [("message", error)],
                ))
            })
    }

    async fn manage(
        &self,
        principal: &Principal,
        work: impl FnOnce(&ScheduleBook) -> Result<UiText, UiText> + Send + 'static,
    ) -> ScheduleResult {
        let ticket = match self
            .governance
            .begin(GovernedAction::ManageSchedules, principal, &[])
        {
            Ok(ticket) => ticket,
            Err(reason) => return refused(reason),
        };
        let started = Instant::now();
        let book = Arc::clone(&self.schedules);
        let outcome = run_blocking(move || Ok(work(&book)))
            .await
            .unwrap_or_else(|error| {
                Err(ui_text_with_params(
                    SCHEDULE_STORAGE_FAILED_KEY,
                    "",
                    [("message", error)],
                ))
            });
        let result = match outcome {
            Ok(audit) => {
                ticket.finish(AuditOutcome::Succeeded, Some(audit));
                notify(
                    self.events.as_ref(),
                    AUTOMATION_CHANGED_EVENT,
                    &AutomationChanged::default(),
                );
                ScheduleResult {
                    accepted: true,
                    detail: Default::default(),
                    ..Default::default()
                }
            }
            Err(detail) => {
                ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
                refused(detail)
            }
        };

        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(
                GovernedAction::ManageSchedules,
                result.accepted,
                started.elapsed(),
            )
            .with_targets(1, u32::from(!result.accepted)),
        ));
        result
    }

    pub async fn save_schedule(
        &self,
        request: SaveScheduleRequest,
        principal: &Principal,
    ) -> ScheduleResult {
        let mut schedule = match check_schedule(request.schedule.into_option().unwrap_or_default())
        {
            Ok(schedule) => schedule,
            Err(detail) => return refused(detail),
        };
        schedule.owner = principal.operator.clone().into();

        self.manage(principal, move |book| {
            let name = schedule.name.clone();
            book.save(schedule)?;
            Ok(ui_text_with_params(
                SCHEDULE_AUDIT_SAVED_KEY,
                "",
                [("name", name)],
            ))
        })
        .await
    }

    pub async fn remove_schedule(
        &self,
        request: RemoveScheduleRequest,
        principal: &Principal,
    ) -> ScheduleResult {
        let id = request.id.trim().to_string();

        self.manage(principal, move |book| {
            let name = book
                .remove(&id)?
                .ok_or_else(|| ui_text(SCHEDULE_MISSING_KEY))?
                .name;
            Ok(ui_text_with_params(
                SCHEDULE_AUDIT_REMOVED_KEY,
                "",
                [("name", name)],
            ))
        })
        .await
    }
}
