use std::env;
use std::path::{Path, PathBuf};

pub fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

pub fn binary_path(root: &Path, name: &str) -> PathBuf {
    root.join(executable_name(name))
}

fn home() -> Option<PathBuf> {
    env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

pub fn conventional_tool_dirs() -> Vec<PathBuf> {
    let mut dirs = ["ANDROID_HOME", "ANDROID_SDK_ROOT"]
        .into_iter()
        .filter_map(env::var_os)
        .map(|sdk| PathBuf::from(sdk).join("platform-tools"))
        .collect::<Vec<_>>();

    if cfg!(target_os = "macos") {
        dirs.extend(home().map(|home| home.join("Library/Android/sdk/platform-tools")));
        dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    } else if cfg!(windows) {
        dirs.extend(
            env::var_os("LOCALAPPDATA")
                .map(|local| PathBuf::from(local).join("Android\\Sdk\\platform-tools")),
        );
        dirs.extend(
            env::var_os("LOCALAPPDATA")
                .map(|local| PathBuf::from(local).join("Microsoft\\WinGet\\Links")),
        );
    } else {
        dirs.extend(home().map(|home| home.join("Android/Sdk/platform-tools")));
        dirs.extend(["/usr/local/bin", "/usr/bin", "/snap/bin"].map(PathBuf::from));
    }
    dirs
}

pub fn find_tool(name: &str) -> Option<PathBuf> {
    which::which(name).ok().or_else(|| {
        conventional_tool_dirs()
            .into_iter()
            .map(|dir| binary_path(&dir, name))
            .find(|path| is_executable_file(path))
    })
}

pub fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(unix)]
pub fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}
