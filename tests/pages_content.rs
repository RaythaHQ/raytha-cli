mod common;

use common::{Harness, ID};
use serde_json::{Value, json};

fn list(items: Value) -> Value {
    let n = items.as_array().map(|a| a.len()).unwrap_or(0);
    json!({ "items": items, "totalCount": n })
}

async fn mock_active_theme_with_template(h: &Harness, template: &str) {
    h.ok(
        "GET",
        "/themes",
        list(json!([
            {"developerName": "other", "isActive": false},
            {"developerName": "mine", "isActive": true}
        ])),
    )
    .await;
    h.ok(
        "GET",
        &format!("/webtemplates/theme/mine/template/{template}"),
        json!({"id": "TPLID", "developerName": template}),
    )
    .await;
}

fn page_json() -> Value {
    json!({
        "id": "PAGEID", "title": "About", "webTemplateId": "TPLID", "routePath": "about",
        "widgets": {"main": [
            {"id": "W1", "widgetType": "hero", "settingsJson": "{\"headline\":\"Hi\",\"minHeight\":400}"}
        ]},
        "publishedWidgets": {}
    })
}

async fn mock_page_routes(h: &Harness) {
    h.ok("POST", "/sitepages", json!("PAGEID")).await;
    h.ok("PUT", "/sitepages/PAGEID/settings", json!("PAGEID"))
        .await;
    h.ok("PUT", "/sitepages/PAGEID/widgets", json!("PAGEID"))
        .await;
    h.ok("PUT", "/sitepages/PAGEID/publish", json!("PAGEID"))
        .await;
    h.ok("PUT", "/sitepages/PAGEID/set-as-home-page", json!("PAGEID"))
        .await;
    h.ok("GET", "/sitepages/PAGEID", page_json()).await;
}

