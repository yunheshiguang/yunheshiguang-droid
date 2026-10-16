use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use yunheshiguang_sdk::fs::write_atomic;

pub fn load_json<T: DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    match fs::read(path) {
        Ok(body) => yunheshiguang_contract::json::lenient(|| serde_json::from_slice(&body))
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    load_json(path).unwrap_or_else(|error| {
        log::warn!("local state {}: {error}", path.display());
        set_aside(path);
        None
    })
}

fn set_aside(path: &Path) {
    let Some(name) = path.file_name() else {
        return;
    };
    let mut aside = name.to_os_string();
    aside.push(".unreadable");

    if let Err(error) = fs::rename(path, path.with_file_name(aside)) {
        log::warn!(
            "local state {}: could not move the unreadable file aside: {error}",
            path.display()
        );
    }
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    write_atomic(path, &serde_json::to_vec_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_whole_files_and_reads_them_back() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("nested").join("state.json");

        assert_eq!(read_json::<Vec<u32>>(&path), None);
        write_json(&path, &vec![1_u32, 2]).expect("state should save");
        assert_eq!(read_json::<Vec<u32>>(&path), Some(vec![1, 2]));
    }

    #[test]
    fn moves_an_unreadable_file_aside_instead_of_letting_the_next_save_replace_it() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("state.json");
        fs::write(&path, "not json").expect("file should be writable");

        assert!(load_json::<Vec<u32>>(&path).is_err());
        assert_eq!(read_json::<Vec<u32>>(&path), None);
        assert!(!path.exists());
        assert_eq!(
            fs::read_to_string(directory.path().join("state.json.unreadable")).unwrap(),
            "not json"
        );
    }

    #[cfg(unix)]
    #[test]
    fn moves_a_file_it_may_not_read_aside_as_well() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("state.json");
        fs::write(&path, "[1]").expect("file should be writable");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&path).is_ok() {
            return;
        }

        assert_eq!(read_json::<Vec<u32>>(&path), None);
        assert!(!path.exists());
        assert!(directory.path().join("state.json.unreadable").exists());
    }
}
