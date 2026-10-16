use std::sync::Arc;

use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
use yunheshiguang_contract::yunheshiguang::packages::v1::{
    ExportPackageRequest, ExportPackageResult, InspectPackageRequest, InspectPackageResult,
    InstallPackageRequest, InstallPackageResult, ListMarketPackagesRequest, MarketPackages,
    PackageService, TrustPackageSignerRequest, TrustPackageSignerResult,
};

use crate::workspace::contract::{PackageOrigin, package_origin};
use crate::workspace::governance::Governance;
use crate::workspace::messages::API_BAD_REQUEST_KEY;
use crate::workspace::rpc::{Caller, refusal};
use crate::workspace::text::ui_text_with_params;

use super::PackagesService;

fn local_files_refused(
    caller: &Caller,
    origin: Option<&PackageOrigin>,
) -> Result<(), ConnectError> {
    let from_disk = matches!(
        origin.and_then(|origin| origin.origin.as_ref()),
        Some(package_origin::Origin::Path(_))
    );
    if from_disk && caller.window().is_none() {
        return Err(refusal(
            ErrorCode::InvalidArgument,
            ui_text_with_params(
                API_BAD_REQUEST_KEY,
                "",
                [(
                    "message",
                    "send the package itself as `origin.json`, or name a market package",
                )],
            ),
        ));
    }

    Ok(())
}

pub struct PackagesRpc {
    pub service: Arc<PackagesService>,
    pub governance: Arc<Governance>,
}

impl PackageService for PackagesRpc {
    async fn export_package(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, ExportPackageRequest>,
    ) -> ServiceResult<ExportPackageResult> {
        let principal = Caller::of(&ctx)?.principal(&self.governance);
        let request = request.to_owned_message();

        Response::ok(self.service.export(request, &principal).await)
    }

    async fn inspect_package(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, InspectPackageRequest>,
    ) -> ServiceResult<InspectPackageResult> {
        let request = request.to_owned_message();
        local_files_refused(&Caller::of(&ctx)?, request.origin.as_option())?;

        Response::ok(self.service.inspect(request).await)
    }

    async fn install_package(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, InstallPackageRequest>,
    ) -> ServiceResult<InstallPackageResult> {
        let caller = Caller::of(&ctx)?;
        let request = request.to_owned_message();
        local_files_refused(&caller, request.origin.as_option())?;

        Response::ok(
            self.service
                .install(request, &caller.principal(&self.governance))
                .await,
        )
    }

    async fn trust_package_signer(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, TrustPackageSignerRequest>,
    ) -> ServiceResult<TrustPackageSignerResult> {
        let principal = Caller::of(&ctx)?.principal(&self.governance);
        let request = request.to_owned_message();

        Response::ok(self.service.trust_signer(request, &principal).await)
    }

    async fn list_market_packages(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListMarketPackagesRequest>,
    ) -> ServiceResult<MarketPackages> {
        Response::ok(self.service.market())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_package_files_on_this_computer_to_api_callers() {
        let origin = |origin| PackageOrigin {
            origin: Some(origin),
            ..Default::default()
        };
        let api = Caller::Api(yunheshiguang_sdk::api::ApiIdentity::new("ci"));
        let window = Caller::Window("main".to_string());
        let path = origin(package_origin::Origin::Path("/etc/passwd".into()));
        let json = origin(package_origin::Origin::Json("{}".into()));

        assert!(local_files_refused(&api, Some(&path)).is_err());
        assert!(local_files_refused(&api, Some(&json)).is_ok());
        assert!(local_files_refused(&api, None).is_ok());
        assert!(local_files_refused(&window, Some(&path)).is_ok());
    }
}
