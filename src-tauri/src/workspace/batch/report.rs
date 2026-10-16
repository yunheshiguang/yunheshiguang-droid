use std::collections::BTreeMap;
use std::time::Duration;
use yunheshiguang_contract::Enumeration;
use yunheshiguang_contract::KnownEnum;

use crate::workspace::contract::*;

pub(super) fn log_outcome(slug: &str, result: &BatchActionResult, elapsed: Duration) {
    let elapsed_ms = elapsed.as_millis();

    if !result.accepted {
        log::warn!(
            "batch {slug}: blocked after {elapsed_ms} ms ({})",
            diagnostic_codes(&result.diagnostics)
        );
        return;
    }

    let total = result.device_results.len();
    let failures = failure_kinds(&result.device_results);
    let failed: usize = failures.values().sum();

    if failed == 0 {
        log::info!("batch {slug}: {total}/{total} device(s) succeeded in {elapsed_ms} ms");
    } else {
        log::warn!(
            "batch {slug}: {}/{total} device(s) succeeded in {elapsed_ms} ms, failures {failures:?}",
            total - failed
        );
    }
}

fn failure_kinds(results: &[AdbDeviceActionResult]) -> BTreeMap<&'static str, usize> {
    let mut kinds = BTreeMap::new();

    for result in results.iter().filter(|result| !result.success) {
        let kind = AdbErrorKindContract::known(result.error_kind)
            .map(|kind| kind.proto_name())
            .unwrap_or("UNKNOWN");
        *kinds.entry(kind).or_insert(0) += 1;
    }

    kinds
}

fn diagnostic_codes(diagnostics: &[DiagnosticSignal]) -> String {
    let codes: Vec<&str> = diagnostics
        .iter()
        .map(|signal| {
            DiagnosticCode::known(signal.code)
                .map(|code| code.proto_name())
                .unwrap_or("UNKNOWN")
        })
        .collect();

    codes.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(success: bool, kind: AdbErrorKindContract) -> AdbDeviceActionResult {
        AdbDeviceActionResult {
            device_id: "R58M20ABCDE".into(),
            success,
            error_kind: kind.into(),
            message: String::new(),
            ..Default::default()
        }
    }

    #[test]
    fn counts_failures_by_kind_without_device_identifiers() {
        let kinds = failure_kinds(&[
            device(true, AdbErrorKindContract::Unspecified),
            device(false, AdbErrorKindContract::Timeout),
            device(false, AdbErrorKindContract::Timeout),
            device(false, AdbErrorKindContract::PermissionDenied),
        ]);

        let rendered = format!("{kinds:?}");

        assert_eq!(kinds.values().sum::<usize>(), 3);
        assert!(!rendered.contains("R58M20ABCDE"));
        assert_eq!(kinds.len(), 2);
    }
}
