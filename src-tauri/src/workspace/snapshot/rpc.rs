use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::workspace::v1::{
    GetWorkspaceSnapshotRequest, WorkspaceService, WorkspaceSnapshot,
};

use crate::workspace::rpc::failure;

use super::SnapshotService;

pub struct SnapshotRpc(pub Arc<SnapshotService>);

impl WorkspaceService for SnapshotRpc {
    async fn get_workspace_snapshot(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetWorkspaceSnapshotRequest>,
    ) -> ServiceResult<WorkspaceSnapshot> {
        Response::ok(self.0.snapshot().await.map_err(failure)?)
    }
}
