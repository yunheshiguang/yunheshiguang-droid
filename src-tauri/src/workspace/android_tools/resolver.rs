use std::path::{Path, PathBuf};

use crate::workspace::contract::{
    AndroidPlatformToolsBinary, AndroidPlatformToolsBinaryKind, AndroidPlatformToolsSource,
    ConnectionHealth, DiagnosticCode, DiagnosticSignal, UiText,
};
use crate::workspace::diagnostics::signal;
use crate::workspace::messages::{
    ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY, ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
    ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY, ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
};
use crate::workspace::text::ui_text_with_params;
use crate::workspace::tooling::find_tool;

use super::binary::{binary_path, executable_name, is_executable_file, path_to_string};
use super::runtime_config::ScopedAndroidPlatformToolsConfig;

#[derive(Clone, Debug)]
pub struct AndroidToolEnvironment {
    pub source: AndroidPlatformToolsSource,
    pub platform_tools_root: PathBuf,
    pub adb_path: Option<PathBuf>,
    pub fastboot_path: Option<PathBuf>,
    pub diagnostics: Vec<DiagnosticSignal>,
}

impl AndroidToolEnvironment {
    pub fn health(&self) -> ConnectionHealth {
        if self.diagnostics.is_empty() {
            ConnectionHealth::Ready
        } else if self.adb_path.is_some() || self.fastboot_path.is_some() {
            ConnectionHealth::Degraded
        } else {
            ConnectionHealth::Blocked
        }
    }

    pub fn binaries(&self) -> Vec<AndroidPlatformToolsBinary> {
        [
            (
                AndroidPlatformToolsBinaryKind::Adb,
                self.adb_path
                    .as_deref()
                    .map(path_to_string)
                    .unwrap_or_default(),
            ),
            (
                AndroidPlatformToolsBinaryKind::Fastboot,
                self.fastboot_path
                    .as_deref()
                    .map(path_to_string)
                    .unwrap_or_default(),
            ),
        ]
        .into_iter()
        .map(|(kind, resolved_path)| AndroidPlatformToolsBinary {
            kind: kind.into(),
            available: !resolved_path.is_empty(),
            resolved_path,
            ..Default::default()
        })
        .collect()
    }
}

pub fn adb_missing_text() -> UiText {
    ui_text_with_params(
        ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
        "",
        [("binary", executable_name("adb"))],
    )
}

fn diagnostic(
    id: impl Into<String>,
    code: DiagnosticCode,
    message_key: &'static str,
    params: impl IntoIterator<Item = (&'static str, String)>,
) -> DiagnosticSignal {
    signal(id, code, ui_text_with_params(message_key, "", params))
}

fn resolve_from_system_path(binary: &str) -> Option<PathBuf> {
    find_tool(binary)
}

fn resolve_from_manual_directory(root: &Path, binary: &str) -> Option<PathBuf> {
    let path = binary_path(root, binary);

    if is_executable_file(&path) {
        Some(path)
    } else {
        None
    }
}

fn resolve_from_adb_sibling(adb_path: &Path, binary: &str) -> Option<PathBuf> {
    let parent = adb_path.parent()?;
    resolve_from_manual_directory(parent, binary)
}

fn invalid_manual_directory(root: &Path) -> DiagnosticSignal {
    diagnostic(
        "android-platform-tools-invalid-directory",
        DiagnosticCode::AndroidPlatformToolsInvalidDirectory,
        ANDROID_PLATFORM_TOOLS_INVALID_DIRECTORY_KEY,
        [("path", path_to_string(root))],
    )
}

fn missing_binary(binary: &str, source: AndroidPlatformToolsSource) -> DiagnosticSignal {
    let (id, code, key) = match binary {
        "fastboot" => (
            "android-platform-tools-fastboot-missing",
            DiagnosticCode::AndroidPlatformToolsFastbootMissing,
            ANDROID_PLATFORM_TOOLS_FASTBOOT_MISSING_KEY,
        ),
        _ => (
            "android-platform-tools-adb-missing",
            DiagnosticCode::AndroidPlatformToolsAdbMissing,
            ANDROID_PLATFORM_TOOLS_ADB_MISSING_KEY,
        ),
    };

    diagnostic(
        id,
        code,
        key,
        [
            ("binary", executable_name(binary)),
            ("source", format!("{source:?}")),
        ],
    )
}

fn unavailable() -> DiagnosticSignal {
    diagnostic(
        "android-platform-tools-unavailable",
        DiagnosticCode::AndroidPlatformToolsUnavailable,
        ANDROID_PLATFORM_TOOLS_UNAVAILABLE_KEY,
        std::iter::empty::<(&'static str, String)>(),
    )
}

pub fn probe_android_tools(config: &ScopedAndroidPlatformToolsConfig) -> AndroidToolEnvironment {
    let source = match config.source {
        AndroidPlatformToolsSource::ManualDirectory => AndroidPlatformToolsSource::ManualDirectory,
        AndroidPlatformToolsSource::AppPrivate => AndroidPlatformToolsSource::AppPrivate,
        _ => AndroidPlatformToolsSource::SystemPath,
    };
    let mut diagnostics = Vec::new();

    let (adb_path, fastboot_path, platform_tools_root) = match source {
        AndroidPlatformToolsSource::ManualDirectory | AndroidPlatformToolsSource::AppPrivate => {
            let root = &config.platform_tools_root;

            if root.as_os_str().is_empty() || !root.is_dir() {
                diagnostics.push(invalid_manual_directory(root));
                (None, None, config.platform_tools_root.clone())
            } else {
                (
                    resolve_from_manual_directory(root, "adb"),
                    resolve_from_manual_directory(root, "fastboot"),
                    config.platform_tools_root.clone(),
                )
            }
        }
        _ => {
            let adb_path = resolve_from_system_path("adb");
            let fastboot_path = adb_path
                .as_deref()
                .and_then(|path| resolve_from_adb_sibling(path, "fastboot"))
                .or_else(|| resolve_from_system_path("fastboot"));
            (adb_path, fastboot_path, PathBuf::new())
        }
    };

    if adb_path.is_none() {
        diagnostics.push(missing_binary("adb", source));
    }

    if fastboot_path.is_none() {
        diagnostics.push(missing_binary("fastboot", source));
    }

    if adb_path.is_none() && fastboot_path.is_none() {
        diagnostics.push(unavailable());
    }

    AndroidToolEnvironment {
        source,
        platform_tools_root,
        adb_path,
        fastboot_path,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_invalid_manual_directory() {
        let config = ScopedAndroidPlatformToolsConfig {
            source: AndroidPlatformToolsSource::ManualDirectory,
            platform_tools_root: PathBuf::new(),
        };

        let env = probe_android_tools(&config);

        assert!(env.adb_path.is_none());
        assert!(env.fastboot_path.is_none());
        assert!(
            env.diagnostics
                .iter()
                .any(|item| { item.code == DiagnosticCode::AndroidPlatformToolsInvalidDirectory })
        );
    }
}
