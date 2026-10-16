use std::sync::Arc;

use connectrpc::{
    ConnectError, ErrorCode, RequestContext, Response, ServiceRequest, ServiceResult,
};
use yunheshiguang_contract::yunheshiguang::documents::v1::{
    DocumentList, DocumentService, ListDocumentsRequest, RemoveDocumentRequest,
    RemoveDocumentResult, SaveDocumentRequest, SaveDocumentResult,
};

use crate::workspace::governance::Governance;
use crate::workspace::rpc::{principal, refusal};

use super::{DocumentsError, DocumentsService};

fn rejected(error: DocumentsError) -> ConnectError {
    match error {
        DocumentsError::Invalid(message) => ConnectError::new(ErrorCode::InvalidArgument, message),
        DocumentsError::Denied(reason) => refusal(ErrorCode::PermissionDenied, reason),
        DocumentsError::Failed(message) => ConnectError::new(ErrorCode::Unavailable, message),
    }
}

pub struct DocumentsRpc {
    pub service: Arc<DocumentsService>,
    pub governance: Arc<Governance>,
}

impl DocumentService for DocumentsRpc {
    async fn list_documents(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, ListDocumentsRequest>,
    ) -> ServiceResult<DocumentList> {
        let request = request.to_owned_message();

        Response::ok(self.service.list(request).await.map_err(rejected)?)
    }

    async fn save_document(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, SaveDocumentRequest>,
    ) -> ServiceResult<SaveDocumentResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        self.service
            .save(request, &principal)
            .await
            .map_err(rejected)?;

        Response::ok(SaveDocumentResult::default())
    }

    async fn remove_document(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, RemoveDocumentRequest>,
    ) -> ServiceResult<RemoveDocumentResult> {
        let principal = principal(&self.governance, &ctx)?;
        let request = request.to_owned_message();
        self.service
            .remove(request, &principal)
            .await
            .map_err(rejected)?;

        Response::ok(RemoveDocumentResult::default())
    }
}
