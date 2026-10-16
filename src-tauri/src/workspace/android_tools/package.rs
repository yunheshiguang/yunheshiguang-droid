use crate::workspace::contract::AndroidPlatformToolsPackageKind;
use crate::workspace::tool_archive::{ArchiveFormat, PinnedArchive};

const VERSION: &str = "37.0.0";
const WINDOWS_URL: &str = "https://dl.google.com/android/repository/platform-tools_r37.0.0-win.zip";
const MACOS_URL: &str =
    "https://dl.google.com/android/repository/platform-tools_r37.0.0-darwin.zip";
const LINUX_URL: &str = "https://dl.google.com/android/repository/platform-tools_r37.0.0-linux.zip";
const WINDOWS_SHA256: &str = "4fe305812db074cea32903a489d061eb4454cbc90a49e8fea677f4b7af764918";
const MACOS_SHA256: &str = "094a1395683c509fd4d48667da0d8b5ef4d42b2abfcd29f2e8149e2f989357c7";
const LINUX_SHA256: &str = "198ae156ab285fa555987219af237b31102fefe8b9d2bc274708a8d4f2865a07";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AndroidPlatformToolsPackage {
    pub kind: AndroidPlatformToolsPackageKind,
    pub url: &'static str,
    pub version: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

impl AndroidPlatformToolsPackage {
    pub fn from_kind(kind: AndroidPlatformToolsPackageKind) -> Option<Self> {
        match kind {
            AndroidPlatformToolsPackageKind::Windows => Some(Self {
                kind,
                url: WINDOWS_URL,
                version: VERSION,
                sha256: WINDOWS_SHA256,
                size: 8_092_164,
            }),
            AndroidPlatformToolsPackageKind::Macos => Some(Self {
                kind,
                url: MACOS_URL,
                version: VERSION,
                sha256: MACOS_SHA256,
                size: 16_442_240,
            }),
            AndroidPlatformToolsPackageKind::Linux => Some(Self {
                kind,
                url: LINUX_URL,
                version: VERSION,
                sha256: LINUX_SHA256,
                size: 9_167_924,
            }),
            AndroidPlatformToolsPackageKind::Unspecified => None,
        }
    }

    pub fn archive(self) -> PinnedArchive {
        PinnedArchive {
            url: self.url,
            size: self.size,
            sha256: self.sha256,
            format: ArchiveFormat::Zip,
        }
    }

    pub fn current_host() -> Option<Self> {
        package_for_host(
            std::env::consts::OS,
            std::env::consts::ARCH,
            cfg!(target_env = "gnu"),
        )
    }

    pub fn supports_current_host(self) -> bool {
        Self::current_host()
            .map(|current| current.kind == self.kind)
            .unwrap_or(false)
    }
}

fn package_for_host(
    os: &str,
    arch: &str,
    uses_gnu_libc: bool,
) -> Option<AndroidPlatformToolsPackage> {
    let kind = match (os, arch, uses_gnu_libc) {
        ("windows", "x86_64", _) => AndroidPlatformToolsPackageKind::Windows,
        ("macos", "x86_64" | "aarch64", _) => AndroidPlatformToolsPackageKind::Macos,
        ("linux", "x86_64", true) => AndroidPlatformToolsPackageKind::Linux,
        _ => return None,
    };
    AndroidPlatformToolsPackage::from_kind(kind)
}

pub fn package_kind_slug(kind: AndroidPlatformToolsPackageKind) -> &'static str {
    match kind {
        AndroidPlatformToolsPackageKind::Windows => "windows",
        AndroidPlatformToolsPackageKind::Macos => "macos",
        AndroidPlatformToolsPackageKind::Linux => "linux",
        AndroidPlatformToolsPackageKind::Unspecified => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_packages() {
        let windows =
            AndroidPlatformToolsPackage::from_kind(AndroidPlatformToolsPackageKind::Windows)
                .expect("Windows package should exist");
        let macos = AndroidPlatformToolsPackage::from_kind(AndroidPlatformToolsPackageKind::Macos)
            .expect("macOS package should exist");
        let linux = AndroidPlatformToolsPackage::from_kind(AndroidPlatformToolsPackageKind::Linux)
            .expect("Linux package should exist");
        assert_eq!(windows.size, 8_092_164);
        assert_eq!(macos.size, 16_442_240);
        assert_eq!(linux.size, 9_167_924);
        assert_eq!(windows.sha256, WINDOWS_SHA256);
        assert_eq!(macos.sha256, MACOS_SHA256);
        assert_eq!(linux.sha256, LINUX_SHA256);
        assert!(
            AndroidPlatformToolsPackage::from_kind(AndroidPlatformToolsPackageKind::Unspecified)
                .is_none()
        );
        assert_eq!(
            package_kind_slug(AndroidPlatformToolsPackageKind::Windows),
            "windows"
        );
    }

    #[test]
    fn selects_only_compatible_official_host_packages() {
        assert_eq!(
            package_for_host("windows", "x86_64", false).map(|package| package.kind),
            Some(AndroidPlatformToolsPackageKind::Windows)
        );
        assert!(package_for_host("windows", "aarch64", false).is_none());
        assert_eq!(
            package_for_host("macos", "x86_64", false).map(|package| package.kind),
            Some(AndroidPlatformToolsPackageKind::Macos)
        );
        assert_eq!(
            package_for_host("macos", "aarch64", false).map(|package| package.kind),
            Some(AndroidPlatformToolsPackageKind::Macos)
        );
        assert_eq!(
            package_for_host("linux", "x86_64", true).map(|package| package.kind),
            Some(AndroidPlatformToolsPackageKind::Linux)
        );
        assert!(package_for_host("linux", "x86_64", false).is_none());
        assert!(package_for_host("linux", "aarch64", true).is_none());
    }
}
