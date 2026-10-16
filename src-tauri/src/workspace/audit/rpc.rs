use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::governance::v1::{
    AuditPage, AuditReport, AuditService, ExportAuditRequest, ExportAuditResult, QueryAuditRequest,
    ReportAuditRequest,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::{failure, principal};

use super::requests::{export_as, query_as, report_as};

pub struct AuditRpc(pub Arc<Governance>);

impl AuditService for AuditRpc {
    async fn query_audit(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, QueryAuditRequest>,
    ) -> ServiceResult<AuditPage> {
        let principal = principal(&self.0, &ctx)?;
        let query = request
            .to_owned_message()
            .query
            .into_option()
            .unwrap_or_default();

        Response::ok(
            query_as(&self.0, query, &principal)
                .await
                .map_err(failure)?,
        )
    }

    async fn report_audit(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, ReportAuditRequest>,
    ) -> ServiceResult<AuditReport> {
        let principal = principal(&self.0, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(
            report_as(&self.0, request, &principal)
                .await
                .map_err(failure)?,
        )
    }

    async fn export_audit(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, ExportAuditRequest>,
    ) -> ServiceResult<ExportAuditResult> {
        let principal = principal(&self.0, &ctx)?;
        let request = request.to_owned_message();

        Response::ok(export_as(&self.0, request, &principal).await)
    }
}
