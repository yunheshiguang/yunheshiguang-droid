use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::profiles::v1::{
    OrganizeResult, ProfileService, RemoveDeviceGroupRequest, SaveDeviceGroupRequest,
    UpdateDeviceProfilesRequest,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::principal;

use super::ProfilesService;

pub struct ProfilesRpc {
    pub service: Arc<ProfilesService>,
    pub governance: Arc<Governance>,
}

impl ProfileService for ProfilesRpc {
    async fn update_device_profiles(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, UpdateDeviceProfilesRequest>,
    ) -> ServiceResult<OrganizeResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.update_profiles(request, &principal).await)
    }

    async fn save_device_group(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, SaveDeviceGroupRequest>,
    ) -> ServiceResult<OrganizeResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.save_group(request, &principal).await)
    }

    async fn remove_device_group(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, RemoveDeviceGroupRequest>,
    ) -> ServiceResult<OrganizeResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.remove_group(request, &principal).await)
    }
}
