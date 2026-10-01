//! HTTP client for the Raytha v1 REST API.
//!
//! Responsibilities: auth header, URL building with safe path-segment encoding, retries for
//! idempotent reads, unwrapping Raytha's `{ success, result, error }` envelope, and translating
//! every failure into a `CliError` with a hint an agent can act on.

use crate::config::Config;
use crate::error::{CliError, EXIT_AUTH, EXIT_NOT_FOUND, EXIT_VALIDATION, Result};
use reqwest::blocking::{Client as Http, RequestBuilder, Response, multipart};
use reqwest::{Method, Url, redirect};
use serde_json::{Map, Value, json};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub enum Body<'a> {
    None,
    Json(&'a Value),
    File(&'a Path),
}

pub struct Client {
    http: Http,
    base: Url,
    key: String,
    pub site: String,
}

pub type Query<'a> = &'a [(&'a str, String)];

impl Client {
    pub fn new(cfg: &Config, timeout_secs: u64) -> Result<Self> {
        let http = Http::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(redirect::Policy::none())
            .user_agent(concat!("raytha-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| CliError::server("client_init", e.to_string()))?;
        let base = Url::parse(&format!("{}/raytha/api/v1/", cfg.url))
            .map_err(|e| CliError::config(format!("Invalid RAYTHA_URL: {e}")))?;
        Ok(Client {
            http,
            base,
            key: cfg.api_key.clone(),
            site: cfg.url.clone(),
        })
    }

    fn url(&self, segs: &[&str]) -> Url {
        let mut url = self.base.clone();
        {
            let mut p = url.path_segments_mut().expect("http base url");
            p.pop_if_empty();
            p.extend(segs);
        }
        url
    }

    pub fn get(&self, segs: &[&str], query: Query) -> Result<Value> {
        self.call(Method::GET, segs, query, Body::None)
    }

    pub fn post(&self, segs: &[&str], body: Option<&Value>) -> Result<Value> {
        self.call(
            Method::POST,
            segs,
            &[],
            body.map(Body::Json).unwrap_or(Body::None),
        )
    }

    pub fn put(&self, segs: &[&str], body: Option<&Value>) -> Result<Value> {
        self.call(
            Method::PUT,
            segs,
            &[],
            body.map(Body::Json).unwrap_or(Body::None),
        )
    }

    pub fn delete(&self, segs: &[&str], query: Query) -> Result<Value> {
        self.call(Method::DELETE, segs, query, Body::None)
    }

    /// DELETE with a JSON body (bulk deletes list the ids to remove).
    pub fn delete_json(&self, segs: &[&str], body: &Value) -> Result<Value> {
        self.call(Method::DELETE, segs, &[], Body::Json(body))
    }

    /// GET that returns a text body (HTML previews) instead of JSON. Errors map like any other call.
    pub fn get_text(&self, segs: &[&str], query: Query) -> Result<String> {
        let label = segs.join("/");
        let mut rb = self
            .request(Method::GET, segs)
            .header("Accept", "text/html");
        if !query.is_empty() {
            rb = rb.query(query);
        }
        let resp = rb.send().map_err(|e| self.network_error(e))?;
        let status = resp.status();
        let text = resp.text().map_err(|e| self.network_error(e))?;
        if status.is_success() {
            Ok(text)
        } else {
            Err(http_error(status.as_u16(), &text, &label))
        }
    }

    /// Fetches a public page exactly as a visitor would (no API key, no error mapping) and returns
    /// the status and body.
    pub fn fetch_public(&self, path: &str) -> Result<(u16, String)> {
        let url = format!("{}/{}", self.site, path.trim_start_matches('/'));
        let resp = self
            .http
            .get(&url)
            .header("Accept", "text/html")
            .send()
            .map_err(|e| self.network_error(e))?;
        let status = resp.status().as_u16();
        let body = resp.text().unwrap_or_default();
        Ok((status, body))
    }

    pub fn upload(&self, segs: &[&str], path: &Path) -> Result<Value> {
        self.call(Method::POST, segs, &[], Body::File(path))
    }

    /// Fetches an absolute URL without the API key (public media and file URLs).
    pub fn download(&self, url: &str) -> Result<Vec<u8>> {
        let resolved = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("{}/{}", self.site, url.trim_start_matches('/'))
        };
        let parsed = Url::parse(&resolved).map_err(|e| {
            CliError::server("bad_url", format!("Bad download URL '{resolved}': {e}"))
        })?;
        let same_host = Url::parse(&self.site)
            .map(|s| s.host_str() == parsed.host_str())
            .unwrap_or(false);
        let mut rb = self.http.get(parsed);
        if same_host {
            rb = rb.header("X-API-KEY", &self.key);
        }
        let resp = rb.send().map_err(|e| self.network_error(e))?;
        let mut resp = resp;
        // Media URLs are redirects to storage; follow them manually without sending the key.
        let mut hops = 0;
        while resp.status().is_redirection() && hops < 5 {
            let Some(loc) = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
            else {
                break;
            };
            let next = resp
                .url()
                .join(&loc)
                .map_err(|e| CliError::server("bad_url", e.to_string()))?;
            resp = self
                .http
                .get(next)
                .send()
                .map_err(|e| self.network_error(e))?;
            hops += 1;
        }
        if !resp.status().is_success() {
            return Err(CliError::server(
                "download_failed",
                format!("Downloading {resolved} returned HTTP {}.", resp.status()),
            )
            .with_status(resp.status().as_u16()));
        }
        resp.bytes()
            .map(|b| b.to_vec())
            .map_err(|e| self.network_error(e))
    }

    fn request(&self, method: Method, segs: &[&str]) -> RequestBuilder {
        self.http
            .request(method, self.url(segs))
            .header("X-API-KEY", &self.key)
            .header("Accept", "application/json")
    }

    pub fn call(&self, method: Method, segs: &[&str], query: Query, body: Body) -> Result<Value> {
        let label = segs.join("/");
        let attempts = if method == Method::GET { 3 } else { 1 };
        for attempt in 0..attempts {
            let last = attempt + 1 == attempts;
            let mut rb = self.request(method.clone(), segs);
            if !query.is_empty() {
                rb = rb.query(query);
            }
            rb = match &body {
                Body::None => rb,
                Body::Json(v) => rb.json(v),
                Body::File(p) => rb.multipart(file_form(p)?),
            };
            match rb.send() {
                Ok(resp) => {
                    if !last && matches!(resp.status().as_u16(), 429 | 502 | 503 | 504) {
                        sleep(retry_delay(&resp, attempt));
                        continue;
                    }
                    return self.finish(resp, &method, &label);
                }
                Err(e) => {
                    let err = self.network_error(e);
                    if !last {
                        sleep(Duration::from_millis(400 * (attempt as u64 + 1)));
                        continue;
                    }
                    return Err(err);
                }
            }
        }
        unreachable!("loop always returns")
    }

    fn network_error(&self, e: reqwest::Error) -> CliError {
        let (code, hint) = if e.is_timeout() {
            (
                "timeout",
                "The request timed out. Retry, or raise --timeout (seconds).".to_string(),
            )
        } else if e.is_connect() {
            (
                "connection_failed",
                format!(
                    "Could not connect to {}. Check RAYTHA_URL and that the site is running.",
                    self.site
                ),
            )
        } else {
            (
                "network_error",
                "Check connectivity and RAYTHA_URL.".to_string(),
            )
        };
        CliError::server(code, e.to_string()).with_hint(hint)
    }

    fn finish(&self, resp: Response, method: &Method, label: &str) -> Result<Value> {
        let status = resp.status();
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let text = resp.text().map_err(|e| self.network_error(e))?;

        if status.is_redirection() {
            let target = location.unwrap_or_else(|| "(unknown)".into());
            return Err(CliError::server(
                "redirected",
                format!("The site answered with a redirect to {target}."),
            )
            .with_status(status.as_u16())
            .with_hint(
                "Raytha may be configured to redirect to another website, or RAYTHA_URL uses http \
                 where the site requires https (or the reverse). Set RAYTHA_URL to the final URL.",
            ));
        }

        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(Value::Null);
            }
            let value: Value = serde_json::from_str(&text).map_err(|_| {
                CliError::server(
                    "not_raytha",
                    "The response was not JSON, so this may not be a Raytha API.",
                )
                .with_status(status.as_u16())
                .with_hint("Check RAYTHA_URL points at the Raytha site root.")
            })?;
            return unwrap_envelope(value, method);
        }

        Err(http_error(status.as_u16(), &text, label))
    }
}

fn retry_delay(resp: &Response, attempt: u32) -> Duration {
    let from_header = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    match from_header {
        Some(s) => Duration::from_secs(s.min(5)),
        None => Duration::from_millis(500 * (attempt as u64 + 1)),
    }
}

fn file_form(path: &Path) -> Result<multipart::Form> {
    let data = std::fs::read(path)
        .map_err(|e| CliError::usage(format!("Cannot read {}: {e}", path.display())))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload")
        .to_string();
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    let part = multipart::Part::bytes(data)
        .file_name(name)
        .mime_str(&mime)
        .map_err(|e| CliError::usage(e.to_string()))?;
    Ok(multipart::Form::new().part("file", part))
}

/// Raytha wraps results as `{ success, result, error }`. Return `result`; turn a string id from a
/// mutating call into `{ "id": ... }` so every command returns an object.
pub fn unwrap_envelope(value: Value, method: &Method) -> Result<Value> {
    let is_envelope = value
        .as_object()
        .map(|o| o.contains_key("success") && o.contains_key("result"))
        .unwrap_or(false);
    if !is_envelope {
        return Ok(value);
    }
    let success = value
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if !success {
        let msg = value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Request failed.");
        return Err(CliError::validation(msg));
    }
    let result = value.get("result").cloned().unwrap_or(Value::Null);
    if *method != Method::GET
        && let Value::String(id) = &result
    {
        return Ok(json!({ "id": id }));
    }
    Ok(result)
}

fn permission_for(seg: &str) -> &'static str {
    match seg {
        "themes" | "webtemplates" | "widgettemplates" => "ManageTemplates",
        "sitepages" => "ManageSitePages",
        "contenttypes" | "menus" => "ManageContentTypes",
        "contentitems" => "read/edit permission on that content type",
        "mediaitems" => "ManageMediaItems / UploadMediaItems",
        "users" | "usergroups" => "ManageUsers",
        _ => "the required",
    }
}

fn list_command_for(seg: &str) -> Option<&'static str> {
    Some(match seg {
        "themes" => "raytha theme list",
        "webtemplates" => "raytha web-template list --theme <theme>",
        "widgettemplates" => "raytha widget-template list --theme <theme>",
        "sitepages" => "raytha site-page list",
        "contenttypes" => "raytha content-type list",
        "contentitems" => "raytha content list <content-type>",
        "mediaitems" => "raytha media list",
        "menus" => "raytha menu list",
        "users" => "raytha user list",
        "usergroups" => "raytha user-group list",
        _ => return None,
    })
}

