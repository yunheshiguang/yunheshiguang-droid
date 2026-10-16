pub use crate::workspace::tooling::{
    binary_path, executable_name, is_executable_file, path_to_string,
};

#[cfg(test)]
mod tests {
    use super::executable_name;

    #[test]
    fn builds_platform_executable_name() {
        let name = executable_name("adb");

        if cfg!(windows) {
            assert_eq!(name, "adb.exe");
        } else {
            assert_eq!(name, "adb");
        }
    }
}
