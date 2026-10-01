//! The content model (content types, fields, choices, views) as one portable JSON document.

use crate::client::{Body, Client};
use crate::error::{CliError, Result};
use crate::input::{json_value, read_text};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Export every content type with its fields, choices and views as one document.
    ///
    /// Two exports of the same model are identical, so the file diffs and commits cleanly.
    /// `--summary` returns a short outline instead, small enough to keep in an agent's context.
    Export {
        /// Write the document to this file and return only a count.
        #[arg(long, value_name = "PATH", conflicts_with = "summary")]
        out: Option<PathBuf>,
        /// A one-line-per-field outline instead of the full document.
        #[arg(long)]
        summary: bool,
    },
    /// Create or update content types, fields, choices and views from an exported document.
    ///
    /// Matched by developer name; nothing is deleted and a field's type never changes. It applies
    /// completely or not at all. Always try `--dry-run` first. The file may be the raw document or
    /// the output of `raytha schema export`.
    Import {
        /// Schema file (`-` for stdin).
        file: String,
        /// Report what would change and apply nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::Export { out, summary } => {
            let doc = client.get(&["contenttypes", "export"], &[])?;
            if summary {
                return Ok(outline(&doc));
            }
            match out {
                Some(path) => {
                    let text = serde_json::to_string_pretty(&doc)? + "\n";
                    super::write_file(&path, &text)?;
                    let count = doc
                        .get("contentTypes")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);
                    Ok(json!({ "file": path.display().to_string(), "contentTypes": count }))
                }
                None => Ok(doc),
            }
        }
        Cmd::Import { file, dry_run } => {
            let mut doc = json_value(&if file == "-" {
                "-".to_string()
            } else {
                read_text(&file)?
            })?;
            // Accept the CLI's own `{ok, data}` envelope as well as the bare document.
            if let Some(inner) = doc.get("data").filter(|_| doc.get("ok").is_some()).cloned() {
                doc = inner;
            }
            if doc.get("contentTypes").is_none() {
                return Err(
                    CliError::usage("Not a schema document: no `contentTypes` array.")
                        .with_hint("Create one with `raytha schema export --out schema.json`."),
                );
            }
            let query = [("dryRun", dry_run.to_string())];
            client.call(
                Method::POST,
                &["contenttypes", "import"],
                &query,
                Body::Json(&doc),
            )
        }
    }
}

/// `type (primary): field:kind, ...` plus view routes. Enough to write templates against.
fn outline(doc: &Value) -> Value {
    let types: Vec<Value> = doc
        .get("contentTypes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|t| {
            let fields: Vec<String> = t
                .get("fields")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|f| {
                    let name = f
                        .get("developerName")
                        .and_then(Value::as_str)
                        .unwrap_or("?");
                    let kind = f.get("fieldType").and_then(Value::as_str).unwrap_or("?");
                    let mut s = format!("{name}:{kind}");
                    if let Some(rel) = f.get("relatedContentType").and_then(Value::as_str) {
                        s.push_str(&format!("->{rel}"));
                    }
                    let choices: Vec<&str> = f
                        .get("choices")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|c| c.get("developerName").and_then(Value::as_str))
                        .collect();
                    if !choices.is_empty() {
                        s.push_str(&format!("[{}]", choices.join("|")));
                    }
                    if f.get("isRequired").and_then(Value::as_bool) == Some(true) {
                        s.push('*');
                    }
                    s
                })
                .collect();
            let views: Vec<String> = t
                .get("views")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|v| {
                    let name = v
                        .get("developerName")
                        .and_then(Value::as_str)
                        .unwrap_or("?");
                    match v.get("routePath").and_then(Value::as_str) {
                        Some(p) if !p.is_empty() => format!("{name} -> /{p}"),
                        _ => name.to_string(),
                    }
                })
                .collect();
            json!({
                "developerName": t.get("developerName"),
                "primaryField": t.get("primaryField"),
                "route": t.get("defaultRouteTemplate"),
                "fields": fields,
                "views": views,
            })
        })
        .collect();
    json!({ "legend": "name:type, ->related type, [choices], * required", "contentTypes": types })
}