/// Turns an error response (RFC 7807 problem details or Raytha's legacy shape) into a `CliError`.
pub fn http_error(status: u16, body: &str, label: &str) -> CliError {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let obj = parsed.as_ref().and_then(Value::as_object);
    let get_str = |k: &str| {
        obj.and_then(|o| o.get(k))
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|s| !s.is_empty())
    };
    let message = get_str("detail")
        .or_else(|| get_str("error"))
        .or_else(|| get_str("title"))
        .or_else(|| get_str("message"))
        .unwrap_or_else(|| format!("HTTP {status}"));
    let fields = obj
        .and_then(|o| o.get("errors"))
        .filter(|v| v.as_object().map(|m| !m.is_empty()).unwrap_or(false))
        .cloned();
    let seg0 = label.split('/').next().unwrap_or("").to_lowercase();

    let mut err = match status {
        401 => CliError::new("unauthorized", message, EXIT_AUTH).with_hint(
            "The API key was rejected. Check RAYTHA_API_KEY and RAYTHA_URL. Keys are shown once \
             at creation; generate a new one under Settings > Administrators > (admin) > API Keys.",
        ),
        403 => CliError::new("forbidden", message, EXIT_AUTH).with_hint(format!(
            "The API key's administrator lacks the {} permission. API keys inherit the \
             permissions of their admin; grant the role that permission (Settings > Roles) or \
             use a key from a more privileged admin. Run `raytha doctor` to see what this key can do.",
            permission_for(&seg0)
        )),
        404 => {
            let mut e = CliError::new("not_found", message, EXIT_NOT_FOUND);
            if let Some(cmd) = list_command_for(&seg0) {
                e = e.with_hint(format!(
                    "Nothing matched '/{label}'. List what exists with `{cmd}`."
                ));
            }
            e
        }
        400 => {
            let mut e = CliError::new("validation_failed", message, EXIT_VALIDATION);
            e = e.with_hint(if fields.is_some() {
                "Fix the fields listed in error.fields and retry."
            } else {
                "Read error.message, adjust the request, and retry. `raytha guide errors` lists common causes."
            });
            e
        }
        422 => CliError::new("invalid_identifier", message, EXIT_VALIDATION).with_hint(
            "An id was malformed. Ids are 22-character strings returned by list/get commands; \
             copy them exactly.",
        ),
        409 => CliError::new("conflict", message, EXIT_VALIDATION),
        413 => CliError::new("payload_too_large", message, EXIT_VALIDATION)
            .with_hint("The upload exceeds the server's size limit."),
        429 => CliError::server("rate_limited", message)
            .with_hint("Slow down and retry in a few seconds."),
        s if s >= 500 => CliError::server("server_error", message)
            .with_hint("Raytha failed while handling the request. Retry once; if it persists the server logs have the cause."),
        _ => CliError::new("request_failed", message, EXIT_VALIDATION),
    };
    if let Some(f) = fields {
        err = err.with_fields(normalize_fields(f));
    }
    let num = |k: &str| obj.and_then(|o| o.get(k)).and_then(Value::as_u64);
    if let (Some(line), Some(column)) = (num("line"), num("column")) {
        err = err.with_location(line, column);
    } else if let Some((line, column)) = position_prefix(&err.message) {
        err = err.with_location(line, column);
    }
    err.with_status(status)
}

