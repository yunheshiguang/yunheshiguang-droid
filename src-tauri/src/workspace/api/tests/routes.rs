use super::*;

#[test]
fn answers_health_checks_without_a_token_and_nothing_else() {
    let api = api();

    assert_eq!(
        call(&api, Method::GET, "/v1/health", None, "").status(),
        StatusCode::OK
    );

    let refused = call(&api, Method::POST, SNAPSHOT, None, "{}");
    assert_eq!(refused.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json(refused)["code"], "unauthenticated");
    assert_eq!(
        call(&api, Method::POST, SNAPSHOT, Some("yhsg_wrong"), "{}").status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&api, Method::GET, "/v1/nothing-here", None, "").status(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn serves_procedures_to_callers_with_a_token() {
    let api = api();

    let snapshot = call(&api, Method::POST, SNAPSHOT, Some(&api.secret), "{}");
    assert_eq!(snapshot.status(), StatusCode::OK);
    assert_eq!(json(snapshot)["schemaVersion"], "test");

    let enterprise = call(&api, Method::POST, SNAPSHOT, Some("enterprise-key"), "{}");
    assert_eq!(enterprise.status(), StatusCode::OK);

    let read = call(
        &api,
        Method::GET,
        &format!("{SNAPSHOT}?encoding=json&message=%7B%7D&connect=v1"),
        Some(&api.secret),
        "",
    );
    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(json(read)["schemaVersion"], "test");
}

#[test]
fn reads_the_bearer_scheme_in_any_letter_case() {
    let api = api();
    let request = Request::builder()
        .method(Method::POST)
        .uri(SNAPSHOT)
        .header("content-type", "application/json")
        .header("authorization", format!("bearer   {}", api.secret))
        .body(Body::from("{}"))
        .unwrap();

    let response =
        tauri::async_runtime::block_on(router(api.context.clone()).oneshot(request)).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn acts_as_the_caller_that_presented_the_token() {
    let api = api();

    let batch = call(
        &api,
        Method::POST,
        BATCH,
        Some(&api.secret),
        r#"{"kind":"BATCH_ACTION_KIND_SCREENSHOT","deviceIds":["a"]}"#,
    );
    assert_eq!(batch.status(), StatusCode::OK);
    assert_eq!(json(batch)["accepted"], true);
    call(&api, Method::POST, BATCH, Some("enterprise-key"), "{}");

    assert_eq!(api.callers.names(), ["ci", "ops"]);
}

#[test]
fn refuses_procedures_the_local_api_does_not_serve() {
    let api = api();

    let refused = call(
        &api,
        Method::POST,
        "/yunheshiguang.api.v1.ApiGatewayService/CreateApiToken",
        Some(&api.secret),
        r#"{"name":"more"}"#,
    );
    assert_eq!(refused.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(refused)["code"], "unimplemented");
}

#[test]
fn limits_each_caller_separately() {
    let api = api();
    let app = router(api.context.clone());
    let post = |token: &str| {
        Request::builder()
            .method(Method::POST)
            .uri(SNAPSHOT)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from("{}"))
            .unwrap()
    };

    let statuses = tauri::async_runtime::block_on(async {
        let mut statuses = Vec::new();
        for _ in 0..25 {
            statuses.push(
                app.clone()
                    .oneshot(post(&api.secret))
                    .await
                    .unwrap()
                    .status(),
            );
        }
        statuses.push(
            app.clone()
                .oneshot(post("enterprise-key"))
                .await
                .unwrap()
                .status(),
        );
        statuses
    });

    assert!(
        statuses[..20]
            .iter()
            .all(|status| *status == StatusCode::OK)
    );
    assert_eq!(statuses[24], StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(statuses[25], StatusCode::OK);

    let limited = tauri::async_runtime::block_on(app.oneshot(post(&api.secret))).unwrap();
    assert_eq!(limited.headers()["retry-after"], "1");
    assert_eq!(json(limited)["code"], "resource_exhausted");
}

#[test]
fn takes_query_tokens_only_for_the_event_stream() {
    let api = api();
    api.context.relay.open();
    let events = format!("/v1/events?access_token={}", api.secret);
    let snapshot = format!("{SNAPSHOT}?access_token={}", api.secret);

    assert_ne!(
        call(&api, Method::GET, &events, None, "").status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        call(&api, Method::POST, &snapshot, None, "{}").status(),
        StatusCode::UNAUTHORIZED
    );
}

#[test]
fn hands_out_the_openapi_document() {
    let api = api();

    assert_eq!(
        call(&api, Method::GET, "/v1/openapi.json", None, "").status(),
        StatusCode::UNAUTHORIZED
    );
    let document = call(&api, Method::GET, "/v1/openapi.json", Some(&api.secret), "");
    assert_eq!(document.status(), StatusCode::OK);
    let document = json(document);
    assert_eq!(document["openapi"], "3.1.0");
    assert!(document["paths"][SNAPSHOT]["post"].is_object());
}
