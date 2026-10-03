//! Shared test harness: a wiremock server standing in for Raytha, the compiled `raytha` binary
//! run against it, and a contract check that every request the CLI sent is a real operation in
//! the committed OpenAPI snapshot (`tests/fixtures/openapi-v1.json`), including JSON body keys.
#![allow(dead_code)]

use serde_json::{Value, json};
use std::process::Command;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const API: &str = "/raytha/api/v1";

pub struct Harness {
    pub server: MockServer,
}

pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub json: Value,
}

impl Out {
    pub fn data(&self) -> &Value {
        assert_eq!(
            self.json["ok"],
            json!(true),
            "expected success, got: {}",
            self.stdout
        );
        &self.json["data"]
    }

    pub fn error(&self) -> &Value {
        assert_eq!(
            self.json["ok"],
            json!(false),
            "expected failure, got: {}",
            self.stdout
        );
        &self.json["error"]
    }
}

#[derive(Debug, Clone)]
pub struct Req {
    pub method: String,
    /// Path below `/raytha/api/v1`, for example `/themes/default`.
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
}

pub fn envelope(result: Value) -> Value {
    json!({ "success": true, "result": result })
}

impl Harness {
    pub async fn new() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }

    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Answers `method path` (relative to the API root) with a success envelope.
    pub async fn ok(&self, http_method: &str, rel: &str, result: Value) {
        self.respond(http_method, rel, 200, envelope(result)).await;
    }

    /// Serves `items` as a paged list (`pageNumber`/`pageSize` honoured) at `GET rel`.
    pub async fn paged(&self, rel: &str, items: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path(format!("{API}{rel}")))
            .respond_with(move |req: &wiremock::Request| {
                let q: std::collections::HashMap<String, String> = req
                    .url
                    .query_pairs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                let page: usize = q
                    .get("pageNumber")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                let size: usize = q.get("pageSize").and_then(|v| v.parse().ok()).unwrap_or(50);
                let slice: Vec<Value> = items
                    .iter()
                    .skip((page - 1) * size)
                    .take(size)
                    .cloned()
                    .collect();
                ResponseTemplate::new(200).set_body_json(envelope(
                    json!({ "items": slice, "totalCount": items.len() }),
                ))
            })
            .mount(&self.server)
            .await;
    }

    pub async fn respond(&self, http_method: &str, rel: &str, status: u16, body: Value) {
        Mock::given(method(http_method))
            .and(path(format!("{API}{rel}")))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .mount(&self.server)
            .await;
    }

    /// Runs the binary with the harness URL and a dummy key. Extra env pairs override.
    pub async fn run(&self, args: &[&str]) -> Out {
        self.run_env(args, &[]).await
    }

    pub async fn run_env(&self, args: &[&str], env: &[(&str, &str)]) -> Out {
        self.run_full(args, env, None).await
    }

    pub async fn run_stdin(&self, args: &[&str], stdin: &str) -> Out {
        self.run_full(args, &[], Some(stdin.to_string())).await
    }

    async fn run_full(&self, args: &[&str], env: &[(&str, &str)], stdin: Option<String>) -> Out {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let env: Vec<(String, String)> = env
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let url = self.url();
        tokio::task::spawn_blocking(move || run_binary(&args, &url, &env, stdin))
            .await
            .expect("binary task")
    }

    /// Requests that hit the API, in order, with the API prefix stripped.
    pub async fn requests(&self) -> Vec<Req> {
        self.server
            .received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.url.path().starts_with(API))
            .map(|r| Req {
                method: r.method.to_string(),
                path: r.url.path()[API.len()..].to_string(),
                query: r
                    .url
                    .query_pairs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                body: serde_json::from_slice(&r.body).ok(),
            })
            .collect()
    }

    /// Writes only (POST/PUT/DELETE), which is what most tests assert on.
    pub async fn writes(&self) -> Vec<Req> {
        self.requests()
            .await
            .into_iter()
            .filter(|r| r.method != "GET")
            .collect()
    }

    /// Fails if any request the CLI made is not an operation in the OpenAPI snapshot, used a query
    /// parameter the operation does not declare, or sent a JSON body key the operation's request
    /// schema does not list. Body schemas have been distinct per command since Raytha 2.0.0.
    pub async fn assert_contract(&self) {
        let spec = spec_paths();
        for r in self.requests().await {
            let op = spec
                .iter()
                .find(|(tpl, methods)| {
                    template_matches(tpl, &r.path) && methods.contains_key(&r.method.to_lowercase())
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} {} is not an operation in tests/fixtures/openapi-v1.json",
                        r.method, r.path
                    )
                });
            let operation = &op.1[&r.method.to_lowercase()];
            let declared: Vec<String> = operation["parameters"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter(|p| p["in"] == "query")
                        .filter_map(|p| p["name"].as_str().map(str::to_lowercase))
                        .collect()
                })
                .unwrap_or_default();
            for (k, _) in &r.query {
                assert!(
                    declared.contains(&k.to_lowercase()),
                    "{} {}: query parameter '{k}' is not declared (declared: {declared:?})",
                    r.method,
                    r.path
                );
            }
            let Some(keys) = r.body.as_ref().and_then(Value::as_object) else {
                continue;
            };
            if keys.is_empty() {
                continue;
            }
            let body = operation.get("requestBody");
            let props: Vec<String> = body
                .and_then(|b| b.get("properties"))
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|p| p.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let json_body = body
                .and_then(|b| b.get("contentType"))
                .and_then(Value::as_str)
                == Some("application/json");
            assert!(
                json_body && !props.is_empty(),
                "{} {} sent a JSON body but the spec declares no JSON request body (keys: {:?})",
                r.method,
                r.path,
                keys.keys().collect::<Vec<_>>()
            );
            for k in keys.keys() {
                assert!(
                    props.iter().any(|p| p == k),
                    "{} {}: body property '{k}' is not in the request schema (declared: {props:?})",
                    r.method,
                    r.path
                );
            }
        }
    }
}

