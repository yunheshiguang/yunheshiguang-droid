use crate::workspace::adb::{AdbEnvironment, AdbService};
use crate::workspace::android_tools::{
    AndroidPlatformToolsRegistry, AndroidToolEnvironment, probe_android_tools,
};
use crate::workspace::tooling::path_to_string;

pub struct HostProbe {
    pub tools: AndroidToolEnvironment,
    pub adb: AdbEnvironment,
}

impl HostProbe {
    pub fn collect(registry: &AndroidPlatformToolsRegistry, adb_service: &AdbService) -> Self {
        let config = registry.config();
        let mut tools = probe_android_tools(&config);
        registry.append_install_diagnostics(&mut tools.diagnostics);
        let adb = adb_service.probe_tools_locked(&tools);

        Self { tools, adb }
    }

    pub fn adb_path(&self) -> String {
        self.tools
            .adb_path
            .as_deref()
            .map(path_to_string)
            .unwrap_or_default()
    }

    pub fn fastboot_path(&self) -> String {
        self.tools
            .fastboot_path
            .as_deref()
            .map(path_to_string)
            .unwrap_or_default()
    }
}
