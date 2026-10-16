use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::updates::v1::{
    CheckForUpdatesRequest, GetUpdateStateRequest, SetAutomaticChecksRequest, UpdateService,
    UpdateSettingsResult, UpdateState,
};

use crate::workspace::rpc::Caller;

use super::checker::Updates;

pub struct UpdatesRpc(pub Arc<Updates>);

impl UpdateService for UpdatesRpc {
    async fn get_update_state(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetUpdateStateRequest>,
    ) -> ServiceResult<UpdateState> {
        Response::ok(self.0.state())
    }

    async fn check_for_updates(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, CheckForUpdatesRequest>,
    ) -> ServiceResult<UpdateState> {
        Caller::of(&ctx)?;
        Response::ok(self.0.check(false).await)
    }

    async fn set_automatic_checks(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, SetAutomaticChecksRequest>,
    ) -> ServiceResult<UpdateSettingsResult> {
        Caller::of(&ctx)?;
        let check_automatically = request.to_owned_message().check_automatically;
        Response::ok(self.0.set_automatic(check_automatically))
    }
}
