use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::write_atomic;

pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

fn is_name(value: &str, max_len: usize, allowed: impl Fn(u8) -> bool) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(allowed)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Collection(String);

impl Collection {
    pub fn parse(raw: &str) -> Result<Self, DocumentError> {
        if is_name(raw, 64, |byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
        }) {
            Ok(Self(raw.to_string()))
        } else {
            Err(DocumentError::InvalidName)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn parse(raw: &str) -> Result<Self, DocumentError> {
        if is_name(raw, 128, |byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        }) {
            Ok(Self(raw.to_string()))
        } else {
            Err(DocumentError::InvalidName)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Document {
    pub id: DocumentId,
    pub body: String,
}

impl Document {
    pub fn new(id: DocumentId, body: impl Into<String>) -> Result<Self, DocumentError> {
        let body = body.into();

        if body.len() > MAX_DOCUMENT_BYTES {
            return Err(DocumentError::TooLarge);
        }

        serde_json::from_str::<serde_json::Value>(&body).map_err(|_| DocumentError::NotJson)?;

        Ok(Self { id, body })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentError {
    InvalidName,
    TooLarge,
    NotJson,
    Io(io::Error),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("invalid collection or document name"),
            Self::TooLarge => write!(
                formatter,
                "document is larger than {MAX_DOCUMENT_BYTES} bytes"
            ),
            Self::NotJson => formatter.write_str("document body is not valid JSON"),
            Self::Io(error) => write!(formatter, "document storage failed: {error}"),
        }
    }
}

impl Error for DocumentError {}

impl From<io::Error> for DocumentError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait DocumentStore: Send + Sync {
    fn list(&self, collection: &Collection) -> Result<Vec<Document>, DocumentError>;

    fn save(&self, collection: &Collection, document: &Document) -> Result<(), DocumentError>;

    fn remove(&self, collection: &Collection, id: &DocumentId) -> Result<(), DocumentError>;

    fn contains(&self, collection: &Collection, id: &DocumentId) -> Result<bool, DocumentError> {
        Ok(self
            .list(collection)?
            .iter()
            .any(|document| &document.id == id))
    }
}

pub struct FolderDocumentStore {
    root: PathBuf,
}

impl FolderDocumentStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn collection_dir(&self, collection: &Collection) -> PathBuf {
        self.root.join(collection.as_str())
    }
}

impl DocumentStore for FolderDocumentStore {
    fn list(&self, collection: &Collection) -> Result<Vec<Document>, DocumentError> {
        let entries = match fs::read_dir(self.collection_dir(collection)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut documents = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    log::warn!("documents {}: {error}", collection.as_str());
                    continue;
                }
            };
            let readable = entry.metadata().is_ok_and(|metadata| {
                metadata.is_file() && metadata.len() <= MAX_DOCUMENT_BYTES as u64
            });
            if !readable {
                continue;
            }
            let path = entry.path();

            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }

            let Some(id) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| DocumentId::parse(stem).ok())
            else {
                continue;
            };

