use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub use atomic_write_file::AtomicWriteFile;

pub fn open_atomic(path: &Path) -> io::Result<AtomicWriteFile> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    let mut options = AtomicWriteFile::options();
    #[cfg(unix)]
    {
        use atomic_write_file::unix::OpenOptionsExt as _;
        use std::os::unix::fs::OpenOptionsExt as _;

        options.mode(0o600).preserve_mode(false);
    }
    options.open(path)
}

pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = open_atomic(path)?;
    file.write_all(contents)?;
    file.commit()
}

#[cfg(test)]
mod tests {
    use super::write_atomic;

    #[test]
    fn replaces_the_whole_file_and_creates_missing_folders() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nested").join("state.json");

        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"second");
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }

    #[cfg(unix)]
    #[test]
    fn keeps_files_readable_by_their_owner_alone() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("credentials.json");
        std::fs::write(&path, b"old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        write_atomic(&path, b"new").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
