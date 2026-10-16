use std::collections::{BTreeMap, VecDeque};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use yunheshiguang_sdk::telemetry::{OperationSample, ProbeSample, TelemetryEvent, sink_info};

use crate::workspace::clock::timestamp;
use crate::workspace::contract::{OperationHealth, ProbeHealth, TelemetrySinkInfo};
use crate::workspace::messages::{TELEMETRY_HEALTH_DESCRIPTION_KEY, TELEMETRY_HEALTH_LABEL_KEY};

pub const PROBE_CAPACITY: usize = 60;

fn millis(duration: Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}

#[derive(Default)]
struct OperationStats {
    runs: u32,
    failures: u32,
    total_ms: u64,
    slowest_ms: u32,
    last_finished_at: Option<SystemTime>,
}

#[derive(Default)]
struct HealthState {
    probes: VecDeque<ProbeHealth>,
    operations: BTreeMap<(i32, i32), OperationStats>,
}

#[derive(Default)]
pub struct HealthMonitor {
    state: Mutex<HealthState>,
}

impl HealthMonitor {
    pub fn describe(&self) -> TelemetrySinkInfo {
        sink_info(
            "core.health",
            TELEMETRY_HEALTH_LABEL_KEY,
            TELEMETRY_HEALTH_DESCRIPTION_KEY,
        )
    }

    pub fn record(&self, event: &TelemetryEvent) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        match event {
            TelemetryEvent::OperationFinished(sample) => operation(&mut state, sample),
            TelemetryEvent::DevicesProbed(sample) => probe(&mut state, sample),
            _ => {}
        }
    }

    pub fn probes(&self) -> Vec<ProbeHealth> {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        state.probes.iter().cloned().collect()
    }

    pub fn operations(&self) -> Vec<OperationHealth> {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);

        state
            .operations
            .iter()
            .map(|(&(action, batch_action), stats)| OperationHealth {
                action: action.into(),
                batch_action: batch_action.into(),
                runs: stats.runs,
                failures: stats.failures,
                average_ms: u32::try_from(stats.total_ms / u64::from(stats.runs.max(1)))
                    .unwrap_or(u32::MAX),
                slowest_ms: stats.slowest_ms,
                last_finished_at: stats.last_finished_at.map(timestamp).into(),
                ..Default::default()
            })
            .collect()
    }
}

fn operation(state: &mut HealthState, sample: &OperationSample) {
    let stats = state
        .operations
        .entry((sample.action as i32, sample.batch_action as i32))
        .or_default();
    let elapsed = millis(sample.duration);

    stats.runs = stats.runs.saturating_add(1);
    stats.failures = stats.failures.saturating_add(u32::from(!sample.succeeded));
    stats.total_ms = stats.total_ms.saturating_add(u64::from(elapsed));
    stats.slowest_ms = stats.slowest_ms.max(elapsed);
    stats.last_finished_at = Some(sample.finished_at);
}

fn probe(state: &mut HealthState, sample: &ProbeSample) {
    state.probes.push_front(ProbeHealth {
        probed_at: timestamp(sample.probed_at).into(),
        duration_ms: millis(sample.duration),
        devices: sample.devices,
        online: sample.online,
        unauthorized: sample.unauthorized,
        offline: sample.offline,
        ..Default::default()
    });
    state.probes.truncate(PROBE_CAPACITY);
}
