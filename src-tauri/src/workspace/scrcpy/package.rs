use crate::workspace::contract::ScrcpyPackageKind;
use crate::workspace::tool_archive::{ArchiveFormat, PinnedArchive};

pub const RELEASE: &str = "v5.0";

macro_rules! release_file {
    ($file:literal) => {
        concat!(
            "https://github.com/Genymobile/scrcpy/releases/download/v5.0/",
            $file
        )
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrcpyPackage {
    pub kind: ScrcpyPackageKind,
    pub archive: PinnedArchive,
}

const fn zip(url: &'static str, size: u64, sha256: &'static str) -> PinnedArchive {
    PinnedArchive {
        url,
        size,
        sha256,
        format: ArchiveFormat::Zip,
    }
}

const fn tar_gz(url: &'static str, size: u64, sha256: &'static str) -> PinnedArchive {
    PinnedArchive {
        url,
        size,
        sha256,
        format: ArchiveFormat::TarGz,
    }
}

impl ScrcpyPackage {
    pub fn from_kind(kind: ScrcpyPackageKind) -> Option<Self> {
        let archive = match kind {
            ScrcpyPackageKind::WindowsX86 => zip(
                release_file!("scrcpy-win32-v5.0.zip"),
                10_133_772,
                "31413dd59a80cb76e7747ef91b4994934b546c8860e8e7976046c612a50e5cc3",
            ),
            ScrcpyPackageKind::WindowsX64 => zip(
                release_file!("scrcpy-win64-v5.0.zip"),
                11_264_659,
                "44c10d9e82f20ea67227d14d37bf9fbe3603117c5736df3f514544a02ba20a73",
            ),
            ScrcpyPackageKind::WindowsArm64 => zip(
                release_file!("scrcpy-winarm64-v5.0.zip"),
                9_825_783,
                "afc1db17b216f91470d2c8f41ea786ae18ab1cb2bc2324bc05ec3d7685aa6054",
            ),
            ScrcpyPackageKind::MacosAarch64 => tar_gz(
                release_file!("scrcpy-macos-aarch64-v5.0.tar.gz"),
                12_565_892,
                "7cb4e41c859b05b36e89dc9be6c353cc5980c00d7f7f6a763b5b355551b82e9c",
            ),
            ScrcpyPackageKind::MacosX8664 => tar_gz(
                release_file!("scrcpy-macos-x86_64-v5.0.tar.gz"),
                13_479_803,
                "dacb995c8eb42528cb96b2da2a2e3110fe5c82281378e6044fc7cccf48a7c39a",
            ),
            ScrcpyPackageKind::LinuxX8664 => tar_gz(
                release_file!("scrcpy-linux-x86_64-v5.0.tar.gz"),
                17_883_086,
                "f052ad9eb981879e8c5f066c5ef122b39b6c9853b383d6497219030e6549baed",
            ),
            ScrcpyPackageKind::Unspecified => return None,
        };

        Some(Self { kind, archive })
    }

    pub fn current_host() -> Option<Self> {
        kind_for(std::env::consts::OS, std::env::consts::ARCH).and_then(Self::from_kind)
    }
}

fn kind_for(os: &str, arch: &str) -> Option<ScrcpyPackageKind> {
    match (os, arch) {
        ("windows", "x86") => Some(ScrcpyPackageKind::WindowsX86),
        ("windows", "x86_64") => Some(ScrcpyPackageKind::WindowsX64),
        ("windows", "aarch64") => Some(ScrcpyPackageKind::WindowsArm64),
        ("macos", "aarch64") => Some(ScrcpyPackageKind::MacosAarch64),
        ("macos", "x86_64") => Some(ScrcpyPackageKind::MacosX8664),
        ("linux", "x86_64") => Some(ScrcpyPackageKind::LinuxX8664),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yunheshiguang_contract::Enumeration;

    #[test]
    fn pins_one_official_package_per_supported_host() {
        assert_eq!(kind_for("linux", "aarch64"), None);
        assert_eq!(
            kind_for("macos", "aarch64"),
            Some(ScrcpyPackageKind::MacosAarch64)
        );

        for kind in ScrcpyPackageKind::values()
            .iter()
            .copied()
            .filter(|kind| *kind != ScrcpyPackageKind::Unspecified)
        {
            let package = ScrcpyPackage::from_kind(kind).expect("every package kind is pinned");

            assert!(package.archive.url.contains(RELEASE));
            assert_eq!(package.archive.sha256.len(), 64);
        }
    }
}
