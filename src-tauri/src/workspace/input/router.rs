use std::collections::HashSet;

use yunheshiguang_sdk::device::DeviceShell;
use yunheshiguang_sdk::input::{InputChannel, InputRequest};

use crate::workspace::contract::{InputChannels, UiText};
use crate::workspace::messages::INPUT_NO_CHANNEL_KEY;
use crate::workspace::text::ui_text;

#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub id: String,
    pub label: Option<UiText>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Delivery {
    pub route: Option<Route>,
    pub result: Result<(), UiText>,
}

pub fn plain_message(text: &UiText) -> String {
    text.params
        .get("message")
        .cloned()
        .or_else(|| (!text.fallback.is_empty()).then(|| text.fallback.clone()))
        .unwrap_or_else(|| text.key.clone())
}

pub struct InputRouter {
    channels: Vec<Box<dyn InputChannel>>,
}

impl InputRouter {
    pub fn new(contributed: Vec<Box<dyn InputChannel>>, core: Vec<Box<dyn InputChannel>>) -> Self {
        let mut seen = HashSet::new();
        let channels = contributed
            .into_iter()
            .chain(core)
            .filter(|channel| {
                let id = channel.describe().id;
                let first = seen.insert(id.clone());

                if !first {
                    log::warn!("input channel {id} is installed twice; keeping the first");
                }
                first
            })
            .collect();

        Self { channels }
    }

    pub fn channels(&self) -> InputChannels {
        InputChannels {
            channels: self
                .channels
                .iter()
                .map(|channel| channel.describe())
                .collect(),
            ..Default::default()
        }
    }

    pub fn deliver(&self, device: &dyn DeviceShell, request: &InputRequest<'_>) -> Delivery {
        for channel in &self.channels {
            if channel.accepts(device, request) {
                let info = channel.describe();

                return Delivery {
                    route: Some(Route {
                        id: info.id,
                        label: info.label.into(),
                    }),
                    result: channel.deliver(device, request),
                };
            }
        }

        Delivery {
            route: None,
            result: Err(ui_text(INPUT_NO_CHANNEL_KEY)),
        }
    }
}
