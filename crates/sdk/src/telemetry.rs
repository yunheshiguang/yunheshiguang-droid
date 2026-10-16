use std::time::{Duration, SystemTime};

use crate::contract::yunheshiguang::governance::v1::GovernedAction;
use crate::contract::yunheshiguang::telemetry::v1::TelemetrySinkInfo;
use crate::contract::yunheshiguang::workspace::v1::BatchActionKind;
use crate::text::ui_text;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct OperationSample {
    pub action: GovernedAction,
    pub batch_action: BatchActionKind,
    pub succeeded: bool,
    pub duration: Duration,
    pub targets: u32,
    pub failed_targets: u32,
    pub finished_at: SystemTime,
}

impl OperationSample {
    pub fn new(action: GovernedAction, succeeded: bool, duration: Duration) -> Self {
        Self {
            action,
            batch_action: BatchActionKind::Unspecified,
            succeeded,
            duration,
            targets: 0,
            failed_targets: 0,
            finished_at: SystemTime::now(),
        }
    }

    #[must_use]
    pub fn with_batch_action(mut self, batch_action: BatchActionKind) -> Self {
        self.batch_action = batch_action;
        self
    }

    #[must_use]
    pub fn with_targets(mut self, targets: u32, failed_targets: u32) -> Self {
        self.targets = targets;
        self.failed_targets = failed_targets;
        self
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeSample {
    pub duration: Duration,
    pub devices: u32,
    pub online: u32,
    pub unauthorized: u32,
    pub offline: u32,
    pub probed_at: SystemTime,
}

impl ProbeSample {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            devices: 0,
            online: 0,
            unauthorized: 0,
            offline: 0,
            probed_at: SystemTime::now(),
        }
    }

    #[must_use]
    pub fn with_counts(
        mut self,
        devices: u32,
        online: u32,
        unauthorized: u32,
        offline: u32,
    ) -> Self {
        self.devices = devices;
        self.online = online;
        self.unauthorized = unauthorized;
        self.offline = offline;
        self
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum TelemetryEvent {
    OperationFinished(OperationSample),
    DevicesProbed(ProbeSample),
}

pub trait TelemetrySink: Send + Sync {
    fn describe(&self) -> TelemetrySinkInfo;

    fn record(&self, event: &TelemetryEvent);
}

pub fn sink_info(id: &str, label_key: &str, description_key: &str) -> TelemetrySinkInfo {
    TelemetrySinkInfo {
        id: id.to_string(),
        label: ui_text(label_key).into(),
        description: ui_text(description_key).into(),
        ..Default::default()
    }
}
