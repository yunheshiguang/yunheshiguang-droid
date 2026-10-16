use yunheshiguang_contract::yunheshiguang::documents::v1::{
    Document as DocumentContract, DocumentList, ListDocumentsRequest, RemoveDocumentRequest,
    SaveDocumentRequest,
};
use yunheshiguang_sdk::documents::{Collection, Document, DocumentId, DocumentStore};

fn collection(raw: &str) -> Result<Collection, String> {
    Collection::parse(raw).map_err(|error| error.to_string())
}

pub fn list(
    store: &dyn DocumentStore,
    request: &ListDocumentsRequest,
) -> Result<DocumentList, String> {
    let collection = collection(&request.collection)?;
    let documents = store
        .list(&collection)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|document| DocumentContract {
            id: document.id.as_str().to_string(),
            body: document.body,
            ..Default::default()
        })
        .collect();

    Ok(DocumentList {
        collection: collection.as_str().to_string(),
        documents,
        ..Default::default()
    })
}

pub fn checked_save(request: &SaveDocumentRequest) -> Result<(Collection, Document), String> {
    let collection = collection(&request.collection)?;
    let incoming = request.document.clone().into_option().unwrap_or_default();
    let id = DocumentId::parse(&incoming.id).map_err(|error| error.to_string())?;
    let document = Document::new(id, incoming.body).map_err(|error| error.to_string())?;

    Ok((collection, document))
}

pub fn checked_remove(request: &RemoveDocumentRequest) -> Result<(Collection, DocumentId), String> {
    let collection = collection(&request.collection)?;
    let id = DocumentId::parse(&request.id).map_err(|error| error.to_string())?;

    Ok((collection, id))
}

#[cfg(test)]
pub fn save(
    store: &dyn DocumentStore,
    request: &SaveDocumentRequest,
) -> Result<Collection, String> {
    let (collection, document) = checked_save(request)?;

    store
        .save(&collection, &document)
        .map_err(|error| error.to_string())?;
    Ok(collection)
}

#[cfg(test)]
pub fn remove(
    store: &dyn DocumentStore,
    request: &RemoveDocumentRequest,
) -> Result<Collection, String> {
    let (collection, id) = checked_remove(request)?;

    store
        .remove(&collection, &id)
        .map_err(|error| error.to_string())?;
    Ok(collection)
}
