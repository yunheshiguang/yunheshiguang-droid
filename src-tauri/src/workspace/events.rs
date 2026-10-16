use std::sync::Arc;

use serde::Serialize;
use tauri::{App, AppHandle, Emitter, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

pub trait WorkspaceEvents: Send + Sync {
    fn emit(&self, event: &'static str, payload: serde_json::Value);
}

pub fn notify(events: &dyn WorkspaceEvents, event: &'static str, payload: &impl Serialize) {
    match serde_json::to_value(payload) {
        Ok(value) => events.emit(event, value),
        Err(error) => log::warn!("events: {event} payload failed: {error}"),
    }
}

struct AppEvents<R: Runtime>(AppHandle<R>);

impl<R: Runtime> WorkspaceEvents for AppEvents<R> {
    fn emit(&self, event: &'static str, payload: serde_json::Value) {
        if let Err(error) = self.0.emit(event, payload) {
            log::warn!("events: {event} failed: {error}");
        }
    }
}

pub struct EventsModule;

impl<R: Runtime> WorkspaceModule<R> for EventsModule {
    fn id(&self) -> &'static str {
        "events"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        let events: Arc<dyn WorkspaceEvents> = Arc::new(AppEvents(app.handle().clone()));

        app.manage(events);

        Ok(())
    }
}
