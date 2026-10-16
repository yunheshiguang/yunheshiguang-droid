use std::sync::{Mutex, PoisonError};

use tokio::sync::broadcast;

use crate::workspace::adb::DEVICES_CHANGED_EVENT;
use crate::workspace::automation::AUTOMATION_CHANGED_EVENT;
use crate::workspace::cloud::CLOUD_PHONES_CHANGED_EVENT;
use crate::workspace::contract::ApiEvent;
use crate::workspace::documents::DOCUMENTS_CHANGED_EVENT;
use crate::workspace::profiles::DEVICE_PROFILES_CHANGED_EVENT;

pub const FORWARDED_EVENTS: [&str; 5] = [
    DEVICES_CHANGED_EVENT,
    DOCUMENTS_CHANGED_EVENT,
    DEVICE_PROFILES_CHANGED_EVENT,
    AUTOMATION_CHANGED_EVENT,
    CLOUD_PHONES_CHANGED_EVENT,
];

const BACKLOG: usize = 256;

#[derive(Default)]
pub struct EventRelay {
    sender: Mutex<Option<broadcast::Sender<ApiEvent>>>,
}

impl EventRelay {
    fn sender(&self) -> std::sync::MutexGuard<'_, Option<broadcast::Sender<ApiEvent>>> {
        self.sender.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn open(&self) {
        *self.sender() = Some(broadcast::channel(BACKLOG).0);
    }

    pub fn close(&self) {
        *self.sender() = None;
    }

    pub fn subscribe(&self) -> Option<broadcast::Receiver<ApiEvent>> {
        self.sender().as_ref().map(broadcast::Sender::subscribe)
    }

    pub fn publish(&self, event: &str, payload_json: &str) {
        if let Some(sender) = self.sender().as_ref() {
            let _ = sender.send(ApiEvent {
                event: event.to_string(),
                payload_json: payload_json.to_string(),
                ..Default::default()
            });
        }
    }
}
