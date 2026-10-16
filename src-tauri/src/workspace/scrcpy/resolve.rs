use std::path::{Path, PathBuf};
use yunheshiguang_contract::KnownEnum;

use crate::workspace::contract::{
    ScrcpyBinary, ScrcpyBinaryKind, ScrcpyBinarySource, ScrcpyConfig, ScrcpyDiagnosticCode,
};
use crate::workspace::tooling::{binary_path, find_tool, is_executable_file, path_to_string};

use super::diagnostics::ScrcpyProblem;

pub const CLIENT: &str = "scrcpy";
pub const SERVER: &str = "scrcpy-server";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resolved {
    pub client: Option<PathBuf>,
    pub server: Option<PathBuf>,
}

pub fn release_in(root: &Path) -> Option<Resolved> {
    let client = binary_path(root, CLIENT);
    let server = root.join(SERVER);

    (is_executable_file(&client) && server.is_file()).then_some(Resolved {
        client: Some(client),
        server: Some(server),
    })
}

pub fn resolve(
    config: &ScrcpyConfig,
    managed_root: Option<&Path>,
) -> Result<Resolved, ScrcpyProblem> {
    let missing = || ScrcpyProblem::new(ScrcpyDiagnosticCode::BinaryMissing, "");

    match ScrcpyBinarySource::known(config.source).unwrap_or_default() {
        ScrcpyBinarySource::ManualDirectory => {
            let root = Path::new(config.scrcpy_root.trim());
            if config.scrcpy_root.trim().is_empty() || !root.is_dir() {
                return Err(ScrcpyProblem::new(
                    ScrcpyDiagnosticCode::InvalidDirectory,
                    "",
                ));
            }
            let client = binary_path(root, CLIENT);
            if !is_executable_file(&client) {
                return Err(missing());
            }
            let server = Some(root.join(SERVER)).filter(|server| server.is_file());

            Ok(Resolved {
                client: Some(client),
                server,
            })
        }
        ScrcpyBinarySource::AppPrivate => managed_root.and_then(release_in).ok_or_else(missing),
        ScrcpyBinarySource::SystemPath | ScrcpyBinarySource::Unspecified => find_tool(CLIENT)
            .map(|client| Resolved {
                client: Some(client),
                server: None,
            })
            .ok_or_else(missing),
    }
}

pub fn binaries(resolved: &Resolved, adb: Option<&Path>) -> Vec<ScrcpyBinary> {
    let entry = |kind: ScrcpyBinaryKind, path: Option<&Path>| ScrcpyBinary {
        kind: kind.into(),
        resolved_path: path.map(path_to_string).unwrap_or_default(),
        available: path.is_some(),
        ..Default::default()
    };

    vec![
        entry(ScrcpyBinaryKind::Client, resolved.client.as_deref()),
        entry(ScrcpyBinaryKind::Server, resolved.server.as_deref()),
        entry(ScrcpyBinaryKind::Adb, adb),
    ]
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn executable(path: &Path) {
        fs::write(path, "#!/bin/sh\n").unwrap();
        crate::workspace::tool_archive::ensure_executable(path).unwrap();
    }

    #[test]
    fn finds_a_release_only_when_client_and_server_are_both_there() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(release_in(folder.path()), None);

        executable(&binary_path(folder.path(), CLIENT));
        assert_eq!(release_in(folder.path()), None);

        fs::write(folder.path().join(SERVER), b"jar").unwrap();
        let found = release_in(folder.path()).unwrap();
        assert_eq!(found.server, Some(folder.path().join(SERVER)));
    }

    #[test]
    fn explains_a_manual_directory_that_is_missing_or_empty() {
        let folder = tempfile::tempdir().unwrap();
        let manual = |root: &Path| ScrcpyConfig {
            source: ScrcpyBinarySource::ManualDirectory.into(),
            scrcpy_root: root.to_string_lossy().into_owned(),
            ..ScrcpyConfig::default()
        };

        assert_eq!(
            resolve(&manual(&folder.path().join("nope")), None)
                .unwrap_err()
                .code,
            ScrcpyDiagnosticCode::InvalidDirectory
        );
        assert_eq!(
            resolve(&manual(folder.path()), None).unwrap_err().code,
            ScrcpyDiagnosticCode::BinaryMissing
        );

        executable(&binary_path(folder.path(), CLIENT));
        let found = resolve(&manual(folder.path()), None).unwrap();
        assert!(found.client.is_some());
        assert_eq!(found.server, None);
    }

    #[test]
    fn needs_an_install_before_the_managed_source_works() {
        let config = ScrcpyConfig {
            source: ScrcpyBinarySource::AppPrivate.into(),
            ..ScrcpyConfig::default()
        };

        assert_eq!(
            resolve(&config, None).unwrap_err().code,
            ScrcpyDiagnosticCode::BinaryMissing
        );
    }
}
