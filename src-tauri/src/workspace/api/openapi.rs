use std::sync::OnceLock;

use serde_json::{Value, json};
use yunheshiguang_contract::LOCAL_API_PROCEDURES;

const OPENAPI_JSON: &str = include_str!("openapi/local-api.openapi.json");

fn served(mut document: Value) -> Value {
    if let Some(paths) = document.get_mut("paths").and_then(Value::as_object_mut) {
        paths.retain(|path, _| LOCAL_API_PROCEDURES.contains(&path.as_str()));
    }
    document["info"]["title"] = json!("yunheshiguang local API");
    document["info"]["description"] = json!(
        "The app's procedures over Connect, gRPC and gRPC-Web. Every call needs a bearer token."
    );
    document["components"]["securitySchemes"] = json!({
        "bearer": { "type": "http", "scheme": "bearer" }
    });
    document["security"] = json!([{ "bearer": [] }]);

    document
}

pub fn document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();

    DOCUMENT.get_or_init(|| {
        served(serde_json::from_str(OPENAPI_JSON).expect("the OpenAPI document is JSON"))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn describes_exactly_the_procedures_the_local_api_serves() {
        let paths = document()["paths"]
            .as_object()
            .expect("the OpenAPI document has paths");
        let described: BTreeSet<&str> = paths.keys().map(String::as_str).collect();

        assert_eq!(described, LOCAL_API_PROCEDURES.iter().copied().collect());
        assert!(described.len() > 20);
        assert!(
            paths
                .values()
                .all(|operations| operations.get("post").is_some())
        );
        assert!(
            paths["/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot"]
                .get("get")
                .is_some()
        );
        assert_eq!(document()["security"], json!([{ "bearer": [] }]));
    }
}
