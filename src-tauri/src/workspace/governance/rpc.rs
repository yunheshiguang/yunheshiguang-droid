use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::governance::v1::{
    GetGovernanceStateRequest, GovernanceService, GovernanceState,
};

use crate::workspace::rpc::principal;

use super::Governance;

pub struct GovernanceRpc(pub Arc<Governance>);

impl GovernanceService for GovernanceRpc {
    async fn get_governance_state(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, GetGovernanceStateRequest>,
    ) -> ServiceResult<GovernanceState> {
        let principal = principal(&self.0, &ctx)?;

        Response::ok(self.0.state_for(&principal))
    }
}
