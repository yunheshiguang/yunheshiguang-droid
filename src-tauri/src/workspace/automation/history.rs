use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};
use yunheshiguang_contract::KnownEnum;

use yunheshiguang_sdk::fs::write_atomic;

use crate::workspace::clock;
use crate::workspace::contract::{FlowRun, FlowRuns, RunStatus, StepStatus};
use crate::workspace::local_state::read_json;
use crate::workspace::messages::AUTOMATION_INTERRUPTED_KEY;
use crate::workspace::text::ui_text;

pub const MAX_FINISHED_RUNS: usize = 200;
pub const DETAILED_RUNS: usize = 20;

fn running(run: &FlowRun) -> bool {
    run.status == RunStatus::Running
}

fn interrupt(run: &mut FlowRun) {
    run.status = RunStatus::Failed.into();
    run.finished_at = clock::now().into();
    run.detail = ui_text(AUTOMATION_INTERRUPTED_KEY).into();

    for step in &mut run.steps {
        if matches!(
            StepStatus::known(step.status),
            Some(StepStatus::Pending | StepStatus::Running)
        ) {
            step.status = StepStatus::Skipped.into();
        }
    }
}

fn keep_recent_details(runs: &mut [FlowRun]) {
    for run in runs
        .iter_mut()
        .filter(|run| !running(run))
        .skip(DETAILED_RUNS)
    {
        for step in &mut run.steps {
            step.result = Default::default();
        }
    }
}

pub struct RunHistory {
    path: Option<PathBuf>,
    runs: Mutex<Vec<FlowRun>>,
    writing: Mutex<()>,
    interrupted: Mutex<Vec<FlowRun>>,
}

impl RunHistory {
    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self {
            path: None,
            runs: Mutex::new(Vec::new()),
            writing: Mutex::new(()),
            interrupted: Mutex::new(Vec::new()),
        }
    }

    pub fn load(path: PathBuf) -> Self {
        let mut runs = read_json::<FlowRuns>(&path)
            .map(|stored| stored.runs)
            .unwrap_or_default();
        let mut interrupted = Vec::new();
        for run in runs.iter_mut().filter(|run| running(run)) {
            interrupt(run);
            interrupted.push(run.clone());
        }
        runs.sort_by(|left, right| right.id.cmp(&left.id));
        runs.truncate(MAX_FINISHED_RUNS);
        keep_recent_details(&mut runs);

        Self {
            path: Some(path),
            runs: Mutex::new(runs),
            writing: Mutex::new(()),
            interrupted: Mutex::new(interrupted),
        }
    }

    pub fn take_interrupted(&self) -> Vec<FlowRun> {
        let interrupted = std::mem::take(
            &mut *self
                .interrupted
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        if !interrupted.is_empty() {
            self.persist();
        }
        interrupted
    }

    pub fn interrupt_running(&self) -> Vec<FlowRun> {
        let mut runs = self.runs();
        let mut interrupted = Vec::new();

        for run in runs.iter_mut().filter(|run| running(run)) {
            interrupt(run);
            interrupted.push(run.clone());
        }
        interrupted
    }

    fn runs(&self) -> std::sync::MutexGuard<'_, Vec<FlowRun>> {
        self.runs.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self) -> Vec<FlowRun> {
        self.runs().clone()
    }

    pub fn get(&self, id: &str) -> Option<FlowRun> {
        self.runs().iter().find(|run| run.id == id).cloned()
    }

    pub fn latest_for_schedule(&self, schedule_id: &str) -> Option<FlowRun> {
        self.runs()
            .iter()
            .find(|run| run.schedule_id == schedule_id)
            .cloned()
    }

    pub fn update(&self, run: &FlowRun) {
        let mut runs = self.runs();

        match runs.iter_mut().find(|candidate| candidate.id == run.id) {
            Some(existing) => *existing = run.clone(),
            None => {
                runs.push(run.clone());
                runs.sort_by(|left, right| right.id.cmp(&left.id));
            }
        }

        let mut finished = 0;
        runs.retain(|candidate| {
            if running(candidate) {
                return true;
            }
            finished += 1;
            finished <= MAX_FINISHED_RUNS
        });
        keep_recent_details(&mut runs);
    }

    pub fn persist(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let _writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
        let snapshot = FlowRuns {
            runs: self.list(),
            ..Default::default()
        };
        let written = serde_json::to_vec(&snapshot)
            .map_err(std::io::Error::other)
            .and_then(|bytes| write_atomic(path, &bytes));

        if let Err(error) = written {
            log::warn!("automation: run history not saved: {error}");
        }
    }
}
