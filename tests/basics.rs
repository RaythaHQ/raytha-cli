mod common;

use common::{Harness, ID};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn missing_config_is_a_usage_error_naming_every_variable() {
    let h = Harness::new().await;
    let out = tokio::task::spawn_blocking(|| {
        std::process::Command::new(env!("CARGO_BIN_EXE_raytha"))
            .args(["theme", "list"])
            .env_remove("RAYTHA_URL")
            .env_remove("RAYTHA_API_KEY")
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    drop(h);
    assert_eq!(out.status.code(), Some(2));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], false);
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("RAYTHA_URL") && msg.contains("RAYTHA_API_KEY"),
        "{msg}"
    );
    assert!(v["error"]["hint"].as_str().unwrap().contains("API Keys"));
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_flag_is_json_on_stdout_with_exit_2() {
    let h = Harness::new().await;
    let out = h.run(&["theme", "list", "--nope"]).await;
    assert_eq!(out.code, 2);
    assert_eq!(out.error()["code"], "usage");
}

#[tokio::test(flavor = "multi_thread")]
async fn help_is_text_and_exits_zero() {
    let h = Harness::new().await;
    let out = h.run(&["--help"]).await;
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("raytha guide"));
}

#[tokio::test(flavor = "multi_thread")]
async fn guide_prints_markdown_and_needs_no_config() {
    let h = Harness::new().await;
    let out = h.run_env(&["guide", "errors"], &[("RAYTHA_URL", "")]).await;
    assert_eq!(out.code, 0);
    assert!(out.stdout.starts_with("# errors"));
    let bad = h.run(&["guide", "missing-topic"]).await;
    assert_eq!(bad.code, 2);
    assert!(
        bad.error()["hint"]
            .as_str()
            .unwrap()
            .contains("build-a-site")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn guide_json_wraps_markdown_in_the_envelope() {
    let h = Harness::new().await;
    let out = h.run(&["guide", "media", "--json"]).await;
    assert_eq!(out.data()["topic"], "media");
    assert!(out.data()["markdown"].as_str().unwrap().contains("media"));
}

#[tokio::test(flavor = "multi_thread")]
async fn http_errors_map_to_exit_codes_and_hints() {
    let h = Harness::new().await;
    h.respond("GET", "/themes", 401, json!({"title": "Unauthorized"}))
        .await;
    let out = h.run(&["theme", "list"]).await;
    assert_eq!(out.code, 3);
    assert_eq!(out.error()["code"], "unauthorized");
    assert!(
        out.error()["hint"]
            .as_str()
            .unwrap()
            .contains("RAYTHA_API_KEY")
    );

    let h = Harness::new().await;
    h.respond("GET", "/sitepages", 403, json!({"title": "Forbidden"}))
        .await;
    let out = h.run(&["site-page", "list"]).await;
    assert_eq!(out.code, 3);
    assert_eq!(out.error()["code"], "forbidden");
    assert!(
        out.error()["hint"]
            .as_str()
            .unwrap()
            .contains("ManageSitePages")
    );

    let h = Harness::new().await;
    h.respond(
        "GET",
        "/themes/nope",
        404,
        json!({"title": "Not Found", "detail": "Theme not found"}),
    )
    .await;
    let out = h.run(&["theme", "get", "nope"]).await;
    assert_eq!(out.code, 4);
    assert_eq!(out.error()["code"], "not_found");
    assert_eq!(out.error()["status"], 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn validation_errors_carry_field_messages() {
    let h = Harness::new().await;
    h.respond(
        "POST",
        "/themes",
        400,
        json!({
            "title": "One or more validation errors occurred.",
            "status": 400,
            "errors": { "DeveloperName": ["Developer name is taken."] }
        }),
    )
    .await;
    let out = h.run(&["theme", "create", "taken"]).await;
    assert_eq!(out.code, 5);
    let e = out.error();
    assert_eq!(e["code"], "validation_failed");
    assert_eq!(e["fields"]["DeveloperName"][0], "Developer name is taken.");
}

#[tokio::test(flavor = "multi_thread")]
async fn server_errors_exit_6_and_reads_are_retried() {
    let h = Harness::new().await;
    h.respond("GET", "/themes", 503, json!({"title": "Unavailable"}))
        .await;
    let out = h.run(&["theme", "list"]).await;
    assert_eq!(out.code, 6);
    assert!(h.requests().await.len() >= 2, "GET should be retried");

    let h = Harness::new().await;
    h.respond("POST", "/themes", 500, json!({"title": "Boom"}))
        .await;
    let out = h.run(&["theme", "create", "x"]).await;
    assert_eq!(out.code, 6);
    assert_eq!(h.writes().await.len(), 1, "writes must not be retried");
}

#[tokio::test(flavor = "multi_thread")]
async fn doctor_reports_permissions() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/ping",
        json!({"version": "2.0.0", "organizationName": "Acme"}),
    )
    .await;
    h.ok("GET", "/themes", json!({"items": [], "totalCount": 0}))
        .await;
    h.ok(
        "GET",
        "/contenttypes",
        json!({"items": [], "totalCount": 0}),
    )
    .await;
    h.ok("GET", "/mediaitems", json!({"items": [], "totalCount": 0}))
        .await;
    h.respond("GET", "/sitepages", 403, json!({"title": "Forbidden"}))
        .await;
    h.respond("GET", "/users", 403, json!({"title": "Forbidden"}))
        .await;
    let out = h.run(&["doctor"]).await;
    let d = out.data();
    assert_eq!(d["permissions"]["templates"], true);
    assert_eq!(d["permissions"]["sitePages"], false);
    assert!(
        d["missingPermissions"]
            .to_string()
            .contains("ManageSitePages")
    );
    assert!(d["hint"].is_string());
    assert_eq!(d["missingPermissions"].as_array().unwrap().len(), 2);
    h.assert_contract().await;
    let _ = ID;
}

#[tokio::test(flavor = "multi_thread")]
async fn url_flag_overrides_env_and_trailing_paths_are_stripped() {
    let h = Harness::new().await;
    h.ok("GET", "/ping", json!({"version": "2.0.0"})).await;
    let pasted = format!("{}/raytha/api/v1/", h.url());
    let out = h
        .run_env(
            &["--url", &pasted, "doctor"],
            &[("RAYTHA_URL", "http://127.0.0.1:1")],
        )
        .await;
    assert!(out.json["ok"].is_boolean(), "{}", out.stdout);
    assert!(!h.requests().await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn spec_keeps_only_v1_paths() {
    let h = Harness::new().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/raytha/api/v1/swagger.json"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(json!({
            "paths": {
                "/raytha/api/v1/themes": {"get": {"operationId": "GetThemes"}},
                "/raytha/admin/things": {"get": {"operationId": "Other"}}
            }
        })))
        .mount(&h.server)
        .await;
    let out = h.run(&["spec", "--summary"]).await;
    let text = out.data().to_string();
    assert!(text.contains("/raytha/api/v1/themes"), "{text}");
    assert!(!text.contains("admin/things"), "{text}");
}
