use std::sync::Arc;
use std::time::Duration;

use chrono::Local;

use crate::workspace::contract::{AutomationChanged, RunStatus, Schedule};
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};

use super::runner::{AUTOMATION_CHANGED_EVENT, FlowRunner, StartRun};
use super::schedules::ScheduleBook;

const MAX_SLEEP: Duration = Duration::from_secs(30);

pub fn schedule_principal(schedule: &Schedule, governance: &Governance) -> Principal {
    Principal::new(
        format!("schedule:{}", schedule.name),
        schedule
            .owner
            .clone()
            .into_option()
            .unwrap_or_else(|| governance.current_operator()),
    )
}

pub async fn run_scheduler(
    book: Arc<ScheduleBook>,
    runner: Arc<FlowRunner>,
    governance: Arc<Governance>,
    events: Arc<dyn WorkspaceEvents>,
) {
    loop {
        for schedule in book.due(&Local::now()) {
            let busy = runner
                .history()
                .latest_for_schedule(&schedule.id)
                .is_some_and(|run| run.status == RunStatus::Running);

            if busy {
                log::info!(
                    "automation: schedule {} skipped; its last run is still going",
                    schedule.id
                );
                continue;
            }

            let result = runner
                .start(
                    StartRun::scheduled(&schedule),
                    &schedule_principal(&schedule, &governance),
                )
                .await;
            if !result.accepted {
                log::info!("automation: schedule {} could not start a run", schedule.id);
                notify(
                    events.as_ref(),
                    AUTOMATION_CHANGED_EVENT,
                    &AutomationChanged::default(),
                );
            }
            book.record_problem(
                &schedule.id,
                if result.accepted {
                    None
                } else {
                    result.detail.into()
                },
            );
        }

        let sleep = book
            .next_due()
            .map_or(MAX_SLEEP, |next| {
                (next - Local::now()).to_std().unwrap_or(Duration::ZERO)
            })
            .min(MAX_SLEEP);

        tokio::select! {
            () = tokio::time::sleep(sleep) => {}
            () = book.changed() => {}
        }
    }
}