/// Save-time template checks fold the position into the message as `Line 1, column 9: ...`.
fn position_prefix(message: &str) -> Option<(u64, u64)> {
    let rest = message.strip_prefix("Line ")?;
    let (line, rest) = rest.split_once(", column ")?;
    let (column, _) = rest.split_once(':')?;
    Some((line.trim().parse().ok()?, column.trim().parse().ok()?))
}

/// Keeps ASP.NET's `{ "Field": ["msg"] }` shape but camelCases nothing: agents see exactly the
/// names Raytha reports (for example `Widgets[0].SettingsJson`).
fn normalize_fields(fields: Value) -> Value {
    match fields {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_validation_problem_details() {
        let body = r#"{"title":"One or more validation errors occurred.","status":400,
            "errors":{"Title":["'Title' must not be empty."]},"detail":"Validation failed"}"#;
        let e = http_error(400, body, "themes");
        assert_eq!(e.code, "validation_failed");
        assert_eq!(e.exit, EXIT_VALIDATION);
        assert!(e.fields.unwrap()["Title"].is_array());
    }

    #[test]
    fn parser_position_becomes_line_and_column() {
        let body = r#"{"success":false,"error":"Invalid 'if' tag at (1:6)","line":1,"column":6}"#;
        let j = http_error(400, body, "webtemplates/validate").to_json();
        assert_eq!(j["error"]["line"], 1);
        assert_eq!(j["error"]["column"], 6);
        let plain = http_error(400, r#"{"error":"x"}"#, "webtemplates").to_json();
        assert!(plain["error"].get("line").is_none());
    }

    #[test]
    fn save_time_position_prefix_is_parsed() {
        let body = r#"{"success":false,"error":"Line 2, column 9: Invalid 'if' tag"}"#;
        let j = http_error(400, body, "webtemplates/theme/x").to_json();
        assert_eq!(
            (j["error"]["line"].clone(), j["error"]["column"].clone()),
            (json!(2), json!(9))
        );
    }

    #[test]
    fn forbidden_names_permission() {
        let e = http_error(403, r#"{"title":"Forbidden","detail":"nope"}"#, "themes/x");
        assert_eq!(e.exit, EXIT_AUTH);
        assert!(e.hint.unwrap().contains("ManageTemplates"));
    }

    #[test]
    fn not_found_suggests_list_command() {
        let e = http_error(404, "", "sitepages/abc");
        assert_eq!(e.exit, EXIT_NOT_FOUND);
        assert!(e.hint.unwrap().contains("raytha site-page list"));
    }

    #[test]
    fn unwraps_result_and_ids() {
        let v = json!({"success": true, "result": {"a": 1}});
        assert_eq!(unwrap_envelope(v, &Method::GET).unwrap(), json!({"a": 1}));
        let v = json!({"success": true, "result": "abc"});
        assert_eq!(
            unwrap_envelope(v, &Method::POST).unwrap(),
            json!({"id": "abc"})
        );
        let ping = json!({"success": true, "version": "1.0"});
        assert_eq!(unwrap_envelope(ping.clone(), &Method::GET).unwrap(), ping);
    }
}
