use std::collections::HashSet;

use yunheshiguang_sdk::device::DeviceShell;
use yunheshiguang_sdk::scripts::{ScriptEngine, ScriptOutcome, ScriptRequest};

use crate::workspace::contract::{ScriptEngines, UiText};
use crate::workspace::messages::SCRIPT_NO_ENGINE_KEY;
use crate::workspace::text::ui_text;

#[derive(Clone, Debug, PartialEq)]
pub struct EngineRoute {
    pub id: String,
    pub label: Option<UiText>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptRun {
    pub engine: Option<EngineRoute>,
    pub result: Result<ScriptOutcome, UiText>,
}

pub struct ScriptRunner {
    engines: Vec<Box<dyn ScriptEngine>>,
}

impl ScriptRunner {
    pub fn new(contributed: Vec<Box<dyn ScriptEngine>>, core: Vec<Box<dyn ScriptEngine>>) -> Self {
        let mut seen = HashSet::new();
        let engines = contributed
            .into_iter()
            .chain(core)
            .filter(|engine| {
                let id = engine.describe().id;
                let first = seen.insert(id.clone());

                if !first {
                    log::warn!("script engine {id} is installed twice; keeping the first");
                }
                first
            })
            .collect();

        Self { engines }
    }

    pub fn engines(&self) -> ScriptEngines {
        ScriptEngines {
            engines: self
                .engines
                .iter()
                .map(|engine| engine.describe())
                .collect(),
            ..Default::default()
        }
    }

    pub fn run(&self, device: &dyn DeviceShell, script: &ScriptRequest<'_>) -> ScriptRun {
        for engine in &self.engines {
            if engine.accepts(device, script) {
                let info = engine.describe();

                return ScriptRun {
                    engine: Some(EngineRoute {
                        id: info.id,
                        label: info.label.into(),
                    }),
                    result: engine.run(device, script),
                };
            }
        }

        ScriptRun {
            engine: None,
            result: Err(ui_text(SCRIPT_NO_ENGINE_KEY)),
        }
    }
}
