use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, PoisonError, RwLock};

use yunheshiguang_contract::yunheshiguang::documents::v1::{
    DocumentList, DocumentsChanged, ListDocumentsRequest, RemoveDocumentRequest,
    SaveDocumentRequest,
};
use yunheshiguang_sdk::documents::{Collection, DocumentError, DocumentStore};

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{AuditOutcome, GovernedAction, UiText};
use crate::workspace::events::{WorkspaceEvents, notify};
use crate::workspace::governance::{Governance, Principal};
use crate::workspace::messages::*;
use crate::workspace::text::ui_text_with_params;

use super::store;

pub const DOCUMENTS_CHANGED_EVENT: &str = "workspace:documents-changed";

#[derive(Debug)]
pub enum DocumentsError {
    Invalid(String),
    Denied(UiText),
    Failed(String),
}

type ApiWriteCheck = Arc<dyn Fn(&str) -> Option<UiText> + Send + Sync>;

pub struct DocumentsService {
    store: Arc<dyn DocumentStore>,
    governance: Arc<Governance>,
    events: Arc<dyn WorkspaceEvents>,
    reserved: RwLock<BTreeSet<String>>,
    api_checks: RwLock<BTreeMap<String, ApiWriteCheck>>,
}

impl DocumentsService {
    pub fn new(
        store: Arc<dyn DocumentStore>,
        governance: Arc<Governance>,
        events: Arc<dyn WorkspaceEvents>,
    ) -> Self {
        Self {
            store,
            governance,
            events,
            reserved: RwLock::new(BTreeSet::new()),
            api_checks: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn store(&self) -> Arc<dyn DocumentStore> {
        Arc::clone(&self.store)
    }

    pub fn reserve(&self, collection: &str) {
        self.reserved
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(collection.to_string());
    }

    pub fn check_api_writes(
        &self,
        collection: &str,
        check: impl Fn(&str) -> Option<UiText> + Send + Sync + 'static,
    ) {
        self.api_checks
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(collection.to_string(), Arc::new(check));
    }

    fn api_refusal(&self, collection: &str, body: &str) -> Option<UiText> {
        let check = self
            .api_checks
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(collection)
            .cloned()?;
        check(body)
    }

    fn open(&self, collection: &str) -> Result<(), DocumentsError> {
        let collection = collection.trim();
        if self
            .reserved
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(collection)
        {
            Err(DocumentsError::Invalid(format!(
                "the {collection} collection only changes through its own commands"
            )))
        } else {
            Ok(())
        }
    }

    pub async fn list(
        &self,
        request: ListDocumentsRequest,
    ) -> Result<DocumentList, DocumentsError> {
        self.open(&request.collection)?;
        let store = self.store();

        run_blocking(move || Ok(store::list(store.as_ref(), &request)))
            .await
            .map_err(DocumentsError::Failed)?
            .map_err(DocumentsError::Invalid)
    }

    pub async fn save(
        &self,
        request: SaveDocumentRequest,
        principal: &Principal,
    ) -> Result<(), DocumentsError> {
        self.open(&request.collection)?;
        let (collection, document) =
            store::checked_save(&request).map_err(DocumentsError::Invalid)?;
        if principal.is_api()
            && let Some(reason) = self.api_refusal(collection.as_str(), &document.body)
        {
            self.governance
                .begin(GovernedAction::EditDocuments, principal, &[])
                .map_err(DocumentsError::Denied)?
                .finish(AuditOutcome::Denied, Some(reason.clone()));
            return Err(DocumentsError::Denied(reason));
        }
        let id = document.id.as_str().to_string();
        let store = self.store();
        let target = collection.clone();

        self.governed(
            principal,
            DOCUMENTS_AUDIT_SAVED_KEY,
            collection,
            &id,
            move || store.save(&target, &document),
        )
        .await
    }

    pub async fn remove(
        &self,
        request: RemoveDocumentRequest,
        principal: &Principal,
    ) -> Result<(), DocumentsError> {
        self.open(&request.collection)?;
        let (collection, id) = store::checked_remove(&request).map_err(DocumentsError::Invalid)?;
        let store = self.store();
        let (probed, probe) = (collection.clone(), id.clone());
        let present = run_blocking(move || Ok(store.contains(&probed, &probe)))
            .await
            .map_err(DocumentsError::Failed)?
            .map_err(|error| DocumentsError::Failed(error.to_string()))?;
        if !present {
            return Ok(());
        }
        let name = id.as_str().to_string();
        let store = self.store();
        let target = collection.clone();

        self.governed(
            principal,
            DOCUMENTS_AUDIT_REMOVED_KEY,
            collection,
            &name,
            move || store.remove(&target, &id),
        )
        .await
    }

    async fn governed(
        &self,
        principal: &Principal,
        audit_key: &'static str,
        collection: Collection,
        id: &str,
        work: impl FnOnce() -> Result<(), DocumentError> + Send + 'static,
    ) -> Result<(), DocumentsError> {
        let ticket = self
            .governance
            .begin(GovernedAction::EditDocuments, principal, &[])
            .map_err(DocumentsError::Denied)?;
        let outcome = run_blocking(move || Ok(work().map_err(|error| error.to_string())))
            .await
            .and_then(|result| result);

        match outcome {
            Ok(()) => {
                ticket.finish(
                    AuditOutcome::Succeeded,
                    Some(ui_text_with_params(
                        audit_key,
                        "",
                        [
                            ("collection", collection.as_str().to_string()),
                            ("id", id.to_string()),
                        ],
                    )),
                );
                notify(
                    self.events.as_ref(),
                    DOCUMENTS_CHANGED_EVENT,
                    &DocumentsChanged {
                        collection: collection.as_str().to_string(),
                        ..Default::default()
                    },
                );
                Ok(())
            }
            Err(error) => {
                log::warn!("documents: {}/{id}: {error}", collection.as_str());
                ticket.finish(
                    AuditOutcome::Failed,
                    Some(ui_text_with_params(
                        DOCUMENTS_STORAGE_FAILED_KEY,
                        "",
                        [("message", error.clone())],
                    )),
                );
                Err(DocumentsError::Failed(error))
            }
        }
    }
}
