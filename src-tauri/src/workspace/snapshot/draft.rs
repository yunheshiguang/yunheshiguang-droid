use crate::workspace::contract::{
    AdbRuntimeState, AndroidPlatformToolsState, ConnectionPlan, DeviceGroup, DeviceRecord,
    DiagnosticSignal, WorkspaceSnapshot,
};

pub struct SnapshotDraft {
    snapshot: WorkspaceSnapshot,
}

impl SnapshotDraft {
    pub fn new(schema_version: &str, locale: String, host_platform: String) -> Self {
        Self {
            snapshot: WorkspaceSnapshot {
                schema_version: schema_version.to_string(),
                locale,
                host_platform,
                ..WorkspaceSnapshot::default()
            },
        }
    }

    pub fn add_devices(&mut self, devices: Vec<DeviceRecord>) {
        self.snapshot.devices.extend(devices);
    }

    pub fn devices(&self) -> &[DeviceRecord] {
        &self.snapshot.devices
    }

    pub fn annotate_devices(&mut self, annotate: impl FnMut(&mut DeviceRecord)) {
        self.snapshot.devices.iter_mut().for_each(annotate);
    }

    pub fn add_groups(&mut self, groups: Vec<DeviceGroup>) {
        self.snapshot.groups.extend(groups);
    }

    pub fn add_diagnostics(&mut self, diagnostics: impl IntoIterator<Item = DiagnosticSignal>) {
        self.snapshot.diagnostics.extend(diagnostics);
    }

    pub fn add_connection_plans(&mut self, plans: Vec<ConnectionPlan>) {
        self.snapshot.connection_plans.extend(plans);
    }

    pub fn set_tool_paths(&mut self, adb_path: String, fastboot_path: String) {
        self.snapshot.adb_path = adb_path;
        self.snapshot.fastboot_path = fastboot_path;
    }

    pub fn set_android_platform_tools(&mut self, state: AndroidPlatformToolsState) {
        self.snapshot.android_platform_tools = state.into();
    }

    pub fn set_adb_runtime(&mut self, state: AdbRuntimeState) {
        self.snapshot.adb = state.into();
    }

    pub fn finish(self) -> WorkspaceSnapshot {
        self.snapshot
    }
}

pub trait SnapshotSection {
    fn id(&self) -> &'static str;

    fn contribute(&self, draft: &mut SnapshotDraft);
}