            match fs::read_to_string(&path) {
                Ok(body) => {
                    if let Ok(document) = Document::new(id, body) {
                        documents.push(document);
                    }
                }
                Err(error) => log::warn!("documents {}: {error}", path.display()),
            }
        }

        documents.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(documents)
    }

    fn save(&self, collection: &Collection, document: &Document) -> Result<(), DocumentError> {
        let path = self
            .collection_dir(collection)
            .join(format!("{}.json", document.id.as_str()));

        Ok(write_atomic(&path, document.body.as_bytes())?)
    }

    fn remove(&self, collection: &Collection, id: &DocumentId) -> Result<(), DocumentError> {
        let path = self
            .collection_dir(collection)
            .join(format!("{}.json", id.as_str()));

        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn contains(&self, collection: &Collection, id: &DocumentId) -> Result<bool, DocumentError> {
        let path = self
            .collection_dir(collection)
            .join(format!("{}.json", id.as_str()));

        match fs::symlink_metadata(path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        Collection, Document, DocumentError, DocumentId, DocumentStore, FolderDocumentStore,
    };

    fn id(raw: &str) -> DocumentId {
        DocumentId::parse(raw).unwrap()
    }

    #[test]
    fn keeps_names_safe_to_use_as_paths() {
        for name in ["automation-flows", "a", "flows2"] {
            assert!(Collection::parse(name).is_ok(), "{name}");
        }

        for name in ["", "../etc", "Flows", "-flows", "a/b", "a b"] {
            assert!(matches!(
                Collection::parse(name),
                Err(DocumentError::InvalidName)
            ));
        }

        assert!(DocumentId::parse("0b6f1c2e-3a4d-4e5f-8a9b-0c1d2e3f4a5b").is_ok());
        assert!(DocumentId::parse("..").is_err());
        assert!(DocumentId::parse("a/b").is_err());
        assert!(
            DocumentId::parse("Battery").is_err(),
            "case-insensitive disks would map it onto `battery`"
        );
    }

    #[test]
    fn accepts_only_bounded_json_bodies() {
        assert!(Document::new(id("a"), r#"{"name":"x"}"#).is_ok());
        assert!(matches!(
            Document::new(id("a"), "not json"),
            Err(DocumentError::NotJson)
        ));
        assert!(matches!(
            Document::new(
                id("a"),
                format!("\"{}\"", "x".repeat(super::MAX_DOCUMENT_BYTES))
            ),
            Err(DocumentError::TooLarge)
        ));
    }

    #[test]
    fn lists_only_regular_files_small_enough_to_be_documents() {
        let root = tempfile::tempdir().unwrap();
        let store = FolderDocumentStore::new(root.path());
        let flows = Collection::parse("automation-flows").unwrap();
        store
            .save(&flows, &Document::new(id("kept"), r#"{"v":1}"#).unwrap())
            .unwrap();
        let folder = root.path().join("automation-flows");
        fs::write(
            folder.join("huge.json"),
            format!("\"{}\"", "x".repeat(super::MAX_DOCUMENT_BYTES + 1)),
        )
        .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(folder.join("kept.json"), folder.join("linked.json")).unwrap();

        let listed = store.list(&flows).unwrap();
        assert_eq!(
            listed
                .iter()
                .map(|document| document.id.as_str())
                .collect::<Vec<_>>(),
            ["kept"]
        );
        assert!(store.contains(&flows, &id("kept")).unwrap());
        assert!(!store.contains(&flows, &id("absent")).unwrap());
    }

    #[test]
    fn saves_lists_replaces_and_removes_documents() {
        let root = tempfile::tempdir().unwrap();
        let store = FolderDocumentStore::new(root.path());
        let flows = Collection::parse("automation-flows").unwrap();

        assert!(store.list(&flows).unwrap().is_empty());

        store
            .save(&flows, &Document::new(id("b"), r#"{"v":1}"#).unwrap())
            .unwrap();
        store
            .save(&flows, &Document::new(id("a"), r#"{"v":1}"#).unwrap())
            .unwrap();
        store
            .save(&flows, &Document::new(id("b"), r#"{"v":2}"#).unwrap())
            .unwrap();

        let listed = store.list(&flows).unwrap();
        assert_eq!(
            listed.iter().map(|doc| doc.id.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(listed[1].body, r#"{"v":2}"#);

        store.remove(&flows, &id("a")).unwrap();
        store.remove(&flows, &id("missing")).unwrap();
        assert_eq!(store.list(&flows).unwrap().len(), 1);
    }

    #[test]
    fn skips_files_that_are_not_valid_documents() {
        let root = tempfile::tempdir().unwrap();
        let store = FolderDocumentStore::new(root.path());
        let flows = Collection::parse("automation-flows").unwrap();
        let directory = root.path().join("automation-flows");

        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("good.json"), "{}").unwrap();
        fs::write(directory.join("broken.json"), "{").unwrap();
        fs::write(directory.join("good (conflicted copy).json"), "{}").unwrap();
        fs::write(directory.join("notes.txt"), "hello").unwrap();
        fs::write(directory.join("latin1.json"), [0x22, 0xe9, 0x22]).unwrap();
        fs::create_dir(directory.join("folder.json")).unwrap();

        let ids: Vec<_> = store
            .list(&flows)
            .unwrap()
            .into_iter()
            .map(|doc| doc.id.as_str().to_string())
            .collect();
        assert_eq!(ids, ["good"]);
    }
}
