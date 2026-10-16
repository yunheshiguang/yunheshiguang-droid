use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use yunheshiguang_contract::yunheshiguang::templates::v1::{
    FlowTemplateCatalog, ListFlowTemplatesRequest, TemplateService,
};

use crate::workspace::governance::Governance;

use super::TemplateCatalog;

pub struct TemplatesRpc {
    pub catalog: Arc<TemplateCatalog>,
    pub governance: Arc<Governance>,
}

impl TemplateService for TemplatesRpc {
    async fn list_flow_templates(
        &self,
        _ctx: RequestContext,
        _request: ServiceRequest<'_, ListFlowTemplatesRequest>,
    ) -> ServiceResult<FlowTemplateCatalog> {
        Response::ok(
            self.catalog
                .catalog(&self.governance.granted_capabilities()),
        )
    }
}
