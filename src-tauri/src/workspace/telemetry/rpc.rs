use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::telemetry::v1::{
    GetWorkspaceHealthRequest, TelemetryService, WorkspaceHealth,
};

use super::Telemetry;

pub struct TelemetryRpc(pub Arc<Telemetry>);

impl TelemetryService for TelemetryRpc {
    async fn get_workspace_health(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetWorkspaceHealthRequest>,
    ) -> ServiceResult<WorkspaceHealth> {
        Response::ok(self.0.health())
    }
}
