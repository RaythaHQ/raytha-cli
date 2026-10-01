//! Batch creation of content items (`content import`) and `@file:` attachment references.

use super::task;
use super::{resolve_template, web_template_id};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::read_text;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Raytha accepts at most this many items in one batch request.
const BATCH_SIZE: usize = 500;
const FILE_PREFIX: &str = "@file:";

pub struct ImportArgs {
    pub content_type: String,
    pub file: String,
    pub template: Option<String>,
    pub template_id: Option<String>,
    pub draft: bool,
    pub wait_timeout: u64,
}

pub fn run(client: &Client, args: ImportArgs) -> Result<Value> {
    let text = read_text(&args.file)?;
    let mut rows = parse_rows(&text)?;
    if rows.is_empty() {
        return Err(CliError::usage("The file holds no items."));
    }
    let base = if args.file == "-" {
        PathBuf::from(".")
    } else {
        Path::new(&args.file)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };
    let mut uploads = HashMap::new();
    for (i, row) in rows.iter_mut().enumerate() {
        resolve_files_cached(client, row, &base, &mut uploads)
            .map_err(|e| CliError::new(&e.code, format!("Item {i}: {}", e.message), e.exit))?;
    }

    let template_id = match resolve_template(
        client,
        args.template.as_deref(),
        args.template_id.as_deref(),
        None,
    )? {
        Some(t) => t,
        None => web_template_id(client, None, super::content::DEFAULT_DETAIL_TEMPLATE)?,
    };

    let mut results: Vec<Value> = Vec::new();
    let mut created = 0u64;
    let mut failed = 0u64;
    for (n, chunk) in rows.chunks(BATCH_SIZE).enumerate() {
        let offset = n * BATCH_SIZE;
        let items: Vec<Value> = chunk
            .iter()
            .map(|c| json!({ "content": c, "saveAsDraft": args.draft }))
            .collect();
        let started = client.post(
            &["contentitems", &args.content_type, "batch"],
            Some(&json!({ "templateId": template_id, "items": items })),
        )?;
        let id = started
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CliError::server("unexpected_response", "Batch returned no task id."))?
            .to_string();
        let done = task::wait_raw(client, &id, args.wait_timeout)?;
        let info = done.get("statusInfo").and_then(Value::as_str).unwrap_or("");
        let parsed: Value = serde_json::from_str(info).map_err(|_| {
            CliError::server(
                "unexpected_response",
                format!("Batch task {id} finished without a result: {info}"),
            )
        })?;
        created += parsed.get("created").and_then(Value::as_u64).unwrap_or(0);
        failed += parsed.get("failed").and_then(Value::as_u64).unwrap_or(0);
        for item in parsed
            .get("items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let mut item = item.clone();
            let idx = item.get("index").and_then(Value::as_u64).unwrap_or(0) as usize + offset;
            item["index"] = json!(idx);
            results.push(item);
        }
    }

    let report = json!({
        "total": rows.len(),
        "created": created,
        "failed": failed,
        "items": results,
    });
    if failed > 0 {
        return Err(CliError::validation(format!(
            "{failed} of {} item(s) were rejected; the other {created} were created.",
            rows.len()
        ))
        .with_hint("See error.fields.report.items for each failed row's `index` and `errors`; fix those rows and import only them again.")
        .with_fields(json!({ "report": report })));
    }
    Ok(report)
}

/// A JSON array of objects, or JSON Lines (one object per line).
pub fn parse_rows(text: &str) -> Result<Vec<Value>> {
    let t = text.trim();
    let rows: Vec<Value> = if t.starts_with('[') {
        serde_json::from_str(t)?
    } else {
        t.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .enumerate()
            .map(|(i, l)| {
                serde_json::from_str(l)
                    .map_err(|e| CliError::usage(format!("Line {} is not valid JSON: {e}", i + 1)))
            })
            .collect::<Result<_>>()?
    };
    for (i, r) in rows.iter().enumerate() {
        if !r.is_object() {
            return Err(CliError::usage(format!(
                "Item {i} is not a JSON object of field values."
            )));
        }
    }
    Ok(rows)
}

/// Replaces every string value `@file:<path>` with the object key of that file after uploading it.
/// Relative paths are resolved against `base`. Works for nested values (repeaters).
pub fn resolve_files(client: &Client, value: &mut Value, base: &Path) -> Result<()> {
    resolve_files_cached(client, value, base, &mut HashMap::new())
}

fn resolve_files_cached(
    client: &Client,
    value: &mut Value,
    base: &Path,
    uploads: &mut HashMap<PathBuf, String>,
) -> Result<()> {
    match value {
        Value::String(s) => {
            if let Some(rel) = s.strip_prefix(FILE_PREFIX) {
                let path = base.join(rel.trim());
                let key = match uploads.get(&path) {
                    Some(k) => k.clone(),
                    None => {
                        let k = super::media::upload_key(client, &path)?;
                        uploads.insert(path, k.clone());
                        k
                    }
                };
                *s = key;
            }
        }
        Value::Array(a) => {
            for v in a {
                resolve_files_cached(client, v, base, uploads)?;
            }
        }
        Value::Object(o) => {
            for v in o.values_mut() {
                resolve_files_cached(client, v, base, uploads)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn has_file_refs(value: &Value) -> bool {
    match value {
        Value::String(s) => s.starts_with(FILE_PREFIX),
        Value::Array(a) => a.iter().any(has_file_refs),
        Value::Object(o) => o.values().any(has_file_refs),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_parse_from_array_or_jsonl() {
        assert_eq!(parse_rows(r#"[{"a":1},{"a":2}]"#).unwrap().len(), 2);
        assert_eq!(parse_rows("{\"a\":1}\n\n{\"a\":2}\n").unwrap().len(), 2);
        assert!(parse_rows("[1]").is_err());
        assert!(parse_rows("{\"a\":1}\nnope").is_err());
    }

    #[test]
    fn finds_nested_file_refs() {
        assert!(has_file_refs(&json!({"a": [{"b": "@file:x.png"}]})));
        assert!(!has_file_refs(&json!({"a": "plain"})));
    }
}
