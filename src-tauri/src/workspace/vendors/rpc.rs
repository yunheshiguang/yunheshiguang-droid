use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::vendors::v1::{
    DeviceVendors, IdentifyDevicesRequest, VendorService,
};

use crate::workspace::adb::AdbService;
use crate::workspace::android_tools::{AndroidPlatformToolsRegistry, probe_android_tools};
use crate::workspace::blocking::run_blocking;
use crate::workspace::rpc::failure;

use super::VendorRegistry;

pub struct VendorsRpc {
    pub vendors: Arc<VendorRegistry>,
    pub registry: Arc<AndroidPlatformToolsRegistry>,
    pub adb: Arc<AdbService>,
}

impl VendorService for VendorsRpc {
    async fn identify_devices(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, IdentifyDevicesRequest>,
    ) -> ServiceResult<DeviceVendors> {
        let device_ids = request.to_owned_message().device_ids;
        let vendors = Arc::clone(&self.vendors);
        let registry = Arc::clone(&self.registry);
        let adb = Arc::clone(&self.adb);

        Response::ok(
            run_blocking(move || {
                let _runtime = adb.acquire_runtime();
                Ok(probe_android_tools(&registry.config())
                    .adb_path
                    .map(|adb_path| vendors.vendors(&adb_path, &device_ids))
                    .unwrap_or_default())
            })
            .await
            .map_err(failure)?,
        )
    }
}