fn summary(w: &[common::Req]) -> Vec<String> {
    w.iter()
        .map(|r| format!("{} {}", r.method, r.path))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn site_page_create_builds_draft_saves_widgets_then_publishes() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "raytha_html_page_fullwidth").await;
    mock_page_routes(&h).await;
    let out = h
        .run(&[
            "site-page",
            "create",
            "--title",
            "About",
            "--template",
            "raytha_html_page_fullwidth",
            "--route-path",
            "about",
            "--home",
            "--sections",
            r#"{"main":[{"type":"hero","settings":{"headline":"Hi","minHeight":400}},{"widgetType":"cta","settings":{"headline":"Go"}}]}"#,
        ])
        .await;
    let d = out.data();
    // The page comes back with settings as objects, not JSON strings.
    assert_eq!(d["widgets"]["main"][0]["settings"]["headline"], "Hi");
    assert!(d["widgets"]["main"][0].get("settingsJson").is_none());

    let w = h.writes().await;
    assert_eq!(
        summary(&w),
        [
            "POST /sitepages",
            "PUT /sitepages/PAGEID/settings",
            "PUT /sitepages/PAGEID/widgets",
            "PUT /sitepages/PAGEID/publish",
            "PUT /sitepages/PAGEID/set-as-home-page"
        ]
    );
    let create = w[0].body.as_ref().unwrap();
    assert_eq!(create["saveAsDraft"], true);
    assert_eq!(create["templateId"], "TPLID");
    assert_eq!(w[1].body.as_ref().unwrap()["routePath"], "about");
    let widgets = w[2].body.as_ref().unwrap();
    assert_eq!(widgets["sectionName"], "main");
    let first = &widgets["widgets"][0];
    assert_eq!(first["widgetType"], "hero");
    assert_eq!(
        first["settingsJson"],
        "{\"headline\":\"Hi\",\"minHeight\":400}"
    );
    assert_eq!(
        (first["row"].as_i64(), first["columnSpan"].as_i64()),
        (Some(0), Some(12))
    );
    assert_eq!(widgets["widgets"][1]["row"], 1);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn site_page_create_draft_does_not_publish() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "raytha_html_home").await;
    mock_page_routes(&h).await;
    h.run(&[
        "site-page",
        "create",
        "--title",
        "Home",
        "--template",
        "raytha_html_home",
        "--draft",
        "--sections",
        r#"{"hero":[{"type":"hero","settings":{}}]}"#,
    ])
    .await
    .data();
    let s = summary(&h.writes().await);
    assert!(!s.iter().any(|l| l.ends_with("/publish")), "{s:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn site_page_create_without_template_is_a_usage_error_with_a_hint() {
    let h = Harness::new().await;
    let out = h.run(&["site-page", "create", "--title", "X"]).await;
    assert_eq!(out.code, 2);
    assert!(
        out.error()["hint"]
            .as_str()
            .unwrap()
            .contains("web-template list")
    );
    assert!(h.requests().await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_widgets_are_rejected_before_anything_is_sent() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "raytha_html_home").await;
    mock_page_routes(&h).await;
    let out = h
        .run(&[
            "site-page",
            "create",
            "--title",
            "Home",
            "--template",
            "raytha_html_home",
            "--sections",
            r#"{"hero":[{"settings":{"headline":"no type"}}]}"#,
        ])
        .await;
    assert_eq!(out.code, 2);
    assert!(
        out.error()["message"]
            .as_str()
            .unwrap()
            .contains("widgetType")
    );
    assert!(h.writes().await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn widget_edit_merge_overlays_onto_current_settings() {
    let h = Harness::new().await;
    h.ok("GET", "/sitepages/PAGEID", page_json()).await;
    h.ok("PUT", "/sitepages/PAGEID/widgets/W1", json!("W1"))
        .await;
    h.ok("PUT", "/sitepages/PAGEID/publish", json!("PAGEID"))
        .await;
    h.run(&[
        "site-page",
        "widgets",
        "edit",
        "PAGEID",
        "W1",
        "--section",
        "main",
        "--settings",
        r#"{"headline":"New"}"#,
        "--merge",
        "--publish",
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(w[0].path, "/sitepages/PAGEID/widgets/W1");
    let body = w[0].body.as_ref().unwrap();
    assert_eq!(body["sectionName"], "main");
    let merged: Value = serde_json::from_str(body["settingsJson"].as_str().unwrap()).unwrap();
    assert_eq!(merged, json!({"headline": "New", "minHeight": 400}));
    assert_eq!(w[1].path, "/sitepages/PAGEID/publish");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sections_replace_empties_sections_that_were_not_listed() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/sitepages/PAGEID",
        json!({"id": "PAGEID", "widgets": {"main": [], "sidebar": []}}),
    )
    .await;
    h.ok("PUT", "/sitepages/PAGEID/widgets", json!("PAGEID"))
        .await;
    h.run(&[
        "site-page",
        "sections",
        "PAGEID",
        "--sections",
        r#"{"main":[{"type":"wysiwyg","settings":{"content":"<p>x</p>"}}]}"#,
        "--replace",
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].body.as_ref().unwrap()["sectionName"], "main");
    assert_eq!(w[1].body.as_ref().unwrap()["sectionName"], "sidebar");
    assert_eq!(w[1].body.as_ref().unwrap()["widgets"], json!([]));
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn widget_delete_needs_yes_and_sends_the_section() {
    let h = Harness::new().await;
    h.ok("DELETE", "/sitepages/PAGEID/widgets/W1", json!("W1"))
        .await;
    let refused = h
        .run(&[
            "site-page",
            "widgets",
            "delete",
            "PAGEID",
            "W1",
            "--section",
            "main",
        ])
        .await;
    assert_eq!(refused.code, 2);
    h.run(&[
        "site-page",
        "widgets",
        "delete",
        "PAGEID",
        "W1",
        "--section",
        "main",
        "--yes",
    ])
    .await
    .data();
    let w = h.writes().await;
    assert_eq!(w.len(), 1);
    assert_eq!(
        w[0].query,
        [("sectionName".to_string(), "main".to_string())]
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn site_page_lifecycle_commands_hit_the_right_routes() {
    let h = Harness::new().await;
    for action in ["publish", "unpublish", "discard-draft", "set-as-home-page"] {
        h.ok(
            "PUT",
            &format!("/sitepages/PAGEID/{action}"),
            json!("PAGEID"),
        )
        .await;
    }
    h.ok("PUT", "/sitepages/PAGEID/settings", json!("PAGEID"))
        .await;
    h.ok(
        "GET",
        "/sitepages/widget-definitions",
        json!([{"widgetType": "hero"}]),
    )
    .await;
    for args in [
        ["site-page", "publish", "PAGEID"].as_slice(),
        &["site-page", "unpublish", "PAGEID"],
        &["site-page", "discard-draft", "PAGEID"],
        &["site-page", "set-home", "PAGEID"],
        &["site-page", "settings", "PAGEID", "--route-path", "team"],
        &["site-page", "widget-definitions"],
    ] {
        h.run(args).await.data();
    }
    h.assert_contract().await;
    assert_eq!(h.writes().await.len(), 5);
}

#[tokio::test(flavor = "multi_thread")]
async fn content_create_resolves_the_template_and_sends_fields() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "post_detail").await;
    h.ok("POST", "/contentitems/posts", json!("ITEMID")).await;
    let out = h
        .run(&[
            "content",
            "create",
            "posts",
            "--template",
            "post_detail",
            "--data",
            r#"{"title":"Hello","content":"<p>x</p>"}"#,
        ])
        .await;
    assert_eq!(out.data()["id"], "ITEMID");
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["saveAsDraft"], false);
    assert_eq!(body["templateId"], "TPLID");
    assert_eq!(body["content"]["title"], "Hello");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_create_rejects_non_object_data() {
    let h = Harness::new().await;
    let out = h
        .run(&["content", "create", "posts", "--data", "[1]"])
        .await;
    assert_eq!(out.code, 2);
    assert!(h.requests().await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn content_edit_merge_starts_from_the_draft_when_there_is_one() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/contentitems/posts/ITEMID",
        json!({"id": "ITEMID", "isDraft": true,
               "draftContent": {"title": "Draft title", "summary": "s"},
               "publishedContent": {"title": "Live title", "summary": "old"}}),
    )
    .await;
    h.ok("PUT", "/contentitems/posts/ITEMID", json!("ITEMID"))
        .await;
    h.run(&[
        "content",
        "edit",
        "posts",
        "ITEMID",
        "--data",
        r#"{"summary":"new"}"#,
        "--merge",
    ])
    .await
    .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(body["saveAsDraft"], false);
    assert_eq!(
        body["content"],
        json!({"title": "Draft title", "summary": "new"})
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_publish_resaves_current_content_as_published() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/contentitems/posts/ITEMID",
        json!({"id": "ITEMID", "isDraft": true, "draftContent": {"title": "T"}}),
    )
    .await;
    h.ok("PUT", "/contentitems/posts/ITEMID", json!("ITEMID"))
        .await;
    h.run(&["content", "publish", "posts", "ITEMID"])
        .await
        .data();
    let body = h.writes().await[0].body.clone().unwrap();
    assert_eq!(
        body,
        json!({"saveAsDraft": false, "content": {"title": "T"}})
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn content_list_all_walks_every_page_and_passes_query_options() {
    let h = Harness::new().await;
    let items: Vec<Value> = (0..7).map(|i| json!({"id": format!("I{i}")})).collect();
    h.paged("/contentitems/posts", items).await;
    let out = h
        .run(&[
            "content",
            "list",
            "posts",
            "--all",
            "--page-size",
            "3",
            "--filter",
            "title eq 'x'",
            "--order-by",
            "CreationTime desc",
            "--search",
            "hello",
            "--view-id",
            "V1",
        ])
        .await;
    let d = out.data();
    assert_eq!(d["items"].as_array().unwrap().len(), 7);
    assert_eq!(d["hasMore"], false);
    let reqs = h.requests().await;
    assert_eq!(reqs.len(), 3);
    let q = &reqs[0].query;
    for (k, v) in [
        ("filter", "title eq 'x'"),
        ("orderBy", "CreationTime desc"),
        ("search", "hello"),
        ("viewId", "V1"),
    ] {
        assert!(
            q.contains(&(k.to_string(), v.to_string())),
            "{k} missing from {q:?}"
        );
    }
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_list_reports_has_more_for_a_single_page() {
    let h = Harness::new().await;
    let items: Vec<Value> = (0..5).map(|i| json!({"id": format!("I{i}")})).collect();
    h.paged("/contentitems/posts", items).await;
    let out = h
        .run(&["content", "list", "posts", "--page-size", "2"])
        .await;
    let d = out.data();
    assert_eq!(d["items"].as_array().unwrap().len(), 2);
    assert_eq!(
        (d["totalCount"].as_i64(), d["hasMore"].as_bool()),
        (Some(5), Some(true))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn content_trash_restore_purge_and_route_lookups() {
    let h = Harness::new().await;
    h.ok("DELETE", "/contentitems/posts/ITEMID", json!("ITEMID"))
        .await;
    h.ok("GET", "/contentitems/posts/trash", list(json!([])))
        .await;
    h.ok("PUT", "/contentitems/posts/ITEMID/restore", json!("ITEMID"))
        .await;
    h.ok(
        "DELETE",
        "/contentitems/posts/trash/ITEMID",
        json!("ITEMID"),
    )
    .await;
    h.ok(
        "PUT",
        "/contentitems/posts/ITEMID/unpublish",
        json!("ITEMID"),
    )
    .await;
    h.ok(
        "GET",
        "/contentitems/posts/route/blog/hello",
        json!({"id": "ITEMID"}),
    )
    .await;
    for args in [
        ["content", "delete", "posts", "ITEMID", "--yes"].as_slice(),
        &["content", "trash", "posts"],
        &["content", "restore", "posts", "ITEMID"],
        &["content", "purge", "posts", "ITEMID", "--yes"],
        &["content", "unpublish", "posts", "ITEMID"],
    ] {
        h.run(args).await.data();
    }
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn ids_may_start_with_a_hyphen() {
    let h = Harness::new().await;
    h.ok("DELETE", "/contentitems/posts/-ITEMID", json!("-ITEMID"))
        .await;
    h.run(&["content", "delete", "posts", "-ITEMID", "--yes"])
        .await
        .data();
}

#[tokio::test(flavor = "multi_thread")]
async fn content_type_create_and_field_commands() {
    let h = Harness::new().await;
    h.ok("POST", "/contenttypes", json!("CTID")).await;
    h.ok(
        "GET",
        "/contenttypes/posts",
        json!({"id": "CTID", "developerName": "posts", "primaryFieldId": "F1",
               "contentTypeFields": [
                 {"id": "F1", "developerName": "title", "label": "Title", "isRequired": true},
                 {"id": "F2", "developerName": "summary", "label": "Summary", "isRequired": false,
                  "description": "short"}]}),
    )
    .await;
    h.ok("POST", "/contenttypes/posts/fields", json!("F3"))
        .await;
    h.ok("PUT", "/contenttypes/posts/fields/F2", json!("F2"))
        .await;
    h.ok("DELETE", "/contenttypes/posts/fields/F2", json!("F2"))
        .await;
    h.ok("POST", "/contenttypes/posts/fields/F2/reorder", json!("F2"))
        .await;
    h.ok("PUT", "/contenttypes/posts", json!("CTID")).await;

    let out = h
        .run(&[
            "content-type",
            "create",
            "Blog Posts",
            "--label-plural",
            "Posts",
            "--label-singular",
            "Post",
        ])
        .await;
    assert_eq!(out.data()["developerName"], "blog_posts");
    h.run(&[
        "content-type",
        "fields",
        "create",
        "posts",
        "category",
        "--type",
        "dropdown",
        "--label",
        "Category",
        "--choices",
        "News, Release Notes",
    ])
    .await
    .data();
    h.run(&[
        "content-type",
        "fields",
        "edit",
        "posts",
        "summary",
        "--label",
        "Short summary",
    ])
    .await
    .data();
    h.run(&[
        "content-type",
        "fields",
        "reorder",
        "posts",
        "summary",
        "--position",
        "1",
    ])
    .await
    .data();
    let refused = h
        .run(&["content-type", "fields", "delete", "posts", "summary"])
        .await;
    assert_eq!(refused.code, 2);
    h.run(&[
        "content-type",
        "fields",
        "delete",
        "posts",
        "summary",
        "--yes",
    ])
    .await
    .data();
    h.run(&["content-type", "edit", "posts", "--primary-field", "title"])
        .await
        .data();

    let w = h.writes().await;
    let create = w[0].body.as_ref().unwrap();
    assert_eq!(create["developerName"], "blog_posts");
    assert_eq!(create["defaultRouteTemplate"], "{PrimaryField}");
    let field = w[1].body.as_ref().unwrap();
    assert_eq!(field["fieldType"], "dropdown");
    assert_eq!(
        field["choices"][1],
        json!({"label": "Release Notes", "developerName": "release_notes", "disabled": false})
    );
    let edit = w[2].body.as_ref().unwrap();
    assert_eq!(
        (edit["label"].as_str(), edit["description"].as_str()),
        (Some("Short summary"), Some("short"))
    );
    assert_eq!(w[3].body.as_ref().unwrap()["newFieldOrder"], 1);
    assert_eq!(w[5].body.as_ref().unwrap()["primaryFieldId"], "F1");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_field_names_list_the_known_ones() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/contenttypes/posts",
        json!({"id": "CTID", "contentTypeFields": [{"id": "F1", "developerName": "title"}]}),
    )
    .await;
    let out = h
        .run(&[
            "content-type",
            "fields",
            "edit",
            "posts",
            "nope",
            "--label",
            "X",
        ])
        .await;
    assert_eq!(out.code, 4);
    assert!(out.error()["hint"].as_str().unwrap().contains("title"));
}

#[tokio::test(flavor = "multi_thread")]
async fn view_commands_keep_current_settings_and_wrap_the_filter() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "raytha_html_content_item_list").await;
    h.ok(
        "GET",
        "/contenttypes/posts/views/V1",
        json!({"id": "V1", "isPublished": false, "routePath": "old",
               "defaultNumberOfItemsPerPage": 25, "maxNumberOfItemsPerPage": 80,
               "ignoreClientFilterAndSortQueryParams": false}),
    )
    .await;
    h.ok(
        "PUT",
        "/contenttypes/posts/views/V1/public-settings",
        json!("V1"),
    )
    .await;
    h.ok("PUT", "/contenttypes/posts/views/V1/filter", json!("V1"))
        .await;
    h.ok("PUT", "/contenttypes/posts/views/V1/sort", json!("V1"))
        .await;
    h.ok("POST", "/contenttypes/posts/views", json!("V2")).await;
    h.run(&[
        "content-type",
        "views",
        "settings",
        "posts",
        "V1",
        "--published",
        "--route-path",
        "blog",
        "--template",
        "raytha_html_content_item_list",
    ])
    .await
    .data();
    h.run(&[
        "content-type", "views", "filter", "posts", "V1", "--conditions",
        r#"[{"id":"1","type":"filter_condition","groupOperator":"AND","field":"title","conditionOperator":"contains","value":"x"}]"#,
    ])
    .await
    .data();
    h.run(&[
        "content-type",
        "views",
        "sort",
        "posts",
        "V1",
        "--data",
        r#"{"developerName":"title","showColumn":true,"orderByDirection":"asc"}"#,
    ])
    .await
    .data();
    h.run(&[
        "content-type",
        "views",
        "create",
        "posts",
        "featured",
        "--duplicate-from",
        "V1",
    ])
    .await
    .data();
    let w = h.writes().await;
    let s = w[0].body.as_ref().unwrap();
    assert_eq!(s["isPublished"], true);
    assert_eq!(s["routePath"], "blog");
    assert_eq!(s["templateId"], "TPLID");
    assert_eq!(
        (
            s["defaultNumberOfItemsPerPage"].as_i64(),
            s["maxNumberOfItemsPerPage"].as_i64()
        ),
        (Some(25), Some(80))
    );
    assert_eq!(w[1].body.as_ref().unwrap()["filter"][0]["field"], "title");
    assert_eq!(w[3].body.as_ref().unwrap()["duplicateFromId"], "V1");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn menu_items_carry_the_menu_id() {
    let h = Harness::new().await;
    h.ok(
        "GET",
        "/menus/main",
        json!({"id": "MENUID", "developerName": "main"}),
    )
    .await;
    h.ok("POST", "/menus/main/menu-items", json!("ITEM1")).await;
    h.ok("POST", "/menus/main/set-main", json!("MENUID")).await;
    h.run(&[
        "menu",
        "items",
        "create",
        "main",
        "--label",
        "About",
        "--link",
        "/about",
        "--new-tab",
    ])
    .await
    .data();
    h.run(&["menu", "set-main", "main"]).await.data();
    let w = h.writes().await;
    let b = w[0].body.as_ref().unwrap();
    assert_eq!(b["navigationMenuId"], "MENUID");
    assert_eq!(
        (b["label"].as_str(), b["url"].as_str()),
        (Some("About"), Some("/about"))
    );
    assert_eq!(b["openInNewTab"], true);
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn menu_item_flag_does_not_shadow_the_global_url_option() {
    let h = Harness::new().await;
    h.ok("GET", "/menus/main", json!({"id": "MENUID"})).await;
    h.ok("POST", "/menus/main/menu-items", json!("ITEM1")).await;
    let url = h.url();
    // --url here is the global site URL, placed after the subcommand.
    h.run_env(
        &[
            "menu", "items", "create", "main", "--label", "A", "--link", "/a", "--url", &url,
        ],
        &[("RAYTHA_URL", "http://127.0.0.1:1")],
    )
    .await
    .data();
}

#[tokio::test(flavor = "multi_thread")]
async fn media_upload_returns_key_and_url() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("photo.png");
    std::fs::write(&file, b"\x89PNG fake").unwrap();
    let h = Harness::new().await;
    h.ok("POST", "/mediaitems/upload-direct", json!("abc_photo.png"))
        .await;
    h.ok(
        "GET",
        "/mediaitems/abc_photo.png",
        json!("https://cdn.example.com/abc_photo.png"),
    )
    .await;
    let out = h.run(&["media", "upload", file.to_str().unwrap()]).await;
    let d = out.data();
    assert_eq!(d["objectKey"], "abc_photo.png");
    assert_eq!(d["url"], "https://cdn.example.com/abc_photo.png");
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_upload_file_is_a_usage_error() {
    let h = Harness::new().await;
    let out = h
        .run(&["media", "upload", "/definitely/not/here.png"])
        .await;
    assert!(out.code == 2 || out.code == 6, "{}", out.stdout);
    assert_eq!(out.json["ok"], false);
}

#[tokio::test(flavor = "multi_thread")]
async fn users_password_from_stdin_and_activation() {
    let h = Harness::new().await;
    h.ok("PUT", "/users/U1/password", json!("U1")).await;
    h.ok("PUT", "/users/U1/is-active", json!("U1")).await;
    h.run_stdin(&["user", "password", "U1", "--password-stdin"], "s3cret!\n")
        .await
        .data();
    h.run(&["user", "set-active", "U1", "--active", "false"])
        .await
        .data();
    let w = h.writes().await;
    assert_eq!(w[0].body.as_ref().unwrap()["newPassword"], "s3cret!");
    assert_eq!(w[0].body.as_ref().unwrap()["sendEmail"], false);
    assert_eq!(w[1].body.as_ref().unwrap()["isActive"], false);
    h.assert_contract().await;
    let _ = ID;
}

#[tokio::test(flavor = "multi_thread")]
async fn user_groups_crud_routes() {
    let h = Harness::new().await;
    h.ok("GET", "/usergroups", list(json!([]))).await;
    h.ok("POST", "/usergroups", json!("G1")).await;
    h.ok("PUT", "/usergroups/G1", json!("G1")).await;
    h.ok("DELETE", "/usergroups/G1", json!("G1")).await;
    h.run(&["user-group", "list"]).await.data();
    h.run(&["user-group", "create", "Staff Members"])
        .await
        .data();
    h.run(&["user-group", "edit", "G1", "--label", "Staff"])
        .await
        .data();
    h.run(&["user-group", "delete", "G1", "--yes"]).await.data();
    assert_eq!(
        h.writes().await[0].body.as_ref().unwrap()["developerName"],
        "staff_members"
    );
    h.assert_contract().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_create_defaults_to_the_builtin_detail_template() {
    let h = Harness::new().await;
    mock_active_theme_with_template(&h, "raytha_html_content_item_detail").await;
    h.ok("POST", "/contentitems/posts", json!("ITEMID")).await;
    h.run(&["content", "create", "posts", "--data", r#"{"title":"Hi"}"#])
        .await
        .data();
    assert_eq!(
        h.writes().await[0].body.as_ref().unwrap()["templateId"],
        "TPLID"
    );
    h.assert_contract().await;
}
