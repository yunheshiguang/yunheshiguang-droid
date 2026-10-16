use yunheshiguang_sdk::telemetry::{TelemetryEvent, TelemetrySink};

use crate::workspace::contract::WorkspaceHealth;

use super::health::{HealthMonitor, PROBE_CAPACITY};

pub struct Telemetry {
    health: HealthMonitor,
    sinks: Vec<Box<dyn TelemetrySink>>,
}

impl Telemetry {
    pub fn assemble(sinks: Vec<Box<dyn TelemetrySink>>) -> Self {
        Self {
            health: HealthMonitor::default(),
            sinks,
        }
    }

    pub fn record(&self, event: TelemetryEvent) {
        self.health.record(&event);

        for sink in &self.sinks {
            sink.record(&event);
        }
    }

    pub fn health(&self) -> WorkspaceHealth {
        WorkspaceHealth {
            probes: self.health.probes(),
            probe_capacity: u32::try_from(PROBE_CAPACITY).unwrap_or(u32::MAX),
            operations: self.health.operations(),
            sinks: std::iter::once(self.health.describe())
                .chain(self.sinks.iter().map(|sink| sink.describe()))
                .collect(),
            ..Default::default()
        }
    }
}
