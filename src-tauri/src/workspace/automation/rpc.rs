use std::sync::Arc;

use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
use yunheshiguang_contract::yunheshiguang::automation::v1::{
    AutomationService as AutomationApi, FlowRun, FlowRunResult, FlowRuns, Flows, GetFlowRunRequest,
    ListFlowRunsRequest, ListFlowsRequest, ListSchedulesRequest, RemoveScheduleRequest, RunTrigger,
    SaveScheduleRequest, ScheduleResult, Schedules, StartFlowRunRequest, StopFlowRunRequest,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::{Caller, principal, refusal};

use super::AutomationService;

pub struct AutomationRpc {
    pub service: Arc<AutomationService>,
    pub governance: Arc<Governance>,
}

impl AutomationApi for AutomationRpc {
    async fn list_flows(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListFlowsRequest>,
    ) -> ServiceResult<Flows> {
        Response::ok(
            self.service
                .flows()
                .await
                .map_err(|problem| refusal(ErrorCode::Unavailable, problem))?,
        )
    }

    async fn start_flow_run(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, StartFlowRunRequest>,
    ) -> ServiceResult<FlowRunResult> {
        let caller = Caller::of(&ctx)?;
        let trigger = if caller.window().is_some() {
            RunTrigger::Manual
        } else {
            RunTrigger::Api
        };
        let request = request.to_owned_message();

        Response::ok(
            self.service
                .start(request, trigger, &caller.principal(&self.governance))
                .await,
        )
    }

    async fn stop_flow_run(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, StopFlowRunRequest>,
    ) -> ServiceResult<FlowRunResult> {
        let principal = principal(&self.governance, &ctx)?;

        Response::ok(
            self.service
                .stop(&request.to_owned_message().run_id, &principal),
        )
    }

    async fn list_flow_runs(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListFlowRunsRequest>,
    ) -> ServiceResult<FlowRuns> {
        Response::ok(self.service.runs())
    }

    async fn get_flow_run(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, GetFlowRunRequest>,
    ) -> ServiceResult<FlowRun> {
        let run_id = request.to_owned_message().run_id;

        Response::ok(
            self.service
                .run(&run_id)
                .ok_or_else(|| ConnectError::new(ErrorCode::NotFound, "no run has this id"))?,
        )
    }

    async fn list_schedules(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListSchedulesRequest>,
    ) -> ServiceResult<Schedules> {
        Response::ok(self.service.schedules())
    }

    async fn save_schedule(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, SaveScheduleRequest>,
    ) -> ServiceResult<ScheduleResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.save_schedule(request, &principal).await)
    }

    async fn remove_schedule(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, RemoveScheduleRequest>,
    ) -> ServiceResult<ScheduleResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(self.service.remove_schedule(request, &principal).await)
    }
}
