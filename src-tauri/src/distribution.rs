use std::error::Error;
use std::sync::Arc;

use tauri::{AppHandle, Context, Manager, RunEvent, Runtime, Wry};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use yunheshiguang_sdk::{ContributionParts, Contributions, Extension};

use crate::logging;
use crate::window_lifecycle::{self, MAIN_WINDOW_LABEL, handle_window_event, prepare_window};
use crate::workspace::governance::Governance;
use crate::workspace::i18n::{disclaimer, preferred_locale};
use crate::workspace::lifecycle::Shutdown;
use crate::workspace::module::{core_modules, setup_modules};

pub struct Distribution {
    extensions: Vec<Box<dyn Extension<Wry>>>,
}

impl Distribution {
    pub fn open_source() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_extension(mut self, extension: impl Extension<Wry>) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    pub fn run(self, context: Context<Wry>) {
        let builder = tauri::Builder::default();

        #[cfg(desktop)]
        let builder = builder.plugin(window_lifecycle::single_instance_plugin());

        let builder = builder.plugin(logging::plugin());

        #[cfg(desktop)]
        let builder = builder.plugin(window_lifecycle::window_state_plugin());

        let mut builder = builder
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_opener::init());

        for extension in &self.extensions {
            for plugin in extension.plugins() {
                builder = builder.plugin(plugin);
            }
        }

        let extensions = self.extensions;

        builder
            .invoke_handler(crate::invoke_handler())
            .setup(move |app| {
                let package = app.package_info();
                log::info!("starting {} {}", package.name, package.version);

                let parts = collect_contributions(app.handle(), &extensions)
                    .inspect_err(|error| log::error!("extension setup failed: {error}"))?;

                setup_modules(app, &core_modules(parts))
                    .inspect_err(|error| log::error!("workspace module setup failed: {error}"))?;

                let window = app.get_webview_window(MAIN_WINDOW_LABEL).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "main window not found")
                })?;

                prepare_window(&window)?;
                show_disclaimer(window);
                Ok(())
            })
            .on_window_event(handle_window_event)
            .build(context)
            .expect("error while building tauri application")
            .run(|app, event| {
                if let RunEvent::Exit = event {
                    if let Some(shutdown) = app.try_state::<Arc<Shutdown>>() {
                        shutdown.run();
                    }
                    if let Some(governance) = app.try_state::<Arc<Governance>>() {
                        governance.flush_audit();
                    }
                }
            });
    }
}

pub(crate) fn collect_contributions<R: Runtime>(
    app: &AppHandle<R>,
    extensions: &[Box<dyn Extension<R>>],
) -> Result<ContributionParts, Box<dyn Error>> {
    let mut contributions = Contributions::default();

    for extension in extensions {
        extension
            .contribute(app, &mut contributions)
            .map_err(|error| format!("extension {}: {error}", extension.id()))?;
        log::info!("extension {} installed", extension.id());
    }

    Ok(contributions.into_parts())
}

fn show_disclaimer(window: tauri::WebviewWindow) {
    let focus_window = window.clone();
    let copy = disclaimer(preferred_locale());

    window
        .dialog()
        .message(&copy.body)
        .title(&copy.title)
        .kind(MessageDialogKind::Warning)
        .show(move |_| {
            let _ = focus_window.show();
            let _ = focus_window.set_focus();
        });
}

#[cfg(test)]
mod tests {
    use tauri::test::{MockRuntime, mock_app};
    use tauri::{AppHandle, Runtime};
    use yunheshiguang_sdk::contract::yunheshiguang::governance::v1::GovernanceProvider;
    use yunheshiguang_sdk::governance::{ActionContext, AuthorizationProvider, Decision};
    use yunheshiguang_sdk::{Contributions, Extension, ExtensionError};

    use super::collect_contributions;

    struct DenyAll;

    impl AuthorizationProvider for DenyAll {
        fn describe(&self) -> GovernanceProvider {
            GovernanceProvider {
                id: "test.deny-all".to_string(),
                ..GovernanceProvider::default()
            }
        }

        fn authorize(&self, _context: &ActionContext<'_>) -> Decision {
            Decision::Allow
        }
    }

    #[test]
    fn offers_extensions_the_windows_the_core_capability_covers() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();

        assert_eq!(
            capability["windows"],
            serde_json::json!(yunheshiguang_sdk::WORKSPACE_WINDOWS)
        );
        assert!(
            yunheshiguang_sdk::WORKSPACE_WINDOWS
                .contains(&crate::window_lifecycle::MAIN_WINDOW_LABEL)
        );
    }

    struct Policy(&'static str);

    impl<R: Runtime> Extension<R> for Policy {
        fn id(&self) -> &'static str {
            self.0
        }

        fn contribute(
            &self,
            _app: &AppHandle<R>,
            contributions: &mut Contributions,
        ) -> Result<(), ExtensionError> {
            contributions.set_authorization(DenyAll)?;
            Ok(())
        }
    }

    #[test]
    fn collects_what_each_extension_contributes() {
        let app = mock_app();
        let extensions: Vec<Box<dyn Extension<MockRuntime>>> = vec![Box::new(Policy("policy"))];

        let parts = collect_contributions(app.handle(), &extensions).unwrap();

        assert_eq!(parts.authorization.unwrap().describe().id, "test.deny-all");
    }

    #[test]
    fn names_the_extension_that_claims_a_contract_twice() {
        let app = mock_app();
        let extensions: Vec<Box<dyn Extension<MockRuntime>>> =
            vec![Box::new(Policy("first")), Box::new(Policy("second"))];

        let Err(error) = collect_contributions(app.handle(), &extensions) else {
            panic!("two authorization providers must not both install");
        };

        let message = error.to_string();
        assert!(message.contains("extension second"), "{message}");
        assert!(message.contains("Authorization"), "{message}");
    }
}

#[cfg(all(test, feature = "reference-extension"))]
mod composed_tests;
