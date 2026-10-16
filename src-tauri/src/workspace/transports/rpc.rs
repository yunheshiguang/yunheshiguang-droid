use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::connection::v1::{
    DeviceDiscovery, DiscoverDevicesRequest, DiscoveryService,
};

use crate::workspace::blocking::run_blocking;
use crate::workspace::rpc::failure;

use super::TransportRegistry;

pub struct DiscoveryRpc(pub Arc<TransportRegistry>);

impl DiscoveryService for DiscoveryRpc {
    async fn discover_devices(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, DiscoverDevicesRequest>,
    ) -> ServiceResult<DeviceDiscovery> {
        let transports = Arc::clone(&self.0);

        Response::ok(
            run_blocking(move || Ok(transports.discover()))
                .await
                .map_err(failure)?,
        )
    }
}
