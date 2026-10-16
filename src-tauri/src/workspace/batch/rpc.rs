use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::workspace::v1::{
    BatchActionRequest, BatchActionResult, BatchService as BatchApi,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::principal;

use super::BatchService;

pub struct BatchRpc {
    pub service: Arc<BatchService>,
    pub governance: Arc<Governance>,
}

impl BatchApi for BatchRpc {
    async fn run_batch_action(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, BatchActionRequest>,
    ) -> ServiceResult<BatchActionResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.run(request, &principal).await)
    }
}
