use std::sync::{Arc, Mutex, PoisonError};

use uuid::Uuid;
use yunheshiguang_sdk::cloud::{CloudFuture, CloudPhoneProvider};
use yunheshiguang_sdk::documents::{Collection, Document, DocumentId, DocumentStore};
use yunheshiguang_sdk::governance::provider_info;

use crate::workspace::blocking::run_blocking;
use crate::workspace::contract::{CloudPhone, CloudProviderInfo, NetworkEndpoint, UiText};
use crate::workspace::messages::*;
use crate::workspace::text::{ui_text, ui_text_with_params};

pub const REGISTERED_PROVIDER_ID: &str = "core.registered";
pub const CLOUD_PHONES_COLLECTION: &str = "cloud-phones";

pub fn storage_failed(error: impl ToString) -> UiText {
    ui_text_with_params(
        CLOUD_STORAGE_FAILED_KEY,
        "",
        [("message", error.to_string())],
    )
}

fn collection() -> Collection {
    Collection::parse(CLOUD_PHONES_COLLECTION).expect("collection name is valid")
}

pub struct RegisteredPhones {
    store: Arc<dyn DocumentStore>,
    changing: Mutex<()>,
}

impl RegisteredPhones {
    pub fn new(store: Arc<dyn DocumentStore>) -> Self {
        Self {
            store,
            changing: Mutex::new(()),
        }
    }

    pub fn read(&self) -> Result<Vec<CloudPhone>, UiText> {
        let mut phones = self
            .store
            .list(&collection())
            .map_err(storage_failed)?
            .into_iter()
            .filter_map(|document| {
                yunheshiguang_contract::json::from_str::<CloudPhone>(&document.body)
                    .ok()
                    .map(|phone| CloudPhone {
                        provider_id: REGISTERED_PROVIDER_ID.to_string(),
                        id: document.id.as_str().to_string(),
                        ..phone
                    })
            })
            .collect::<Vec<_>>();
        phones.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        Ok(phones)
    }

    pub fn register(&self, name: &str, endpoint: NetworkEndpoint) -> Result<CloudPhone, UiText> {
        let _changing = self.changing.lock().unwrap_or_else(PoisonError::into_inner);
        if self
            .read()?
            .iter()
            .any(|phone| phone.endpoint.as_option() == Some(&endpoint))
        {
            return Err(ui_text(CLOUD_ALREADY_REGISTERED_KEY));
        }

        let phone = CloudPhone {
            provider_id: REGISTERED_PROVIDER_ID.to_string(),
            id: Uuid::now_v7().to_string(),
            name: name.to_string(),
            endpoint: endpoint.into(),
            ..CloudPhone::default()
        };
        let body = serde_json::to_string(&phone).map_err(storage_failed)?;
        let document = DocumentId::parse(&phone.id)
            .and_then(|id| Document::new(id, body))
            .map_err(storage_failed)?;
        self.store
            .save(&collection(), &document)
            .map_err(storage_failed)?;

        Ok(phone)
    }

    pub fn remove(&self, id: &str) -> Result<CloudPhone, UiText> {
        let _changing = self.changing.lock().unwrap_or_else(PoisonError::into_inner);
        let phone = self
            .read()?
            .into_iter()
            .find(|phone| phone.id == id)
            .ok_or_else(|| ui_text(CLOUD_PHONE_MISSING_KEY))?;
        let id = DocumentId::parse(id).map_err(storage_failed)?;
        self.store
            .remove(&collection(), &id)
            .map_err(storage_failed)?;

        Ok(phone)
    }
}

impl CloudPhoneProvider for RegisteredPhones {
    fn describe(&self) -> CloudProviderInfo {
        let info = provider_info(
            REGISTERED_PROVIDER_ID,
            CLOUD_REGISTERED_LABEL_KEY,
            CLOUD_REGISTERED_DESCRIPTION_KEY,
        );

        CloudProviderInfo {
            id: info.id,
            label: info.label,
            description: info.description,
            can_start: false,
            can_stop: false,
            can_register: true,
            ..Default::default()
        }
    }

    fn phones(&self) -> CloudFuture<'_, Vec<CloudPhone>> {
        let registered = Self::new(Arc::clone(&self.store));

        Box::pin(async move {
            run_blocking(move || Ok(registered.read()))
                .await
                .map_err(storage_failed)?
        })
    }
}
