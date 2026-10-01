//! Commands for the 2.6.7+ server features: background tasks, theme copy and usage, schema
//! export/import, batch import, functions, and the public-site check.

mod common;

use common::{Harness, ID, Req};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

fn summary(w: &[Req]) -> Vec<String> {
    w.iter()
        .map(|r| format!("{} {}", r.method, r.path))
        .collect()
}

fn task(status: &str, info: &str) -> Value {
    json!({
        "id": "T1",
        "status": { "developerName": status, "label": status },
        "statusInfo": info,
        "errorMessage": if status == "error" { json!("boom") } else { Value::Null },
        "percentComplete": 100,
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn task_wait_returns_the_final_status_and_fails_on_error() {
    let h = Harness::new().await;
    h.ok("GET", "/backgroundtasks/T1", task("complete", "done"))
        .await;
    let out = h.run(&["task", "wait", "T1"]).await;
    assert_eq!(out.data()["status"], "complete");
    assert_eq!(out.data()["statusInfo"], "done");

    let bad = Harness::new().await;
    bad.ok("GET", "/backgroundtasks/T1", task("error", "x"))
        .await;
    let out = bad.run(&["task", "wait", "T1"]).await;
    assert_eq!(out.code, 6);
    assert_eq!(out.error()["code"], "task_failed");
    assert_eq!(out.error()["message"], "boom");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn theme_duplicate_posts_the_body_and_optionally_waits() {
    let h = Harness::new().await;
    h.ok("POST", "/themes/aurora/duplicate", json!("T1")).await;
    h.ok("GET", "/backgroundtasks/T1", task("complete", "finished"))
        .await;
    let quick = h
        .run(&[
            "theme",
            "duplicate",
            "aurora",
            "Aurora Copy",
            "--title",
            "Copy",
        ])
        .await;
    assert_eq!(quick.data()["async"], true);
    assert_eq!(quick.data()["theme"], "aurora_copy");
    assert_eq!(
        summary(&h.writes().await),
        ["POST /themes/aurora/duplicate"]
    );
    assert_eq!(
        h.writes().await[0].body.as_ref().unwrap(),
        &json!({"title": "Copy", "developerName": "aurora_copy", "description": "Copy"})
    );

    let waited = h
        .run(&["theme", "duplicate", "aurora", "second", "--wait"])
        .await;
    assert_eq!(waited.data()["status"], "complete");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn match_templates_can_wait_for_the_job() {
    let h = Harness::new().await;
    h.ok("POST", "/themes/mine/match-web-templates", json!("T1"))
        .await;
    h.ok("GET", "/backgroundtasks/T1", task("complete", "activated"))
        .await;
    let out = h
        .run(&["theme", "match-templates", "mine", "--map", "a=b", "--wait"])
        .await;
    assert_eq!(out.data()["statusInfo"], "activated");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn theme_usage_filters_by_template_or_unused() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/themes/mine/usage",
        json!({"templates": [
            {"developerName": "used", "inUse": true, "isBuiltInTemplate": false},
            {"developerName": "idle", "inUse": false, "isBuiltInTemplate": false},
            {"developerName": "raytha_html_home", "inUse": false, "isBuiltInTemplate": true},
        ]}),
    )
    .await;
    let all = h.run(&["theme", "usage", "mine"]).await;
    assert_eq!(all.data()["templates"].as_array().unwrap().len(), 3);
    let unused = h.run(&["theme", "usage", "mine", "--unused"]).await;
    let names: Vec<_> = unused.data()["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["developerName"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, ["idle"]);
    let one = h
        .run(&["theme", "usage", "mine", "--template", "nope"])
        .await;
    assert_eq!(one.code, 4);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn schema_export_summary_and_import_dry_run() {
    let doc = json!({"schemaVersion": 1, "contentTypes": [{
        "developerName": "posts", "primaryField": "title", "defaultRouteTemplate": "posts/{PrimaryField}",
        "fields": [
            {"developerName": "title", "fieldType": "single_line_text", "isRequired": true},
            {"developerName": "author", "fieldType": "one_to_one_relationship", "relatedContentType": "people"},
            {"developerName": "kind", "fieldType": "dropdown", "choices": [{"developerName": "a"}, {"developerName": "b"}]}
        ],
        "views": [{"developerName": "posts_all", "routePath": "posts"}]
    }]});
    let h = Harness::new().await;
    h.ok("GET", "/contenttypes/export", doc.clone()).await;
    h.ok(
        "POST",
        "/contenttypes/import",
        json!({"dryRun": true, "created": 0, "updated": 1, "unchanged": 2, "changes": []}),
    )
    .await;

    let out = h.run(&["schema", "export", "--summary"]).await;
    let t = &out.data()["contentTypes"][0];
    assert_eq!(
        t["fields"],
        json!([
            "title:single_line_text*",
            "author:one_to_one_relationship->people",
            "kind:dropdown[a|b]"
        ])
    );
    assert_eq!(t["views"], json!(["posts_all -> /posts"]));

    // the CLI's own export envelope is accepted as input
    let envelope = json!({"ok": true, "data": doc});
    let imp = h
        .run_stdin(
            &["schema", "import", "-", "--dry-run"],
            &envelope.to_string(),
        )
        .await;
    assert_eq!(imp.data()["updated"], 1);
    let w = h.writes().await;
    assert_eq!(summary(&w), ["POST /contenttypes/import"]);
    assert_eq!(w[0].query, vec![("dryRun".to_string(), "true".to_string())]);
    assert_eq!(
        w[0].body.as_ref().unwrap()["contentTypes"][0]["developerName"],
        "posts"
    );

    let bad = h.run_stdin(&["schema", "import", "-"], "{}").await;
    assert_eq!(bad.error()["code"], "usage");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn function_edit_keeps_what_you_do_not_change() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/functions/hello",
        json!({
            "name": "Hello", "developerName": "hello", "isActive": true, "code": "old()",
            "routePath": "hi", "triggerType": {"developerName": "http_request", "label": "Http request"}
        }),
    )
    .await;
    h.ok("PUT", "/functions/hello", json!(ID)).await;
    h.ok("DELETE", "/functions/hello", json!(ID)).await;
    h.ok("POST", "/functions", json!(ID)).await;

    h.run(&[
        "function",
        "edit",
        "hello",
        "--content",
        "new()",
        "--active",
        "false",
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(
        w[0].body.as_ref().unwrap(),
        &json!({"name": "Hello", "triggerType": "http_request", "isActive": false, "code": "new()", "routePath": "hi"})
    );

    assert_eq!(
        h.run(&["function", "delete", "hello"]).await.error()["code"],
        "usage"
    );
    h.run(&["function", "delete", "hello", "--yes"])
        .await
        .data();
    let bad = h
        .run(&[
            "function",
            "create",
            "x",
            "--trigger",
            "nope",
            "--content",
            "c",
        ])
        .await;
    assert_eq!(bad.error()["code"], "usage");
    h.run(&[
        "function",
        "create",
        "llms.txt",
        "--trigger",
        "http-request",
        "--content",
        "c",
        "--route-path",
        "llms.txt",
    ])
    .await
    .data();
    let w = h.writes().await;
    let create = w.last().unwrap().body.as_ref().unwrap();
    assert_eq!(create["developerName"], "llms.txt");
    assert_eq!(create["triggerType"], "http_request");
    assert_eq!(create["isActive"], true);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_import_batches_waits_and_reports_failed_rows() {
    let h = Harness::new().await;
    h.ok("POST", "/contentitems/books/batch", json!("T1")).await;
    let result = json!({
        "total": 2, "created": 1, "failed": 1,
        "items": [
            {"index": 0, "success": true, "id": "X1", "errors": []},
            {"index": 1, "success": false, "id": null, "errors": [{"field": "author", "message": "No item found"}]}
        ]
    });
    h.ok(
        "GET",
        "/backgroundtasks/T1",
        task("complete", &result.to_string()),
    )
    .await;

    let rows = "{\"title\":\"A\"}\n{\"title\":\"B\",\"author\":\"Nobody\"}\n";
    let out = h
        .run_stdin(
            &[
                "content",
                "import",
                "books",
                "--file",
                "-",
                "--template-id",
                ID,
                "--draft",
            ],
            rows,
        )
        .await;
    assert_eq!(out.code, 5, "{}", out.stdout);
    let e = out.error();
    assert!(e["message"].as_str().unwrap().starts_with("1 of 2"));
    assert_eq!(
        e["fields"]["report"]["items"][1]["errors"][0]["field"],
        "author"
    );

    let w = h.writes().await;
    let body = w[0].body.as_ref().unwrap();
    assert_eq!(body["templateId"], ID);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert_eq!(
        body["items"][0],
        json!({"content": {"title": "A"}, "saveAsDraft": true})
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn at_file_values_are_uploaded_and_replaced_by_the_object_key() {
    let dir = tempfile::tempdir().unwrap();
    let img = dir.path().join("cover.png");
    std::fs::write(&img, "png").unwrap();
    let h = Harness::new().await;
    h.ok(
        "POST",
        "/mediaitems/upload-direct",
        json!({"id": "KEY_cover.png"}),
    )
    .await;
    h.ok(
        "GET",
        "/mediaitems/KEY_cover.png",
        json!("http://x/cover.png"),
    )
    .await;
    h.ok("POST", "/contentitems/books", json!(ID)).await;
    let data = json!({"title": "A", "cover": format!("@file:{}", img.display())}).to_string();
    h.run(&[
        "content",
        "create",
        "books",
        "--template-id",
        ID,
        "--data",
        &data,
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(
        summary(&w),
        ["POST /mediaitems/upload-direct", "POST /contentitems/books"]
    );
    assert_eq!(
        w[1].body.as_ref().unwrap()["content"]["cover"],
        "KEY_cover.png"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn views_are_compact_unless_full_and_site_page_list_is_brief() {
    let h = Harness::new().await;
    h.paged("/contenttypes/posts/views", vec![json!({"id": "V1"})])
        .await;
    h.ok("GET", "/contenttypes/posts/views/V1", json!({"id": "V1"}))
        .await;
    h.paged(
        "/sitepages",
        vec![json!({
            "id": "P1", "title": "About", "isPublished": true, "routePath": "about",
            "webTemplate": {"developerName": "page", "content": "<big/>"},
            "widgets": {"main": [{"id": "w"}]}
        })],
    )
    .await;

    h.run(&["content-type", "views", "list", "posts"])
        .await
        .data();
    h.run(&["content-type", "views", "list", "posts", "--full"])
        .await
        .data();
    h.run(&["content-type", "views", "get", "posts", "V1", "--full"])
        .await
        .data();
    let reqs = h.requests().await;
    let compact: Vec<_> = reqs
        .iter()
        .map(|r| r.query.iter().any(|(k, v)| k == "compact" && v == "false"))
        .collect();
    assert_eq!(compact, [false, true, true]);

    let brief = h.run(&["site-page", "list"]).await;
    let p = &brief.data()["items"][0];
    assert_eq!(p["template"], "page");
    assert!(p.get("widgets").is_none() && p.get("webTemplate").is_none());
    let full = h.run(&["site-page", "list", "--full"]).await;
    assert!(full.data()["items"][0].get("widgets").is_some());
    h.assert_contract().await;
}

async fn public(h: &Harness, p: &str, status: u16, body: &str) {
    Mock::given(method("GET"))
        .and(path(p))
        .respond_with(ResponseTemplate::new(status).set_body_string(body))
        .mount(&h.server)
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn check_requests_public_routes_and_exits_6_on_failure() {
    let h = Harness::new().await;
    h.paged("/contenttypes", vec![json!({"developerName": "posts"})])
        .await;
    h.paged(
        "/contenttypes/posts/views",
        vec![
            json!({"isPublished": true, "routePath": "posts"}),
            json!({"isPublished": false, "routePath": "hidden"}),
        ],
    )
    .await;
    h.paged(
        "/contentitems/posts",
        vec![
            json!({"isPublished": true, "routePath": "posts/ok"}),
            json!({"isPublished": true, "routePath": "posts/broken"}),
        ],
    )
    .await;
    h.paged(
        "/sitepages",
        vec![json!({"isPublished": true, "routePath": "about"})],
    )
    .await;
    public(&h, "/", 200, "<h1>home</h1>").await;
    public(&h, "/posts", 200, "<ul></ul>").await;
    public(&h, "/posts/ok", 200, "<p>{{ leftover }}</p>").await;
    public(
        &h,
        "/posts/broken",
        500,
        "posts_detail: Attempted to divide by zero.",
    )
    .await;
    public(&h, "/about", 200, "").await;
    public(&h, "/raytha-check-no-such-page", 404, "not found").await;

    let out = h.run(&["check"]).await;
    assert_eq!(out.code, 6, "{}", out.stdout);
    let report = &out.error()["fields"]["report"];
    assert_eq!(report["checked"], 6);
    assert_eq!(report["failed"], 2);
    assert_eq!(report["warnings"], 1);
    let routes = report["routes"].as_array().unwrap();
    let broken = routes
        .iter()
        .find(|r| r["path"] == "/posts/broken")
        .unwrap();
    assert!(broken["body"].as_str().unwrap().contains("posts_detail"));
    assert!(routes.iter().any(
        |r| r["path"] == "/about" && r["problems"][0].as_str().unwrap().contains("empty body")
    ));
    // healthy routes are only listed with --all, and unpublished views are skipped
    assert!(
        routes
            .iter()
            .all(|r| r["path"] != "/posts" && r["path"] != "/hidden")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn check_passes_a_healthy_site() {
    let h = Harness::new().await;
    h.paged("/contenttypes", vec![]).await;
    h.paged("/sitepages", vec![]).await;
    public(&h, "/", 200, "<h1>home</h1>").await;
    public(&h, "/raytha-check-no-such-page", 404, "nope").await;
    let out = h.run(&["check", "--all"]).await;
    let d = out.data();
    assert_eq!(
        (d["failed"].as_u64(), d["warnings"].as_u64()),
        (Some(0), Some(0))
    );
    assert_eq!(d["routes"].as_array().unwrap().len(), 2);
}
