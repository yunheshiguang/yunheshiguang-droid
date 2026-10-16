use std::sync::{Arc, Mutex, PoisonError};

use tauri::{App, Manager, Runtime};

use crate::workspace::module::{ModuleSetupError, WorkspaceModule};

type Hook = Box<dyn FnOnce() + Send>;

#[derive(Default)]
pub struct Shutdown {
    hooks: Mutex<Vec<Hook>>,
}

impl Shutdown {
    pub fn on_exit(&self, hook: impl FnOnce() + Send + 'static) {
        self.hooks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Box::new(hook));
    }

    pub fn run(&self) {
        let hooks = std::mem::take(&mut *self.hooks.lock().unwrap_or_else(PoisonError::into_inner));

        for hook in hooks {
            hook();
        }
    }
}

pub struct ShutdownModule;

impl<R: Runtime> WorkspaceModule<R> for ShutdownModule {
    fn id(&self) -> &'static str {
        "shutdown"
    }

    fn setup(&self, app: &mut App<R>) -> Result<(), ModuleSetupError> {
        app.manage(Arc::new(Shutdown::default()));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Shutdown;

    #[test]
    fn runs_every_hook_once_in_the_order_they_were_added() {
        let shutdown = Shutdown::default();
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let calls = Arc::new(AtomicUsize::new(0));

        for name in ["first", "second"] {
            let order = Arc::clone(&order);
            let calls = Arc::clone(&calls);
            shutdown.on_exit(move || {
                order.lock().unwrap().push(name);
                calls.fetch_add(1, Ordering::Relaxed);
            });
        }
        shutdown.run();
        shutdown.run();

        assert_eq!(*order.lock().unwrap(), ["first", "second"]);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
