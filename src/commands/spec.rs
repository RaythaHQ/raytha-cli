//! `raytha spec`: the live OpenAPI document, for anything the curated commands do not cover.

use crate::client::Client;
use crate::error::Result;
use clap::Args;
use serde_json::{Map, Value, json};

#[derive(Args, Debug)]
pub struct SpecArgs {
    /// One line per operation (method, path, operationId) instead of the full document.
    #[arg(long)]
    pub summary: bool,
    /// Keep only paths containing this text (case-insensitive), for example `sitepages`.
    #[arg(long, value_name = "TEXT")]
    pub path: Option<String>,
}

pub fn run(client: &Client, args: &SpecArgs) -> Result<Value> {
    let mut doc = client.get(&["swagger.json"], &[])?;
    let needle = args.path.as_ref().map(|s| s.to_lowercase());

    // The document also lists the admin SPA's private API; only /raytha/api/v1 is the public,
    // key-authenticated surface this CLI manages.
    if let Some(paths) = doc.get_mut("paths").and_then(Value::as_object_mut) {
        paths.retain(|p, _| p.starts_with("/raytha/api/v1"));
        if let Some(n) = &needle {
            paths.retain(|p, _| p.to_lowercase().contains(n));
        }
    }

    if !args.summary {
        return Ok(doc);
    }

    let mut ops = Vec::new();
    if let Some(paths) = doc.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            let Some(item) = item.as_object() else {
                continue;
            };
            for (method, op) in item {
                if !matches!(method.as_str(), "get" | "post" | "put" | "delete" | "patch") {
                    continue;
                }
                let mut o = Map::new();
                o.insert("method".into(), json!(method.to_uppercase()));
                o.insert("path".into(), json!(path));
                o.insert(
                    "operationId".into(),
                    op.get("operationId").cloned().unwrap_or(Value::Null),
                );
                ops.push(Value::Object(o));
            }
        }
    }
    Ok(json!({ "operations": ops }))
}
