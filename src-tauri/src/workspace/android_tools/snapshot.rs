use crate::workspace::contract::{AndroidPlatformToolsConfig, AndroidPlatformToolsState};
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};
use crate::workspace::tooling::path_to_string;

use super::resolver::AndroidToolEnvironment;
use super::runtime_config::AndroidPlatformToolsRegistry;

pub struct AndroidToolsSection<'a> {
    tools: &'a AndroidToolEnvironment,
    registry: &'a AndroidPlatformToolsRegistry,
    adb_path: String,
    fastboot_path: String,
}

impl<'a> AndroidToolsSection<'a> {
    pub fn new(
        tools: &'a AndroidToolEnvironment,
        registry: &'a AndroidPlatformToolsRegistry,
        adb_path: String,
        fastboot_path: String,
    ) -> Self {
        Self {
            tools,
            registry,
            adb_path,
            fastboot_path,
        }
    }
}

impl SnapshotSection for AndroidToolsSection<'_> {
    fn id(&self) -> &'static str {
        "android-platform-tools"
    }

    fn contribute(&self, draft: &mut SnapshotDraft) {
        draft.set_tool_paths(self.adb_path.clone(), self.fastboot_path.clone());
        draft.add_diagnostics(self.tools.diagnostics.iter().cloned());
        draft.set_android_platform_tools(AndroidPlatformToolsState {
            config: AndroidPlatformToolsConfig {
                source: self.tools.source.into(),
                platform_tools_root: path_to_string(&self.tools.platform_tools_root),
                ..Default::default()
            }
            .into(),
            adb_path: self.adb_path.clone(),
            fastboot_path: self.fastboot_path.clone(),
            binaries: self.tools.binaries(),
            diagnostics: self.tools.diagnostics.clone(),
            health: self.tools.health().into(),
            install: self.registry.install_contract().into(),
            last_install_attempt: self
                .registry
                .last_install_attempt_record()
                .into_contract()
                .into(),
            ..Default::default()
        });
    }
}
