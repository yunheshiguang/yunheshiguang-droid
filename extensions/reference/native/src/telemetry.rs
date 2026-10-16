use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use yunheshiguang_sdk::contract::Enumeration;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;
use yunheshiguang_sdk::contract::yunheshiguang::telemetry::v1::TelemetrySinkInfo;
use yunheshiguang_sdk::fs::write_atomic;
use yunheshiguang_sdk::telemetry::{
    OperationSample, ProbeSample, TelemetryEvent, TelemetrySink, sink_info,
};

use crate::messages::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct OperationLabels {
    operation: String,
    batch_action: String,
    outcome: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct KindLabels {
    operation: String,
    batch_action: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct StateLabels {
    state: String,
}

fn duration_histogram() -> Histogram {
    Histogram::new(exponential_buckets(0.05, 2.0, 12))
}

struct Metrics {
    registry: Registry,
    operations: Family<OperationLabels, Counter>,
    durations: Family<KindLabels, Histogram, fn() -> Histogram>,
    devices: Family<StateLabels, Gauge>,
    probe_seconds: Histogram,
}

impl Metrics {
    fn new() -> Self {
        let mut registry = Registry::with_prefix("yunheshiguang");
        let operations = Family::<OperationLabels, Counter>::default();
        let durations = Family::<KindLabels, Histogram, fn() -> Histogram>::new_with_constructor(
            duration_histogram,
        );
        let devices = Family::<StateLabels, Gauge>::default();
        let probe_seconds = duration_histogram();

        registry.register(
            "operations",
            "Governed operations that finished",
            operations.clone(),
        );
        registry.register(
            "operation_duration_seconds",
            "How long governed operations took",
            durations.clone(),
        );
        registry.register(
            "devices",
            "Devices the host saw at the last probe, by state",
            devices.clone(),
        );
        registry.register(
            "probe_duration_seconds",
            "How long probing the devices took",
            probe_seconds.clone(),
        );

        Self {
            registry,
            operations,
            durations,
            devices,
            probe_seconds,
        }
    }

    fn operation(&self, sample: &OperationSample) {
        let operation = short_name(sample.action.proto_name(), "GOVERNED_ACTION_");
        let batch_action = short_name(sample.batch_action.proto_name(), "BATCH_ACTION_KIND_");
        let outcome = if sample.succeeded {
            "succeeded"
        } else {
            "failed"
        };

        self.operations
            .get_or_create(&OperationLabels {
                operation: operation.clone(),
                batch_action: batch_action.clone(),
                outcome: outcome.to_string(),
            })
            .inc();
        self.durations
            .get_or_create(&KindLabels {
                operation,
                batch_action,
            })
            .observe(sample.duration.as_secs_f64());
    }

    fn probe(&self, sample: &ProbeSample) {
        for (state, count) in [
            ("total", sample.devices),
            ("online", sample.online),
            ("unauthorized", sample.unauthorized),
            ("offline", sample.offline),
        ] {
            self.devices
                .get_or_create(&StateLabels {
                    state: state.to_string(),
                })
                .set(i64::from(count));
        }
        self.probe_seconds.observe(sample.duration.as_secs_f64());
    }

    fn render(&self) -> String {
        let mut text = String::new();

        let _ = encode(&mut text, &self.registry);
        text
    }
}

fn short_name(name: &str, prefix: &str) -> String {
    match name.trim_start_matches(prefix) {
        "UNSPECIFIED" => "none".to_string(),
        short => short.to_ascii_lowercase(),
    }
}

pub struct PrometheusTelemetry {
    path: PathBuf,
    metrics: Mutex<Metrics>,
}

impl PrometheusTelemetry {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            metrics: Mutex::new(Metrics::new()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl TelemetrySink for PrometheusTelemetry {
    fn describe(&self) -> TelemetrySinkInfo {
        sink_info(
            "reference.prometheus",
            REFERENCE_PROMETHEUS_LABEL_KEY,
            REFERENCE_PROMETHEUS_DESCRIPTION_KEY,
        )
    }

    fn record(&self, event: &TelemetryEvent) {
        let metrics = self.metrics.lock().unwrap_or_else(PoisonError::into_inner);

        match event {
            TelemetryEvent::OperationFinished(sample) => metrics.operation(sample),
            TelemetryEvent::DevicesProbed(sample) => metrics.probe(sample),
            _ => return,
        }
        if let Err(error) = write_atomic(&self.path, metrics.render().as_bytes()) {
            log::warn!("could not write {}: {error}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::GovernedAction;
    use yunheshiguang_sdk::contract::yunheshiguang::workspace::v1::BatchActionKind;

    use super::*;

    #[test]
    fn writes_openmetrics_text_a_scraper_can_read() {
        let folder = tempfile::tempdir().unwrap();
        let sink = PrometheusTelemetry::new(folder.path().join("nested/metrics.prom"));

        sink.record(&TelemetryEvent::OperationFinished(
            OperationSample::new(
                GovernedAction::RunBatchAction,
                false,
                Duration::from_millis(1_200),
            )
            .with_batch_action(BatchActionKind::Screenshot),
        ));
        sink.record(&TelemetryEvent::DevicesProbed(
            ProbeSample::new(Duration::from_millis(80)).with_counts(4, 3, 1, 0),
        ));

        let text = std::fs::read_to_string(sink.path()).unwrap();

        assert!(text.contains(
            "yunheshiguang_operations_total{operation=\"run_batch_action\",batch_action=\"screenshot\",outcome=\"failed\"} 1"
        ));
        assert!(text.contains(
            "yunheshiguang_operation_duration_seconds_count{operation=\"run_batch_action\",batch_action=\"screenshot\"} 1"
        ));
        assert!(text.contains("yunheshiguang_devices{state=\"online\"} 3"));
        assert!(text.contains("yunheshiguang_probe_duration_seconds_count 1"));
        assert!(text.trim_end().ends_with("# EOF"));
        assert_eq!(
            std::fs::read_dir(folder.path().join("nested"))
                .unwrap()
                .count(),
            1
        );
    }

    #[test]
    fn names_operations_without_the_contract_prefixes() {
        assert_eq!(
            short_name(GovernedAction::PairDevice.proto_name(), "GOVERNED_ACTION_"),
            "pair_device"
        );
        assert_eq!(
            short_name(
                BatchActionKind::Unspecified.proto_name(),
                "BATCH_ACTION_KIND_"
            ),
            "none"
        );
    }
}
