use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use tokio::sync::watch;
use uuid::Uuid;
use yunheshiguang_sdk::documents::DocumentStore;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use crate::workspace::batch::{BatchService, fill_request, reaches_offline, uses_host_files};
use crate::workspace::blocking::run_blocking;
use crate::workspace::clock;
use crate::workspace::contract::*;
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{AuditTicket, Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::snapshot::SnapshotService;
use crate::workspace::telemetry::Telemetry;
use crate::workspace::text::{ui_text, ui_text_with_params};

use super::flows::load_flow;
use super::history::RunHistory;
use super::options::{RunPlan, check_options};
use super::steps::{failed_devices, merge, pending_steps, step_done, trim_outputs, wait};
use super::targets::{check_target, resolve_target};

pub const AUTOMATION_CHANGED_EVENT: &str = "workspace:automation-changed";

pub type StepFuture = Pin<Box<dyn Future<Output = BatchActionResult> + Send>>;

pub trait StepExecutor: Send + Sync {
    fn execute(&self, request: BatchActionRequest, principal: Principal) -> StepFuture;
}

impl StepExecutor for Arc<BatchService> {
    fn execute(&self, request: BatchActionRequest, principal: Principal) -> StepFuture {
        let service = Arc::clone(self);

        Box::pin(async move { service.run(request, &principal).await })
    }
}

pub trait DeviceSource: Send + Sync {
    fn snapshot(&self) -> WorkspaceSnapshot;
}

impl DeviceSource for SnapshotService {
    fn snapshot(&self) -> WorkspaceSnapshot {
        self.build()
    }
}

#[derive(Clone, Debug)]
pub struct StartRun {
    pub flow_id: String,
    pub target: Option<RunTarget>,
    pub options: Option<RunOptions>,
    pub trigger: RunTrigger,
    pub schedule_id: String,
}

impl StartRun {
    pub fn requested(request: StartFlowRunRequest, trigger: RunTrigger) -> Self {
        Self {
            flow_id: request.flow_id,
            target: request.target.into(),
            options: request.options.into(),
            trigger,
            schedule_id: String::new(),
        }
    }

    pub fn scheduled(schedule: &Schedule) -> Self {
        Self {
            flow_id: schedule.flow_id.clone(),
            target: schedule.target.clone().into(),
            options: schedule.options.clone().into(),
            trigger: RunTrigger::Schedule,
            schedule_id: schedule.id.clone(),
        }
    }
}

pub struct FlowRunner {
    executor: Box<dyn StepExecutor>,
    devices: Arc<dyn DeviceSource>,
    documents: Arc<dyn DocumentStore>,
    governance: Arc<Governance>,
    telemetry: Arc<Telemetry>,
    events: Arc<dyn WorkspaceEvents>,
    history: Arc<RunHistory>,
    stops: Mutex<HashMap<String, watch::Sender<bool>>>,
}

fn refused(detail: UiText) -> FlowRunResult {
    FlowRunResult {
        accepted: false,
        detail: detail.into(),
        run: Default::default(),
        ..Default::default()
    }
}

impl FlowRunner {
    pub fn new(
        executor: Box<dyn StepExecutor>,
        devices: Arc<dyn DeviceSource>,
        documents: Arc<dyn DocumentStore>,
        governance: Arc<Governance>,
        telemetry: Arc<Telemetry>,
        events: Arc<dyn WorkspaceEvents>,
        history: Arc<RunHistory>,
    ) -> Self {
        Self {
            executor,
            devices,
            documents,
            governance,
            telemetry,
            events,
            history,
            stops: Mutex::new(HashMap::new()),
        }
    }

    pub fn history(&self) -> &RunHistory {
        &self.history
    }

    pub fn interrupt_running(&self) {
        let interrupted = self.history.interrupt_running();
        self.stops().clear();
        if !interrupted.is_empty() {
            self.history.persist();
        }
    }

    pub fn runs(&self) -> FlowRuns {
        FlowRuns {
            runs: self.history.list(),
            ..Default::default()
        }
    }

    fn stops(&self) -> std::sync::MutexGuard<'_, HashMap<String, watch::Sender<bool>>> {
        self.stops.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn publish(&self, run: &FlowRun, persist: bool) {
        self.history.update(run);

        if persist {
            let history = Arc::clone(&self.history);
            tauri::async_runtime::spawn_blocking(move || history.persist());
        }
        notify(
            self.events.as_ref(),
            AUTOMATION_CHANGED_EVENT,
            &AutomationChanged {
                run_id: run.id.clone(),
                ..Default::default()
            },
        );
    }

    pub async fn start(self: &Arc<Self>, start: StartRun, principal: &Principal) -> FlowRunResult {
        let plan = match check_options(start.options.as_ref()) {
            Ok(plan) => plan,
            Err(detail) => return refused(detail),
        };
        let target = match check_target(start.target.as_ref()) {
            Ok(target) => target.clone(),
            Err(detail) => return refused(detail),
        };
        let runner = Arc::clone(self);
        let flow_id = start.flow_id.clone();
        let resolved = target.clone();
        let prepared = run_blocking(move || {
            Ok(
                load_flow(runner.documents.as_ref(), &flow_id).and_then(|flow| {
                    let offline = flow
                        .steps
                        .iter()
                        .any(|step| reaches_offline(step.kind.to_i32()));
                    resolve_target(&resolved, offline, || runner.devices.snapshot())
                        .map(|devices| (flow, devices))
                }),
            )
        })
        .await;
        let (flow, device_ids) = match prepared {
            Ok(Ok(prepared)) => prepared,
            Ok(Err(detail)) => return refused(detail),
            Err(error) => {
                return refused(ui_text_with_params(
                    AUTOMATION_LOAD_FAILED_KEY,
                    "",
                    [("message", error)],
                ));
            }
        };
        let ticket =
            match self
                .governance
                .begin(GovernedAction::RunAutomation, principal, &device_ids)
            {
                Ok(ticket) => ticket,
                Err(reason) => {
                    log::info!("automation: run of {} denied by governance", start.flow_id);
                    return refused(reason);
                }
            };
        if principal.is_api()
            && flow
                .steps
                .iter()
                .any(|step| uses_host_files(step.kind.to_i32()))
        {
            log::info!(
                "automation: refused host files in {} for a local API caller",
                start.flow_id
            );
            let reason = ui_text(API_HOST_FILES_KEY);
            ticket.finish(AuditOutcome::Denied, Some(reason.clone()));
            return refused(reason);
        }

        let run = FlowRun {
            id: Uuid::now_v7().to_string(),
            flow_id: start.flow_id,
            flow_name: flow.name.clone(),
            trigger: start.trigger.into(),
            actor: principal.actor.clone(),
            status: RunStatus::Running.into(),
            device_ids,
            iteration: 1,
            iterations: plan.iterations,
            steps: pending_steps(&flow),
            started_at: clock::now().into(),
            finished_at: Default::default(),
            schedule_id: start.schedule_id,
            detail: Default::default(),
            target: start.target.into(),
            options: start.options.into(),
            operator: principal.operator.clone().into(),
            ..Default::default()
        };
        let (stop, stopped) = watch::channel(false);
        self.stops().insert(run.id.clone(), stop);
        self.publish(&run, true);
        log::info!(
            "automation: run {} of {} started on {} device(s)",
            run.id,
            run.flow_id,
            run.device_ids.len()
        );

        let runner = Arc::clone(self);
        let first = run.clone();
        tauri::async_runtime::spawn(runner.execute(run, flow, plan, ticket, stopped));

        FlowRunResult {
            accepted: true,
            detail: Default::default(),
            run: first.into(),
            ..Default::default()
        }
    }

    pub fn stop(self: &Arc<Self>, run_id: &str, principal: &Principal) -> FlowRunResult {
        let Some(run) = self
            .history
            .get(run_id)
            .filter(|_| self.stops().contains_key(run_id))
        else {
            return refused(ui_text(AUTOMATION_RUN_NOT_RUNNING_KEY));
        };
        let ticket =
            match self
                .governance
                .begin(GovernedAction::RunAutomation, principal, &run.device_ids)
            {
                Ok(ticket) => ticket,
                Err(reason) => return refused(reason),
            };
        let requested = self
            .stops()
            .get(run_id)
            .map(|stop| stop.send_replace(true))
            .is_some();

        if requested {
            ticket.finish(
                AuditOutcome::Succeeded,
                Some(ui_text_with_params(
                    AUTOMATION_AUDIT_STOPPED_KEY,
                    "",
                    [("name", run.flow_name)],
                )),
            );
            FlowRunResult {
                accepted: true,
                detail: Default::default(),
                run: self.history.get(run_id).into(),
                ..Default::default()
            }
        } else {
            let detail = ui_text(AUTOMATION_RUN_NOT_RUNNING_KEY);
            ticket.finish(AuditOutcome::Failed, Some(detail.clone()));
            refused(detail)
        }
    }

    async fn execute(
        self: Arc<Self>,
        mut run: FlowRun,
        flow: Flow,
        plan: RunPlan,
        ticket: AuditTicket,
        mut stop: watch::Receiver<bool>,
    ) {
        let started = Instant::now();
        let principal = Principal::new(
            run.actor.clone(),
            run.operator.clone().into_option().unwrap_or_default(),
        );
        let mut failure = None;
        let mut failed = 0;
        let mut interrupted = false;

        'iterations: for iteration in 1..=plan.iterations {
            if iteration > 1 {
                if wait(plan.interval, &mut stop).await {
                    interrupted = true;
                    break;
                }
                run.iteration = iteration;
                run.steps = pending_steps(&flow);
                self.publish(&run, false);
            }

            for (index, step) in flow.steps.iter().enumerate() {
                if *stop.borrow() {
                    interrupted = true;
                    break 'iterations;
                }
                run.steps[index].status = StepStatus::Running.into();
                self.publish(&run, false);

                let (result, attempts) = self
                    .run_step(&mut run, index, &principal, step, &plan, &mut stop)
                    .await;
                let done = step_done(&result);
                let record = &mut run.steps[index];
                record.attempts = attempts;
                record.status = if done {
                    StepStatus::Done.into()
                } else {
                    StepStatus::Failed.into()
                };
                if !done {
                    failed = failed_devices(&result).len().max(1);
                }
                record.result = trim_outputs(result).into();

                if !done {
                    failure = Some(ui_text_with_params(
                        AUTOMATION_STEP_FAILED_KEY,
                        "",
                        [("step", (index + 1).to_string())],
                    ));
                    break 'iterations;
                }
                self.publish(&run, false);
            }
        }

        let stopped = interrupted || (failure.is_some() && *stop.borrow());
        for step in &mut run.steps {
            if step.status == StepStatus::Pending {
                step.status = StepStatus::Skipped.into();
            }
        }
        let status = if stopped {
            RunStatus::Stopped
        } else if failure.is_some() {
            RunStatus::Failed
        } else {
            RunStatus::Succeeded
        };
        run.status = status.into();
        run.detail = if stopped {
            ui_text(AUTOMATION_STOPPED_KEY).into()
        } else {
            failure.into()
        };
        run.finished_at = clock::now().into();
        self.stops().remove(&run.id);
        self.publish(&run, true);
        log::info!(
            "automation: run {} finished as {}",
            run.id,
            status.proto_name()
        );

        let succeeded = status == RunStatus::Succeeded;
        ticket.finish(
            if succeeded {
                AuditOutcome::Succeeded
            } else {
                AuditOutcome::Failed
            },
            Some(ui_text_with_params(
                AUTOMATION_AUDIT_RUN_KEY,
                "",
                [("name", run.flow_name.clone())],
            )),
        );
        let targets = u32::try_from(run.device_ids.len()).unwrap_or(u32::MAX);
        let failed = if succeeded {
            0
        } else {
            u32::try_from(failed).unwrap_or(u32::MAX).min(targets)
        };
        self.telemetry.record(TelemetryEvent::OperationFinished(
            OperationSample::new(GovernedAction::RunAutomation, succeeded, started.elapsed())
                .with_targets(targets, failed),
        ));
    }

    async fn run_step(
        &self,
        run: &mut FlowRun,
        index: usize,
        principal: &Principal,
        step: &FlowStep,
        plan: &RunPlan,
        stop: &mut watch::Receiver<bool>,
    ) -> (BatchActionResult, u32) {
        let kind = BatchActionKind::known(step.kind).unwrap_or(BatchActionKind::Unspecified);
        let mut devices = run.device_ids.clone();
        let mut previous = None;
        let mut attempts = 0;

        loop {
            attempts += 1;
            let mut request = BatchActionRequest {
                device_ids: devices.clone(),
                ..BatchActionRequest::default()
            };
            fill_request(&mut request, kind, &step.value);

            let result = merge(
                previous.take(),
                self.executor.execute(request, principal.clone()).await,
            );
            if step_done(&result) || attempts > plan.retries {
                return (result, attempts);
            }

            let failed = failed_devices(&result);
            if !failed.is_empty() {
                devices = failed;
            }
            let record = &mut run.steps[index];
            record.attempts = attempts;
            record.result = trim_outputs(result.clone()).into();
            self.publish(run, false);
            if wait(plan.retry_delay, stop).await {
                return (result, attempts);
            }
            previous = Some(result);
        }
    }
}
