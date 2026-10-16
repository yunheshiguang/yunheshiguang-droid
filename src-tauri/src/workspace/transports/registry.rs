use std::collections::HashSet;

use yunheshiguang_sdk::transport::TransportAdapter;

use crate::workspace::contract::{ConnectionPlan, DeviceDiscovery, DiscoveryFailure};

pub struct TransportRegistry {
    transports: Vec<(String, Box<dyn TransportAdapter>)>,
}

impl TransportRegistry {
    pub fn new(
        core: Vec<Box<dyn TransportAdapter>>,
        contributed: Vec<Box<dyn TransportAdapter>>,
    ) -> Self {
        let mut seen = HashSet::new();
        let transports = core
            .into_iter()
            .chain(contributed)
            .filter_map(|transport| {
                let id = transport.plan().id;

                if seen.insert(id.clone()) {
                    Some((id, transport))
                } else {
                    log::warn!("transport {id} is installed twice; keeping the first");
                    None
                }
            })
            .collect();

        Self { transports }
    }

    pub fn plans(&self) -> Vec<ConnectionPlan> {
        self.transports
            .iter()
            .map(|(_, transport)| transport.plan())
            .collect()
    }

    pub fn discover(&self) -> DeviceDiscovery {
        let mut discovery = DeviceDiscovery::default();

        for (id, transport) in &self.transports {
            match transport.discover() {
                Ok(endpoints) => {
                    discovery
                        .endpoints
                        .extend(endpoints.into_iter().map(|mut endpoint| {
                            endpoint.transport_id = id.clone();
                            endpoint
                        }))
                }
                Err(reason) => {
                    log::info!("transport {id}: discovery failed");
                    discovery.failures.push(DiscoveryFailure {
                        transport_id: id.clone(),
                        reason: reason.into(),
                        ..Default::default()
                    });
                }
            }
        }

        discovery
    }
}
