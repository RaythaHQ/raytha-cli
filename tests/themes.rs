mod common;

use common::{Harness, ID};
use serde_json::{Value, json};
use std::fs;
use tempfile::tempdir;

fn list(items: Value) -> Value {
    let n = items.as_array().map(|a| a.len()).unwrap_or(0);
    json!({ "items": items, "totalCount": n })
}

#[tokio::test(flavor = "multi_thread")]
async fn theme_create_normalizes_the_name_and_returns_it() {
    let h = Harness::new().await;
    h.ok("POST", "/themes", json!(ID)).await;
    let out = h
        .run(&["theme", "create", "My Fancy Theme", "--title", "Fancy"])
        .await;
    let d = out.data();
    assert_eq!(d["developerName"], "my_fancy_theme");
    assert_eq!(d["id"], ID);
    let w = h.writes().await;
    let body = w[0].body.as_ref().unwrap();
    assert_eq!(body["developerName"], "my_fancy_theme");
    assert_eq!(body["title"], "Fancy");
    assert_eq!(body["insertDefaultThemeMediaItems"], false);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn destructive_commands_refuse_without_yes_and_send_nothing() {
    let h = Harness::new().await;
    for args in [
        vec!["theme", "delete", "x"],
        vec!["web-template", "delete", "x", "y"],
        vec!["content", "delete", "posts", ID],
        vec!["site-page", "delete", ID],
        vec!["menu", "delete", "main"],
    ] {
        let out = h.run(&args).await;
        assert_eq!(out.code, 2, "{args:?}: {}", out.stdout);
        assert!(out.error()["hint"].as_str().unwrap().contains("--yes"));
    }
    assert!(h.requests().await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_and_delete_hit_the_right_routes() {
    let h = Harness::new().await;
    h.ok("POST", "/themes/mine/set-active", json!(ID)).await;
    h.ok("DELETE", "/themes/mine", json!(ID)).await;
    h.run(&["theme", "activate", "mine"]).await.data();
    h.run(&["theme", "delete", "mine", "--yes"]).await.data();
    let w = h.writes().await;
    assert_eq!(
        (w[0].method.as_str(), w[0].path.as_str()),
        ("POST", "/themes/mine/set-active")
    );
    assert_eq!(
        (w[1].method.as_str(), w[1].path.as_str()),
        ("DELETE", "/themes/mine")
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn web_template_create_infers_base_layout_from_renderbody() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("layout.liquid");
    fs::write(&file, "<html>{% renderbody %}</html>").unwrap();
    let h = Harness::new().await;
    h.ok("POST", "/webtemplates/theme/mine", json!(ID)).await;
    h.run(&[
        "web-template",
        "create",
        "mine",
        "my_layout",
        "--file",
        file.to_str().unwrap(),
    ])
    .await
    .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["developerName"], "my_layout");
    assert_eq!(body["isBaseLayout"], true);
    assert_eq!(body["content"], "<html>{% renderbody %}</html>");
    assert_eq!(body["label"], "My Layout");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn web_template_edit_carries_over_the_content_type_access_list() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/post_detail",
        json!({
            "id": ID,
            "label": "Post detail",
            "content": "old",
            "isBaseLayout": false,
            "parentTemplate": { "developerName": "raytha_html_base_layout" },
            "allowAccessForNewContentTypes": true,
            "templateAccessToModelDefinitions": { "ctid1": {}, "ctid2": {} }
        }),
    )
    .await;
    h.ok(
        "GET",
        "/contenttypes",
        list(json!([
            {"id": "ctid1", "developerName": "posts"},
            {"id": "ctid2", "developerName": "events"},
            {"id": "ctid3", "developerName": "other"}
        ])),
    )
    .await;
    h.ok(
        "PUT",
        "/webtemplates/theme/mine/template/post_detail",
        json!(ID),
    )
    .await;

    h.run(&[
        "web-template",
        "edit",
        "mine",
        "post_detail",
        "--content",
        "new body",
    ])
    .await
    .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["content"], "new body");
    assert_eq!(body["label"], "Post detail");
    assert_eq!(
        body["parentTemplateDeveloperName"],
        "raytha_html_base_layout"
    );
    assert_eq!(body["allowAccessForNewContentTypes"], true);
    let mut types: Vec<&str> = body["templateAccessToModelDefinitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    types.sort();
    assert_eq!(types, ["events", "posts"]);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn web_template_get_out_writes_the_file_and_omits_content() {
    let dir = tempdir().unwrap();
    let out_path = dir.path().join("nested/t.liquid");
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/t",
        json!({"id": ID, "content": "<b>hi</b>"}),
    )
    .await;
    let out = h
        .run(&[
            "web-template",
            "get",
            "mine",
            "t",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .await;
    assert!(out.data().get("content").is_none());
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "<b>hi</b>");
}

#[tokio::test(flavor = "multi_thread")]
async fn widget_template_create_sends_fields_from_a_file() {
    let dir = tempdir().unwrap();
    let liquid = dir.path().join("w.liquid");
    let fields = dir.path().join("f.json");
    fs::write(&liquid, "<p>{{ widget.settings.headline }}</p>").unwrap();
    fs::write(
        &fields,
        r#"[{"developerName":"headline","label":"Headline","fieldType":"single_line_text"}]"#,
    )
    .unwrap();
    let h = Harness::new().await;
    h.ok("POST", "/widgettemplates/theme/mine", json!(ID)).await;
    h.run(&[
        "widget-template",
        "create",
        "mine",
        "banner",
        "--file",
        liquid.to_str().unwrap(),
        "--fields",
        &format!("@{}", fields.display()),
    ])
    .await
    .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["developerName"], "banner");
    assert_eq!(body["fields"][0]["fieldType"], "single_line_text");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn widget_template_edit_only_sends_fields_when_asked() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/widgettemplates/theme/mine/template/banner",
        json!({"id": ID, "label": "Banner", "content": "old", "fields": []}),
    )
    .await;
    h.ok(
        "PUT",
        "/widgettemplates/theme/mine/template/banner",
        json!(ID),
    )
    .await;
    h.run(&[
        "widget-template",
        "edit",
        "mine",
        "banner",
        "--content",
        "new",
    ])
    .await
    .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["content"], "new");
    assert_eq!(body["label"], "Banner");
    assert!(body.get("fields").is_none());
}

