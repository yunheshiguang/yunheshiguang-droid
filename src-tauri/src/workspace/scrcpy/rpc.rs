use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::scrcpy::v1::{
    GetScrcpyStateRequest, InstallScrcpyRequest, LaunchScrcpySessionRequest,
    ScrcpyService as ScrcpyApi, ScrcpyState, StopScrcpySessionRequest, UpdateScrcpyConfigRequest,
};

use crate::workspace::blocking::run_blocking;
use crate::workspace::governance::Governance;
use crate::workspace::rpc::{failure, principal};

use super::ScrcpyService;
use super::args::default_config;

pub struct ScrcpyRpc {
    pub service: Arc<ScrcpyService>,
    pub governance: Arc<Governance>,
}

impl ScrcpyApi for ScrcpyRpc {
    async fn get_scrcpy_state(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, GetScrcpyStateRequest>,
    ) -> ServiceResult<ScrcpyState> {
        let service = Arc::clone(&self.service);

        Response::ok(
            run_blocking(move || Ok(service.state(None)))
                .await
                .map_err(failure)?,
        )
    }

    async fn update_scrcpy_config(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, UpdateScrcpyConfigRequest>,
    ) -> ServiceResult<ScrcpyState> {
        let principal = principal(&self.governance, &ctx)?;
        let config = request
            .to_owned_message()
            .config
            .into_option()
            .unwrap_or_else(default_config);
        let service = Arc::clone(&self.service);

        Response::ok(
            run_blocking(move || Ok(service.update_config(config, &principal)))
                .await
                .map_err(failure)?,
        )
    }

    async fn install_scrcpy(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, InstallScrcpyRequest>,
    ) -> ServiceResult<ScrcpyState> {
        let principal = principal(&self.governance, &ctx)?;
        let package_kind = request.to_owned_message().package_kind.to_i32();
        let service = Arc::clone(&self.service);

        Response::ok(
            run_blocking(move || Ok(service.install(package_kind, &principal)))
                .await
                .map_err(failure)?,
        )
    }

    async fn launch_scrcpy_session(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, LaunchScrcpySessionRequest>,
    ) -> ServiceResult<ScrcpyState> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        let service = Arc::clone(&self.service);

        Response::ok(
            run_blocking(move || {
                let owner = principal.actor.clone();
                Ok(service.launch(
                    &request.serial,
                    request.config.into_option(),
                    &principal,
                    &owner,
                ))
            })
            .await
            .map_err(failure)?,
        )
    }

    async fn stop_scrcpy_session(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, StopScrcpySessionRequest>,
    ) -> ServiceResult<ScrcpyState> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        let service = Arc::clone(&self.service);

        Response::ok(
            run_blocking(move || {
                Ok(service.stop(&request.session_id, &request.serial, &principal))
            })
            .await
            .map_err(failure)?,
        )
    }
}
