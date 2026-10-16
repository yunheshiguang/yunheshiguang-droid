use std::sync::Arc;
use std::time::Instant;
use yunheshiguang_contract::Enumeration;

use yunheshiguang_sdk::connection::ConnectionProvider;
use yunheshiguang_sdk::telemetry::{OperationSample, TelemetryEvent};

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::connection::v1::ConnectionService;

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{
    AuditOutcome, ConnectDeviceRequest, ConnectionOutcome, ConnectionResult,
    DisconnectDeviceRequest, PairDeviceRequest, UiText,
};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::rpc::principal;
use crate::workspace::telemetry::Telemetry;

use super::outcome::{connection_result, failed, invalid_request, outcome_of, succeeded};
use super::request::ConnectionRequest;

pub struct ConnectionRpc {
    pub governance: Arc<Governance>,
    pub provider: Arc<dyn ConnectionProvider>,
    pub telemetry: Arc<Telemetry>,
}

impl ConnectionRpc {
    async fn dispatch(
        &self,
        ctx: &RequestContext,
        request: Result<ConnectionRequest, UiText>,
    ) -> ServiceResult<ConnectionResult> {
        let principal = principal(&self.governance, ctx)?;
        let request = match request {
            Ok(request) => request,
            Err(detail) => return Response::ok(invalid_request(detail)),
        };
        let action = request.action();
        let started = Instant::now();
        let result = govern(
            &self.governance,
            Arc::clone(&self.provider),
            &principal,
            request,
        )
        .await;

        if result.outcome != ConnectionOutcome::Denied {
            let ok = succeeded(&result);

            self.telemetry.record(TelemetryEvent::OperationFinished(
                OperationSample::new(action, ok, started.elapsed()).with_targets(1, u32::from(!ok)),
            ));
        }

        Response::ok(result)
    }
}

impl ConnectionService for ConnectionRpc {
    async fn connect_device(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, ConnectDeviceRequest>,
    ) -> ServiceResult<ConnectionResult> {
        let request = request.to_owned_message();

        self.dispatch(&ctx, ConnectionRequest::connect(&request))
            .await
    }

    async fn pair_device(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, PairDeviceRequest>,
    ) -> ServiceResult<ConnectionResult> {
        let request = request.to_owned_message();

        self.dispatch(&ctx, ConnectionRequest::pair(&request)).await
    }

    async fn disconnect_device(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, DisconnectDeviceRequest>,
    ) -> ServiceResult<ConnectionResult> {
        let request = request.to_owned_message();

        self.dispatch(&ctx, ConnectionRequest::disconnect(&request))
            .await
    }
}

pub(super) async fn govern(
    governance: &Arc<Governance>,
    provider: Arc<dyn ConnectionProvider>,
    principal: &Principal,
    request: ConnectionRequest,
) -> ConnectionResult {
    let action = request.action();
    let ticket = match governance.begin(action, principal, &[request.target()]) {
        Ok(ticket) => ticket,
        Err(reason) => {
            log::info!("connection {}: denied", action.proto_name());
            return connection_result(ConnectionOutcome::Denied, "", reason);
        }
    };

    let result = run_blocking(move || Ok(request.run(provider.as_ref())))
        .await
        .unwrap_or_else(|error| {
            log::error!("connection {}: worker failed: {error}", action.proto_name());
            failed(error)
        });
    let audit = if succeeded(&result) {
        AuditOutcome::Succeeded
    } else {
        AuditOutcome::Failed
    };

    ticket.finish(audit, None);
    log::info!(
        "connection {}: {}",
        action.proto_name(),
        outcome_of(&result).proto_name()
    );
    result
}
