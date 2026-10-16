use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use tauri::{App, Emitter, Manager, PhysicalPosition, PhysicalSize, Runtime, WindowEvent};
#[cfg(desktop)]
use tauri_plugin_window_state::{StateFlags, WindowExt};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

pub const MAIN_WINDOW_LABEL: &str = "main";

#[cfg(desktop)]
const MAIN_WINDOW_STATE: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED);

#[cfg(desktop)]
pub fn window_state_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_window_state::Builder::new()
        .with_state_flags(MAIN_WINDOW_STATE)
        .with_filter(|label| label == MAIN_WINDOW_LABEL)
        .skip_initial_state(MAIN_WINDOW_LABEL)
        .build()
}

#[cfg(desktop)]
pub fn single_instance_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        let is_visible = |window: &tauri::WebviewWindow<R>| window.is_visible().unwrap_or(false);
        let target = app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .filter(is_visible)
            .or_else(|| app.webview_windows().into_values().find(is_visible));

        if let Some(window) = target {
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    })
}

pub struct WindowLifecycleModule;

impl<R: Runtime> WorkspaceModule<R> for WindowLifecycleModule {
    fn id(&self) -> &'static str {
        "window-lifecycle"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::new(RestoreRegistry::default()));

        Ok(())
    }
}

pub struct RestoreState {
    focus_lost: AtomicBool,
    reopen_pending: AtomicBool,
}

impl RestoreState {
    fn new() -> Self {
        Self {
            focus_lost: AtomicBool::new(false),
            reopen_pending: AtomicBool::new(false),
        }
    }
}

#[derive(Default)]
pub struct RestoreRegistry {
    windows: Mutex<HashMap<String, Arc<RestoreState>>>,
}

impl RestoreRegistry {
    fn state_for(&self, label: &str) -> Arc<RestoreState> {
        let mut windows = self.windows.lock().unwrap_or_else(|err| err.into_inner());

        windows
            .entry(label.to_string())
            .or_insert_with(|| Arc::new(RestoreState::new()))
            .clone()
    }

    fn remove(&self, label: &str) {
        let mut windows = self.windows.lock().unwrap_or_else(|err| err.into_inner());
        windows.remove(label);
    }
}

pub fn prepare_window(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    size_to_work_area(window)?;

    #[cfg(desktop)]
    window.restore_state(MAIN_WINDOW_STATE)?;

    Ok(())
}

fn size_to_work_area(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor
        && monitor.work_area().size.width > 0
        && monitor.work_area().size.height > 0
    {
        let work_area = *monitor.work_area();
        let work_width = i64::from(work_area.size.width);
        let work_height = i64::from(work_area.size.height);
        let width = (work_width as f64 * 0.7)
            .round()
            .clamp(1.0, work_width as f64) as u32;
        let height = (work_height as f64 * 0.7)
            .round()
            .clamp(1.0, work_height as f64) as u32;
        let x = work_area.position.x + ((work_width - i64::from(width)) / 2) as i32;
        let y = work_area.position.y + ((work_height - i64::from(height)) / 2) as i32;

        window.set_size(PhysicalSize::new(width, height))?;
        window.set_position(PhysicalPosition::new(x, y))?;
    } else {
        window.set_size(PhysicalSize::new(896, 560))?;
        window.center()?;
    }

    Ok(())
}

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    let Some(registry) = window.try_state::<Arc<RestoreRegistry>>() else {
        return;
    };

    let registry = registry.inner().clone();
    let window_label = window.label().to_string();
    let state = registry.state_for(&window_label);

    match event {
        WindowEvent::Focused(false) => {
            state.focus_lost.store(true, Ordering::SeqCst);

            if window.is_minimized().unwrap_or(false) {
                state.reopen_pending.store(true, Ordering::SeqCst);
                return;
            }

            let app_handle = window.app_handle().clone();
            let state = Arc::clone(&state);
            let registry = Arc::clone(&registry);
            let window_label = window_label.clone();

            tauri::async_runtime::spawn_blocking(move || {
                std::thread::sleep(Duration::from_millis(140));

                if !state.focus_lost.load(Ordering::SeqCst) {
                    return;
                }

                if let Some(window) = app_handle.get_webview_window(&window_label) {
                    if window.is_minimized().unwrap_or(false) {
                        state.reopen_pending.store(true, Ordering::SeqCst);
                    }
                } else {
                    registry.remove(&window_label);
                }
            });
        }
        WindowEvent::Focused(true) => {
            state.focus_lost.store(false, Ordering::SeqCst);

            if state.reopen_pending.swap(false, Ordering::SeqCst) {
                let _ = window.emit_to(window_label.as_str(), "workspace:restore-loading", ());
            }
        }
        WindowEvent::Destroyed => {
            registry.remove(&window_label);
        }
        _ => {}
    }
}
