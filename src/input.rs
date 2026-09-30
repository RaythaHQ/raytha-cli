//! Input helpers. Large or structured values always have a file/stdin path so agents never have to
//! shell-escape JSON or liquid.

use crate::error::{CliError, Result};
use clap::Args;
use serde_json::{Map, Value};
use std::io::Read;

/// Reads text from a file path, or from stdin when `src` is `-`.
pub fn read_text(src: &str) -> Result<String> {
    if src == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| CliError::usage(format!("Cannot read stdin: {e}")))?;
        return Ok(s);
    }
    std::fs::read_to_string(src)
        .map_err(|e| CliError::usage(format!("Cannot read file '{src}': {e}")))
}

/// Interprets a value that is inline JSON, `@path` (read the file), or `-` (read stdin).
pub fn json_value(arg: &str) -> Result<Value> {
    let trimmed = arg.trim();
    let text = if trimmed == "-" {
        read_text("-")?
    } else if let Some(path) = trimmed.strip_prefix('@') {
        read_text(path)?
    } else {
        trimmed.to_string()
    };
    Ok(serde_json::from_str(&text)?)
}

/// JSON body input shared by commands that take structured data.
#[derive(Args, Clone, Default, Debug)]
pub struct BodyArgs {
    /// JSON inline, `@path` to read a file, or `-` for stdin.
    #[arg(long, value_name = "JSON|@PATH|-")]
    pub data: Option<String>,
    /// Path to a JSON file (`-` for stdin). Same as `--data @PATH`.
    #[arg(long, value_name = "PATH")]
    pub file: Option<String>,
}

impl BodyArgs {
    pub fn read(&self) -> Result<Option<Value>> {
        match (&self.data, &self.file) {
            (Some(_), Some(_)) => Err(CliError::usage("Use either --data or --file, not both.")),
            (Some(d), None) => Ok(Some(json_value(d)?)),
            (None, Some(f)) => Ok(Some(json_value(&if f == "-" {
                "-".to_string()
            } else {
                format!("@{f}")
            })?)),
            (None, None) => Ok(None),
        }
    }

    pub fn require_object(&self, what: &str) -> Result<Map<String, Value>> {
        match self.read()? {
            Some(Value::Object(m)) => Ok(m),
            Some(_) => Err(CliError::usage(format!("{what} must be a JSON object."))),
            None => Err(CliError::usage(format!(
                "{what} is required: pass --data '<json>' or --file <path>."
            ))),
        }
    }
}

/// Text content (liquid, markup) from `--file` or `--content`.
#[derive(Args, Clone, Default, Debug)]
pub struct ContentSrc {
    /// Path to the template file (`-` for stdin).
    #[arg(long, value_name = "PATH")]
    pub file: Option<String>,
    /// Template content inline. Prefer --file for anything longer than a line.
    #[arg(long)]
    pub content: Option<String>,
}

impl ContentSrc {
    pub fn read(&self) -> Result<Option<String>> {
        match (&self.file, &self.content) {
            (Some(_), Some(_)) => Err(CliError::usage("Use either --file or --content, not both.")),
            (Some(f), None) => Ok(Some(read_text(f)?)),
            (None, Some(c)) => Ok(Some(c.clone())),
            (None, None) => Ok(None),
        }
    }
}

/// Drops nulls and default-looking values (`false`, `""`, `[]`, `{}`) from objects, recursively, so
/// a server DTO that spells out every default compares equal to a hand-written definition that
/// omits them.
pub fn strip_empty(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), strip_empty(v)))
                .filter(|(_, v)| !is_default(v))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(strip_empty).collect()),
        other => other.clone(),
    }
}

fn is_default(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => true,
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(m) => m.is_empty(),
        _ => false,
    }
}

/// `My Cool Theme` -> `my_cool_theme`, matching Raytha's developer-name normalization closely
/// enough for client-side defaults. The server response is always the source of truth.
pub fn to_developer_name(s: &str) -> String {
    let mut out = String::new();
    let mut last_underscore = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_underscore = false;
        } else if !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    out.trim_end_matches('_').to_string()
}

/// `raytha_html_base_layout` -> `Raytha Html Base Layout`.
pub fn humanize(dev: &str) -> String {
    dev.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn developer_names() {
        assert_eq!(to_developer_name("My Cool Theme!"), "my_cool_theme");
        assert_eq!(
            humanize("raytha_html_base_layout"),
            "Raytha Html Base Layout"
        );
    }

    #[test]
    fn inline_json_parses() {
        assert_eq!(json_value(r#"{"a":1}"#).unwrap(), json!({"a": 1}));
        assert!(json_value("{oops").is_err());
    }

    #[test]
    fn nulls_are_stripped() {
        assert_eq!(
            strip_empty(
                &json!({"a": null, "b": [{"c": null, "d": 1, "e": false, "f": [], "g": ""}]})
            ),
            json!({"b": [{"d": 1}]})
        );
    }
}