fn write_theme(dir: &std::path::Path) {
    fs::write(
        dir.join("theme.json"),
        r#"{"title":"Mine","developerName":"mine","description":"d"}"#,
    )
    .unwrap();
    fs::create_dir_all(dir.join("web-templates")).unwrap();
    fs::create_dir_all(dir.join("widget-templates")).unwrap();
    fs::write(
        dir.join("web-templates/layout.liquid"),
        "<html>{% renderbody %}</html>",
    )
    .unwrap();
    // The child sorts before its parent alphabetically, so ordering must come from `parent`.
    fs::write(dir.join("web-templates/a_page.liquid"), "<main>hi</main>").unwrap();
    fs::write(
        dir.join("web-templates/a_page.json"),
        r#"{"label":"A page","parent":"layout"}"#,
    )
    .unwrap();
    fs::write(dir.join("widget-templates/banner.liquid"), "<h1>x</h1>").unwrap();
    fs::write(
        dir.join("widget-templates/banner.json"),
        r#"{"label":"Banner","fields":[{"developerName":"headline","label":"Headline","fieldType":"single_line_text"}]}"#,
    )
    .unwrap();
}

async fn mock_absent_theme(h: &Harness) {
    h.respond("GET", "/themes/mine", 404, json!({"title": "Not Found"}))
        .await;
    h.ok("POST", "/themes", json!(ID)).await;
    h.ok("POST", "/webtemplates/theme/mine", json!(ID)).await;
    h.ok("POST", "/widgettemplates/theme/mine", json!(ID)).await;
    // After creating the theme the CLI reads the (here empty) remote lists back.
    h.ok("GET", "/webtemplates", list(json!([]))).await;
    h.ok("GET", "/widgettemplates", list(json!([]))).await;
    h.ok("GET", "/themes/mine/media", json!([])).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_dry_run_reports_creates_and_writes_nothing() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_absent_theme(&h).await;
    h.ok("POST", "/webtemplates/validate", json!({"isValid": true}))
        .await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap(), "--dry-run"])
        .await;
    let d = out.data();
    assert_eq!(d["dryRun"], true);
    assert_eq!(d["summary"]["create"], 4);
    // the only POSTs are Liquid validation calls, which save nothing
    let w = h.writes().await;
    assert!(
        w.iter().all(|r| r.path == "/webtemplates/validate"),
        "{w:?}"
    );
    assert!(!w.is_empty());
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_dry_run_reports_liquid_syntax_errors_with_position() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_absent_theme(&h).await;
    h.respond(
        "POST",
        "/webtemplates/validate",
        400,
        json!({"success": false, "error": "Invalid 'if' tag at (1:6)", "line": 1, "column": 6}),
    )
    .await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap(), "--dry-run"])
        .await;
    assert_ne!(out.code, 0, "{}", out.stdout);
    let report = &out.error()["fields"]["report"];
    let failed: Vec<_> = report["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["action"] == "failed")
        .collect();
    assert!(!failed.is_empty(), "{report}");
    assert!(failed[0].to_string().contains("(1:6)"));
    assert_eq!(
        (failed[0]["line"].as_u64(), failed[0]["column"].as_u64()),
        (Some(1), Some(6))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn push_creates_the_theme_then_parents_before_children() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_absent_theme(&h).await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    assert_eq!(out.data()["summary"]["create"], 4);

    let w = h.writes().await;
    let order: Vec<(String, String)> = w
        .iter()
        .map(|r| {
            (
                r.path.clone(),
                r.body
                    .as_ref()
                    .and_then(|b| b["developerName"].as_str())
                    .unwrap_or("")
                    .to_string(),
            )
        })
        .collect();
    assert_eq!(order[0].0, "/themes");
    assert_eq!(
        order[1],
        ("/webtemplates/theme/mine".into(), "layout".into())
    );
    assert_eq!(
        order[2],
        ("/webtemplates/theme/mine".into(), "a_page".into())
    );
    assert_eq!(order[3].0, "/widgettemplates/theme/mine");
    let child = w[2].body.as_ref().unwrap();
    assert_eq!(child["parentTemplateDeveloperName"], "layout");
    assert_eq!(child["isBaseLayout"], false);
    assert_eq!(w[1].body.as_ref().unwrap()["isBaseLayout"], true);
    h.assert_contract().await;
}

async fn mock_existing_theme(h: &Harness, layout_content: &str) {
    mock_existing_theme_with(h, layout_content, vec![], json!([])).await;
}

/// `extra_web` are additional remote web template list entries; `media` the remote theme media.
async fn mock_existing_theme_with(
    h: &Harness,
    layout_content: &str,
    extra_web: Vec<Value>,
    media: Value,
) {
    let mut web = vec![
        json!({"developerName": "layout"}),
        json!({"developerName": "a_page"}),
    ];
    web.extend(extra_web);
    h.ok(
        "GET",
        "/themes/mine",
        json!({"id": ID, "title": "Mine", "developerName": "mine", "description": "d"}),
    )
    .await;
    h.ok("GET", "/webtemplates", list(Value::Array(web))).await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/layout",
        json!({"id": ID, "label": "Layout", "content": layout_content, "isBaseLayout": true,
               "allowAccessForNewContentTypes": false, "templateAccessToModelDefinitions": {}}),
    )
    .await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/a_page",
        json!({"id": ID, "label": "A page", "content": "<main>hi</main>",
               "isBaseLayout": false, "parentTemplate": {"developerName": "layout"},
               "allowAccessForNewContentTypes": false, "templateAccessToModelDefinitions": {}}),
    )
    .await;
    h.ok(
        "GET",
        "/widgettemplates",
        list(json!([{"developerName": "banner"}])),
    )
    .await;
    h.ok(
        "GET",
        "/widgettemplates/theme/mine/template/banner",
        json!({"id": ID, "label": "Banner", "content": "<h1>x</h1>", "fields": [
            {"developerName":"headline","label":"Headline","fieldType":"single_line_text",
             "description": null, "isRequired": false, "choices": [], "subFields": []}]}),
    )
    .await;
    h.ok("GET", "/themes/mine/media", media).await;
    h.ok("PUT", "/webtemplates/theme/mine/template/layout", json!(ID))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_is_idempotent_when_remote_already_matches() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_existing_theme(&h, "<html>{% renderbody %}</html>").await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    let d = out.data();
    assert_eq!(d["summary"]["unchanged"], 4, "{d}");
    assert!(h.writes().await.is_empty());
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_updates_only_what_changed() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_existing_theme(&h, "<html>OLD {% renderbody %}</html>").await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    assert_eq!(out.data()["summary"]["update"], 1);
    let w = h.writes().await;
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].path, "/webtemplates/theme/mine/template/layout");
    assert_eq!(w[0].method, "PUT");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_reports_partial_failure_with_the_full_report() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    h.respond("GET", "/themes/mine", 404, json!({"title": "Not Found"}))
        .await;
    h.ok("POST", "/themes", json!(ID)).await;
    h.ok("POST", "/webtemplates/theme/mine", json!(ID)).await;
    h.respond(
        "POST",
        "/widgettemplates/theme/mine",
        400,
        json!({"title": "Invalid", "errors": {"Fields": ["bad field"]}}),
    )
    .await;
    h.ok("GET", "/webtemplates", list(json!([]))).await;
    h.ok("GET", "/widgettemplates", list(json!([]))).await;
    h.ok("GET", "/themes/mine/media", json!([])).await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    assert_eq!(out.code, 5);
    let e = out.error();
    assert_eq!(e["code"], "push_incomplete");
    let actions = e["fields"]["report"]["actions"].as_array().unwrap();
    assert!(
        actions
            .iter()
            .any(|a| a["action"] == "failed" && a["name"] == "banner")
    );
    assert!(
        actions
            .iter()
            .any(|a| a["action"] == "create" && a["name"] == "layout")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn push_prune_deletes_extras_but_never_built_ins() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    let h = Harness::new().await;
    mock_existing_theme_with(
        &h,
        "<html>{% renderbody %}</html>",
        vec![
            json!({"developerName": "old_page"}),
            json!({"developerName": "raytha_html_error_404"}),
        ],
        json!([]),
    )
    .await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/old_page",
        json!({"id": ID, "isBuiltInTemplate": false}),
    )
    .await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/raytha_html_error_404",
        json!({"id": ID, "isBuiltInTemplate": true}),
    )
    .await;
    h.ok(
        "DELETE",
        "/webtemplates/theme/mine/template/old_page",
        json!(ID),
    )
    .await;

    let dry = h
        .run(&[
            "theme",
            "push",
            dir.path().to_str().unwrap(),
            "--prune",
            "--dry-run",
        ])
        .await;
    let actions = dry.data()["actions"].as_array().unwrap().clone();
    assert!(
        actions
            .iter()
            .any(|a| a["name"] == "old_page" && a["action"] == "delete")
    );
    assert!(
        actions
            .iter()
            .any(|a| a["name"] == "raytha_html_error_404" && a["action"] == "skip")
    );
    assert!(h.writes().await.is_empty());

    h.run(&["theme", "push", dir.path().to_str().unwrap(), "--prune"])
        .await
        .data();
    let deletes: Vec<_> = h
        .writes()
        .await
        .into_iter()
        .filter(|r| r.method == "DELETE")
        .collect();
    assert_eq!(deletes.len(), 1);
    assert!(deletes[0].path.ends_with("/old_page"));
}

