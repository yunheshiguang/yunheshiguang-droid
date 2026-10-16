use std::path::Path;

use super::manifest::{Asset, Manifest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallTarget {
    pub os: &'static str,
    pub arch: &'static str,
    pub kind: Option<&'static str>,
}

impl InstallTarget {
    pub fn current() -> Self {
        let arch = match std::env::consts::ARCH {
            "aarch64" => "arm64",
            other => other,
        };
        match std::env::consts::OS {
            "windows" => Self::new("windows", arch, Some("installer")),
            "macos" => Self::new("macos", arch, Some("disk-image")),
            _ => Self::new(
                "linux",
                arch,
                linux_kind(LinuxInstall {
                    appimage: std::env::var_os("APPIMAGE").is_some(),
                    executable: std::env::current_exe().ok().as_deref(),
                    dpkg: Path::new("/var/lib/dpkg/status").exists(),
                    rpm: Path::new("/var/lib/rpm").exists()
                        || Path::new("/usr/lib/sysimage/rpm").exists(),
                }),
            ),
        }
    }

    pub fn new(os: &'static str, arch: &'static str, kind: Option<&'static str>) -> Self {
        Self { os, arch, kind }
    }

    pub fn choose<'a>(&self, manifest: &'a Manifest) -> Option<&'a Asset> {
        let kind = self.kind?;
        let matching = |arch: &str| {
            manifest
                .assets
                .iter()
                .find(|asset| asset.os == self.os && asset.arch == arch && asset.kind == kind)
        };

        matching(self.arch).or_else(|| matching("universal"))
    }
}

pub struct LinuxInstall<'a> {
    pub appimage: bool,
    pub executable: Option<&'a Path>,
    pub dpkg: bool,
    pub rpm: bool,
}

pub fn linux_kind(install: LinuxInstall<'_>) -> Option<&'static str> {
    if install.appimage {
        return Some("appimage");
    }
    let packaged = install
        .executable
        .is_some_and(|path| path.starts_with("/usr/bin") || path.starts_with("/usr/lib"));
    if !packaged {
        return None;
    }

    if install.dpkg {
        Some("deb")
    } else if install.rpm {
        Some("rpm")
    } else {
        None
    }
}
