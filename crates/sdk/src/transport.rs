use crate::contract::yunheshiguang::connection::v1::DiscoveredEndpoint;
use crate::contract::yunheshiguang::workspace::v1::{ConnectionPlan, UiText};

pub trait TransportAdapter: Send + Sync {
    fn plan(&self) -> ConnectionPlan;

    fn discover(&self) -> Result<Vec<DiscoveredEndpoint>, UiText> {
        Ok(Vec::new())
    }
}
