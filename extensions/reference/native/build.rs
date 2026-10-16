const COMMANDS: &[&str] = &["describe"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
