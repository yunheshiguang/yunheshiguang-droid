use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use yunheshiguang_sdk::documents::{
    Collection, Document, DocumentError, DocumentId, DocumentStore,
};

use crate::workspace::contract::{DeviceGroupDefinition, DeviceProfile};

pub const PROFILES_COLLECTION: &str = "device-profiles";
pub const GROUPS_COLLECTION: &str = "device-groups";
const CACHE_TTL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inventory {
    pub profiles: HashMap<String, DeviceProfile>,
    pub groups: Vec<DeviceGroupDefinition>,
}

impl Inventory {
    pub fn group(&self, id: &str) -> Option<&DeviceGroupDefinition> {
        self.groups.iter().find(|group| group.id == id)
    }

    pub fn profile(&self, key: &str, device_id: &str) -> Option<&DeviceProfile> {
        self.profiles
            .get(key)
            .or_else(|| self.profiles.get(device_id))
    }
}

pub fn is_blank(profile: &DeviceProfile) -> bool {
    profile.number == 0
        && profile.alias.is_empty()
        && profile.note.is_empty()
        && profile.tags.is_empty()
        && profile.group_id.is_empty()
}

fn collection(name: &str) -> Collection {
    Collection::parse(name).expect("collection names are valid")
}

pub fn profile_document_id(device_key: &str) -> DocumentId {
    let hex = Sha256::digest(device_key.as_bytes())
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    DocumentId::parse(&format!("device-{hex}")).expect("a hex digest is a valid document id")
}

fn encode(id: DocumentId, value: &impl Serialize) -> Result<Document, DocumentError> {
    let body =
        serde_json::to_string(value).map_err(|error| DocumentError::Io(io::Error::other(error)))?;

    Document::new(id, body)
}

fn decode<T: DeserializeOwned>(document: &Document) -> Option<T> {
    yunheshiguang_contract::json::from_str(&document.body).ok()
}

pub struct ProfileBook {
    store: Arc<dyn DocumentStore>,
    cache: Mutex<Option<(Instant, Arc<Inventory>)>>,
}

impl ProfileBook {
    pub fn new(store: Arc<dyn DocumentStore>) -> Self {
        Self {
            store,
            cache: Mutex::new(None),
        }
    }

    pub fn inventory(&self) -> Arc<Inventory> {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);

        if let Some((read_at, inventory)) = cache.as_ref()
            && read_at.elapsed() < CACHE_TTL
        {
            return Arc::clone(inventory);
        }

        match self.read() {
            Ok(inventory) => {
                let inventory = Arc::new(inventory);
                *cache = Some((Instant::now(), Arc::clone(&inventory)));
                inventory
            }
            Err(error) => {
                log::warn!("profiles: reading the documents failed: {error}");
                cache
                    .as_ref()
                    .map(|(_, inventory)| Arc::clone(inventory))
                    .unwrap_or_default()
            }
        }
    }

    fn read(&self) -> Result<Inventory, DocumentError> {
        let profiles = self
            .store
            .list(&collection(PROFILES_COLLECTION))?
            .iter()
            .filter_map(|document| {
                decode::<DeviceProfile>(document).filter(|profile| {
                    !profile.device_key.is_empty()
                        && profile_document_id(&profile.device_key) == document.id
                })
            })
            .map(|profile| (profile.device_key.clone(), profile))
            .collect();
        let mut groups = self
            .store
            .list(&collection(GROUPS_COLLECTION))?
            .iter()
            .filter_map(|document| {
                decode::<DeviceGroupDefinition>(document).map(|group| DeviceGroupDefinition {
                    id: document.id.as_str().to_string(),
                    ..group
                })
            })
            .collect::<Vec<_>>();
        groups.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(Inventory { profiles, groups })
    }

    pub fn save_profiles(
        &self,
        profiles: &[DeviceProfile],
        retired_keys: &[String],
    ) -> Result<(), DocumentError> {
        let collection = collection(PROFILES_COLLECTION);
        let result = profiles
            .iter()
            .try_for_each(|profile| {
                let id = profile_document_id(&profile.device_key);

                if is_blank(profile) {
                    self.store.remove(&collection, &id)
                } else {
                    self.store.save(&collection, &encode(id, profile)?)
                }
            })
            .and_then(|()| {
                retired_keys
                    .iter()
                    .try_for_each(|key| self.store.remove(&collection, &profile_document_id(key)))
            });

        self.invalidate();
        result
    }

    pub fn save_group(&self, group: &DeviceGroupDefinition) -> Result<(), DocumentError> {
        let result = DocumentId::parse(&group.id).and_then(|id| {
            self.store
                .save(&collection(GROUPS_COLLECTION), &encode(id, group)?)
        });

        self.invalidate();
        result
    }

    pub fn remove_group(&self, id: &str) -> Result<(), DocumentError> {
        let result = DocumentId::parse(id)
            .and_then(|id| self.store.remove(&collection(GROUPS_COLLECTION), &id));

        self.invalidate();
        result
    }

    fn invalidate(&self) {
        *self.cache.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}
