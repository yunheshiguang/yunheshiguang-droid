use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::api::v1::{
    ApiGatewayResult, ApiGatewayService, ApiGatewayState, CreateApiTokenRequest,
    GetApiGatewayRequest, RevokeApiTokenRequest, UpdateApiGatewayRequest,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::principal;

use super::service::ApiService;

pub struct ApiGatewayRpc {
    pub service: Arc<ApiService>,
    pub governance: Arc<Governance>,
}

impl ApiGatewayService for ApiGatewayRpc {
    async fn get_api_gateway(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetApiGatewayRequest>,
    ) -> ServiceResult<ApiGatewayState> {
        Response::ok(self.service.state())
    }

    async fn update_api_gateway(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, UpdateApiGatewayRequest>,
    ) -> ServiceResult<ApiGatewayResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.update(request, &principal).await)
    }

    async fn create_api_token(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, CreateApiTokenRequest>,
    ) -> ServiceResult<ApiGatewayResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.create_token(request, &principal).await)
    }

    async fn revoke_api_token(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, RevokeApiTokenRequest>,
    ) -> ServiceResult<ApiGatewayResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.revoke_token(request, &principal).await)
    }
}
