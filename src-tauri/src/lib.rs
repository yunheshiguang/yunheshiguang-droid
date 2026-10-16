#![allow(refining_impl_trait_internal, refining_impl_trait_reachable)]

mod distribution;
mod logging;
mod window_lifecycle;
mod workspace;

pub use distribution::Distribution;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let distribution = Distribution::open_source();

    #[cfg(feature = "reference-extension")]
    let distribution =
        distribution.with_extension(tauri_plugin_reference_extension::ReferenceExtension);

    distribution.run(tauri::generate_context!());
}

fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    workspace_invoke_handler!()
}
