use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, Manager, WebviewWindowBuilder};
use tauri_plugin_reference_extension::ReferenceExtension;
use yunheshiguang_sdk::Extension;

use super::collect_contributions;

fn invoke_from(
    app: &App<MockRuntime>,
    label: &str,
    command: &str,
) -> Result<InvokeResponseBody, String> {
    let webview = match app.get_webview_window(label) {
        Some(webview) => webview,
        None => WebviewWindowBuilder::new(app, label, Default::default())
            .build()
            .unwrap(),
    };

    get_ipc_response(
        &webview,
        InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map_err(|error| error.to_string())
}

fn describe_from(app: &App<MockRuntime>, label: &str) -> Result<serde_json::Value, String> {
    invoke_from(app, label, "plugin:reference-extension|describe")
        .map(|body| body.deserialize().unwrap())
}

#[test]
fn grants_the_app_commands_to_workspace_windows_only() {
    let app = mock_builder()
        .build(tauri::generate_context!(test = true))
        .unwrap();

    for command in ["rpc", "rpc_cancel", "reveal_log_directory"] {
        let refused = invoke_from(&app, "intruder", command).unwrap_err();
        assert!(refused.contains("not allowed"), "{refused}");

        for label in ["main", "workspace_2"] {
            let admitted = invoke_from(&app, label, command).unwrap_err();
            assert!(!admitted.contains("not allowed"), "{admitted}");
        }
    }
}

#[test]
fn installs_the_reference_extension_and_grants_its_commands_to_workspace_windows_only() {
    let extensions: Vec<Box<dyn Extension<MockRuntime>>> = vec![Box::new(ReferenceExtension)];
    let mut builder = mock_builder();

    for extension in &extensions {
        for plugin in extension.plugins() {
            builder = builder.plugin(plugin);
        }
    }

    let app = builder
        .build(tauri::generate_context!(test = true))
        .unwrap();
    let parts = collect_contributions(app.handle(), &extensions).unwrap();

    assert!(parts.authorization.is_some());
    assert!(parts.entitlements.is_some());
    assert!(parts.documents.is_some());
    assert!(parts.audit_store.is_some());
    assert_eq!(parts.template_sources.len(), 1);
    assert_eq!(parts.telemetry_sinks.len(), 1);
    assert_eq!(parts.vendor_adapters.len(), 1);
    assert_eq!(parts.input_channels.len(), 1);
    assert_eq!(parts.script_engines.len(), 1);
    assert_eq!(parts.api_authenticators.len(), 1);

    let status = describe_from(&app, "main").unwrap();
    assert!(status["auditFolder"].as_str().unwrap().ends_with("audit"));
    assert!(
        status["policyFile"]
            .as_str()
            .unwrap()
            .ends_with("policy.cedar")
    );
    assert!(
        status["metricsFile"]
            .as_str()
            .unwrap()
            .ends_with("metrics.prom")
    );
    assert!(describe_from(&app, "workspace_2").is_ok());

    let refused = describe_from(&app, "intruder").unwrap_err();
    assert!(refused.contains("not allowed"), "{refused}");
}
