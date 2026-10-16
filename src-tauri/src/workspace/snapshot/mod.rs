mod draft;
mod probe;
mod rpc;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use tauri::{App, Manager, Runtime};
use yunheshiguang_sdk::telemetry::{ProbeSample, TelemetryEvent};

use crate::workspace::adb::{AdbSection, AdbService, DeviceIdentities};
use crate::workspace::android_tools::{AndroidPlatformToolsRegistry, AndroidToolsSection};
use crate::workspace::blocking::run_blocking;
use crate::workspace::cloud::{CloudSection, CloudService};
use crate::workspace::contract::{AdbDeviceStateContract, CloudPhone, WorkspaceSnapshot};
use crate::workspace::module::{ModuleSetupError, WorkspaceModule, dependency};
use crate::workspace::profiles::{Inventory, ProfileBook, ProfilesSection, connected_device_keys};
use crate::workspace::telemetry::Telemetry;
use crate::workspace::transports::{TransportRegistry, TransportsSection};

pub use draft::{SnapshotDraft, SnapshotSection};
pub use probe::HostProbe;

const SCHEMA_VERSION: &str = "2026.06.p0";

fn host_platform() -> String {
    std::env::consts::OS.to_string()
}

fn locale() -> String {
    crate::workspace::i18n::preferred_locale().to_string()
}

struct SnapshotSources {
    registry: Arc<AndroidPlatformToolsRegistry>,
    adb_service: Arc<AdbService>,
    transports: Arc<TransportRegistry>,
    identities: Arc<DeviceIdentities>,
    profiles: Arc<ProfileBook>,
    cloud: Arc<CloudService>,
}

pub struct SnapshotService {
    sources: SnapshotSources,
    telemetry: Arc<Telemetry>,
}

impl SnapshotService {
    pub fn build(&self) -> WorkspaceSnapshot {
        let started = Instant::now();
        let snapshot = build_snapshot(&self.sources);
        let elapsed = started.elapsed();
        log::debug!(
            "snapshot: {} device(s) in {} ms",
            snapshot.devices.len(),
            elapsed.as_millis()
        );
        self.telemetry
            .record(TelemetryEvent::DevicesProbed(probe_sample(
                &snapshot, elapsed,
            )));
        snapshot
    }

    pub async fn snapshot(self: &Arc<Self>) -> Result<WorkspaceSnapshot, String> {
        let service = Arc::clone(self);

        run_blocking(move || Ok(service.build())).await
    }
}

pub struct SnapshotModule;

impl<R: Runtime> WorkspaceModule<R> for SnapshotModule {
    fn id(&self) -> &'static str {
        "snapshot"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let service = SnapshotService {
            sources: SnapshotSources {
                registry: dependency(app)?,
                adb_service: dependency(app)?,
                transports: dependency(app)?,
                identities: dependency(app)?,
                profiles: dependency(app)?,
                cloud: dependency(app)?,
            },
            telemetry: dependency(app)?,
        };

        app.manage(Arc::new(service));

        let rpc = rpc::SnapshotRpc(dependency(app)?);
        crate::workspace::rpc::serve(app, |router| router.add_service(Arc::new(rpc)))?;

        Ok(())
    }
}

fn core_sections<'a>(
    probe: &'a HostProbe,
    sources: &'a SnapshotSources,
    inventory: &'a Inventory,
    keys: &'a HashMap<String, String>,
    cloud_phones: &'a [CloudPhone],
) -> Vec<Box<dyn SnapshotSection + 'a>> {
    vec![
        Box::new(AndroidToolsSection::new(
            &probe.tools,
            &sources.registry,
            probe.adb_path(),
            probe.fastboot_path(),
        )),
        Box::new(AdbSection::new(&probe.adb, probe.adb_path())),
        Box::new(ProfilesSection::new(inventory, keys)),
        Box::new(CloudSection::new(cloud_phones)),
        Box::new(TransportsSection::new(&sources.transports)),
    ]
}

fn build_snapshot(sources: &SnapshotSources) -> WorkspaceSnapshot {
    let _runtime = sources.adb_service.acquire_runtime();
    let probe = HostProbe::collect(&sources.registry, &sources.adb_service);
    let keys = connected_device_keys(
        &probe.adb.devices,
        probe.tools.adb_path.as_deref(),
        &sources.identities,
    );
    let inventory = sources.profiles.inventory();
    let cloud_phones = sources.cloud.cached();
    let mut draft = SnapshotDraft::new(SCHEMA_VERSION, locale(), host_platform());
    let mut contributed = HashSet::new();

    for section in core_sections(&probe, sources, &inventory, &keys, &cloud_phones) {
        if contributed.insert(section.id()) {
            section.contribute(&mut draft);
        }
    }

    draft.finish()
}

fn probe_sample(snapshot: &WorkspaceSnapshot, elapsed: std::time::Duration) -> ProbeSample {
    let devices = snapshot
        .adb
        .as_option()
        .map(|adb| adb.devices.as_slice())
        .unwrap_or_default();
    let count = |states: &[AdbDeviceStateContract]| {
        u32::try_from(
            devices
                .iter()
                .filter(|device| states.iter().any(|state| device.state == *state))
                .count(),
        )
        .unwrap_or(u32::MAX)
    };

    ProbeSample::new(elapsed).with_counts(
        u32::try_from(devices.len()).unwrap_or(u32::MAX),
        count(&[AdbDeviceStateContract::Online]),
        count(&[
            AdbDeviceStateContract::Unauthorized,
            AdbDeviceStateContract::NoPermissions,
        ]),
        count(&[AdbDeviceStateContract::Offline]),
    )
}