fn run_binary(args: &[String], url: &str, env: &[(String, String)], stdin: Option<String>) -> Out {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_raytha"));
    cmd.args(args)
        .env_remove("RAYTHA_URL")
        .env_remove("RAYTHA_API_KEY")
        .env("RAYTHA_URL", url)
        .env("RAYTHA_API_KEY", "test-key");
    for (k, v) in env {
        cmd.env(k, v);
    }
    use std::io::Write;
    use std::process::Stdio;
    cmd.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn raytha");
    if let Some(text) = stdin {
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(text.as_bytes())
            .expect("write stdin");
    }
    let output = child.wait_with_output().expect("run raytha");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let json = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    Out {
        code: output.status.code().unwrap_or(-1),
        stdout,
        json,
    }
}

fn spec_paths() -> Vec<(String, serde_json::Map<String, Value>)> {
    let raw = include_str!("../fixtures/openapi-v1.json");
    let v: Value = serde_json::from_str(raw).expect("fixture json");
    v["paths"]
        .as_object()
        .expect("paths")
        .iter()
        .map(|(p, ops)| {
            (
                p.trim_start_matches(API).to_lowercase(),
                ops.as_object().expect("ops").clone(),
            )
        })
        .collect()
}

/// `/themes/{name}/media` matches `/themes/foo/media`, case-insensitively.
fn template_matches(template: &str, actual: &str) -> bool {
    let t: Vec<&str> = template.trim_matches('/').split('/').collect();
    let a: Vec<String> = actual
        .trim_matches('/')
        .split('/')
        .map(str::to_lowercase)
        .collect();
    t.len() == a.len()
        && t.iter()
            .zip(&a)
            .all(|(t, a)| (t.starts_with('{') && t.ends_with('}')) || t == a)
}

/// A URL-safe stand-in for a ShortGuid.
pub const ID: &str = "AbCdEfGhIjKlMnOpQrStUv";