#[tokio::test(flavor = "multi_thread")]
async fn push_without_theme_json_explains_how_to_start() {
    let dir = tempdir().unwrap();
    let h = Harness::new().await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    assert_eq!(out.code, 2);
    assert!(out.error()["hint"].as_str().unwrap().contains("theme pull"));
}

#[tokio::test(flavor = "multi_thread")]
async fn pull_writes_templates_sidecars_and_widget_fields() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("out");
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/themes/mine",
        json!({"id": ID, "title": "Mine", "developerName": "mine", "description": "d"}),
    )
    .await;
    h.ok(
        "GET",
        "/webtemplates",
        list(json!([{"developerName": "layout"}])),
    )
    .await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/layout",
        json!({"id": ID, "label": "Layout", "content": "<html>{% renderbody %}</html>",
               "isBaseLayout": true, "allowAccessForNewContentTypes": false,
               "templateAccessToModelDefinitions": {}}),
    )
    .await;
    h.ok(
        "GET",
        "/widgettemplates",
        list(json!([{"developerName": "banner"}])),
    )
    .await;
    h.ok(
        "GET",
        "/widgettemplates/theme/mine/template/banner",
        json!({"id": ID, "label": "Banner", "content": "<h1>x</h1>",
               "fields": [{"developerName": "headline", "label": "Headline", "fieldType": "single_line_text"}]}),
    )
    .await;
    let out = h
        .run(&[
            "theme",
            "pull",
            "mine",
            target.to_str().unwrap(),
            "--no-media",
        ])
        .await;
    out.data();
    assert_eq!(
        fs::read_to_string(target.join("web-templates/layout.liquid")).unwrap(),
        "<html>{% renderbody %}</html>"
    );
    let side: Value = serde_json::from_str(
        &fs::read_to_string(target.join("web-templates/layout.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(side["isBaseLayout"], true);
    let wside: Value = serde_json::from_str(
        &fs::read_to_string(target.join("widget-templates/banner.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(wside["fields"][0]["developerName"], "headline");
    let theme: Value =
        serde_json::from_str(&fs::read_to_string(target.join("theme.json")).unwrap()).unwrap();
    assert_eq!(theme["developerName"], "mine");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn push_uploads_new_theme_media_and_skips_same_size_files() {
    let dir = tempdir().unwrap();
    write_theme(dir.path());
    fs::create_dir_all(dir.path().join("media")).unwrap();
    fs::write(dir.path().join("media/logo.svg"), "<svg/>").unwrap();
    fs::write(dir.path().join("media/same.css"), "a{}").unwrap();
    let h = Harness::new().await;
    mock_existing_theme_with(
        &h,
        "<html>{% renderbody %}</html>",
        vec![],
        json!([{"id": "m1", "fileName": "same.css", "length": 3}]),
    )
    .await;
    h.ok("POST", "/themes/mine/media", json!({"id": "m2"}))
        .await;
    let out = h
        .run(&["theme", "push", dir.path().to_str().unwrap()])
        .await;
    let actions = out.data()["actions"].as_array().unwrap().clone();
    assert!(
        actions
            .iter()
            .any(|a| a["name"] == "logo.svg" && a["action"] == "create")
    );
    assert!(
        actions
            .iter()
            .any(|a| a["name"] == "same.css" && a["action"] == "unchanged")
    );
    let uploads = h.writes().await;
    assert_eq!(uploads.len(), 1);
    assert_eq!(uploads[0].path, "/themes/mine/media");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn match_templates_posts_the_pairs() {
    let h = Harness::new().await;
    h.ok("POST", "/themes/mine/match-web-templates", json!(ID))
        .await;
    let bad = h
        .run(&["theme", "match-templates", "mine", "--map", "oops"])
        .await;
    assert_eq!(bad.error()["code"], "usage");
    assert!(h.writes().await.is_empty());
    h.run(&[
        "theme",
        "match-templates",
        "mine",
        "--map",
        "a=x,b=y",
        "--map-json",
        r#"{"c":"z"}"#,
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(w[0].path, "/themes/mine/match-web-templates");
    assert_eq!(
        w[0].body.as_ref().unwrap()["matchedWebTemplateDeveloperNames"],
        json!({"a": "x", "b": "y", "c": "z"})
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn web_template_validate_posts_the_content() {
    let h = Harness::new().await;
    h.ok("POST", "/webtemplates/validate", json!({"isValid": true}))
        .await;
    let out = h
        .run(&["web-template", "validate", "--content", "<p>{{ 1 }}</p>"])
        .await;
    assert_eq!(out.data()["valid"], true);
    let w = h.writes().await;
    assert_eq!(w[0].body.as_ref().unwrap()["content"], "<p>{{ 1 }}</p>");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn preview_returns_html_and_maps_render_failures() {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, ResponseTemplate};
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/webtemplates/theme/mine/template/page",
        json!({"id": ID}),
    )
    .await;
    let route = format!("{}/webtemplates/{ID}/render-preview", common::API);
    Mock::given(method("GET"))
        .and(path(route.clone()))
        .and(query_param("viewId", "V1"))
        .respond_with(
            ResponseTemplate::new(400)
                .insert_header("content-type", "application/problem+json")
                .set_body_json(json!({
                    "title": "Template render failed",
                    "status": 400,
                    "detail": "page: Invalid 'if' tag at (3:4)",
                    "line": 3,
                    "column": 4
                })),
        )
        .mount(&h.server)
        .await;
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html")
                .set_body_string("<html><head><title>Hi</title></head></html>"),
        )
        .mount(&h.server)
        .await;

    let ok = h.run(&["web-template", "preview", "mine", "page"]).await;
    assert_eq!(ok.data()["title"], "Hi");
    assert!(ok.data()["html"].as_str().unwrap().contains("<title>"));

    let bad = h
        .run(&["web-template", "preview", "mine", "page", "--view", "V1"])
        .await;
    assert_eq!(bad.code, 5);
    let e = bad.error();
    assert_eq!(e["code"], "validation_failed");
    assert!(e["message"].as_str().unwrap().starts_with("page:"));
    assert_eq!(
        (e["line"].as_u64(), e["column"].as_u64()),
        (Some(3), Some(4))
    );
    assert!(h.writes().await.is_empty());
    h.assert_contract().await;
}
