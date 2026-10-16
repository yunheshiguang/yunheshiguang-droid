use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::cloud::v1::{
    CloudPhoneRequest, CloudPhoneResult, CloudPhones, CloudService as CloudApi,
    ListCloudPhonesRequest, RegisterCloudPhoneRequest,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::principal;

use super::CloudService;

pub struct CloudRpc {
    pub service: Arc<CloudService>,
    pub governance: Arc<Governance>,
}

impl CloudApi for CloudRpc {
    async fn list_cloud_phones(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListCloudPhonesRequest>,
    ) -> ServiceResult<CloudPhones> {
        Response::ok(self.service.list().await)
    }

    async fn start_cloud_phone(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, CloudPhoneRequest>,
    ) -> ServiceResult<CloudPhoneResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.start(request, &principal).await)
    }

    async fn stop_cloud_phone(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, CloudPhoneRequest>,
    ) -> ServiceResult<CloudPhoneResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.stop(request, &principal).await)
    }

    async fn register_cloud_phone(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, RegisterCloudPhoneRequest>,
    ) -> ServiceResult<CloudPhoneResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.register(request, &principal).await)
    }

    async fn remove_cloud_phone(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, CloudPhoneRequest>,
    ) -> ServiceResult<CloudPhoneResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.remove(request, &principal).await)
    }
}
