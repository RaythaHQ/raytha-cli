//! Widget templates: the reusable building blocks placed on site pages. Each has a liquid
//! template and a settings form (`fields`).

use super::{ListArgs, list, require_yes, str_of, write_file};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{ContentSrc, humanize, json_value};
use clap::Subcommand;
use serde_json::{Map, Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List widget templates, optionally for one theme.
    List {
        #[arg(long)]
        theme: Option<String>,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Get one widget template, including liquid content and field definitions.
    Get {
        theme: String,
        name: String,
        /// Write the liquid content to this file instead of returning it inline.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Create a widget template from a liquid file and a field list.
    Create {
        theme: String,
        /// Developer name (cannot match a built-in widget).
        name: String,
        #[arg(long)]
        label: Option<String>,
        #[command(flatten)]
        src: ContentSrc,
        /// Field definitions: JSON array inline, `@path`, or `-`. See `raytha guide widgets`.
        #[arg(long, value_name = "JSON|@PATH|-")]
        fields: Option<String>,
    },
    /// Edit a widget template. Omitted flags keep their current value.
    Edit {
        theme: String,
        name: String,
        #[arg(long)]
        label: Option<String>,
        #[command(flatten)]
        src: ContentSrc,
        /// Replacement field definitions (JSON array inline, `@path`, or `-`).
        #[arg(long, value_name = "JSON|@PATH|-")]
        fields: Option<String>,
    },
    /// Delete a widget template.
    Delete {
        theme: String,
        name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// List the field types a widget's settings form can use.
    FieldTypes,
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List { theme, list: args } => {
            let mut extra = Vec::new();
            if let Some(t) = theme {
                extra.push(("themeDeveloperName", t));
            }
            list(client, &["widgettemplates"], &extra, &args)
        }
        Cmd::Get { theme, name, out } => {
            let mut t = get(client, &theme, &name)?;
            if let Some(path) = out {
                let content = str_of(&t, "content").unwrap_or_default().to_string();
                write_file(&path, &content)?;
                if let Some(o) = t.as_object_mut() {
                    o.remove("content");
                    o.insert("contentFile".into(), json!(path.display().to_string()));
                }
            }
            Ok(t)
        }
        Cmd::Create {
            theme,
            name,
            label,
            src,
            fields,
        } => {
            let content = src.read()?.ok_or_else(|| {
                CliError::usage("Template content is required: pass --file <path> or --content.")
            })?;
            let fields = fields.as_deref().map(parse_fields).transpose()?;
            create(client, &theme, &name, label, content, fields)
        }
        Cmd::Edit {
            theme,
            name,
            label,
            src,
            fields,
        } => {
            let remote = get(client, &theme, &name)?;
            let fields = fields.as_deref().map(parse_fields).transpose()?;
            let body = edit_body(&remote, label, src.read()?, fields);
            client.put(
                &["widgettemplates", "theme", &theme, "template", &name],
                Some(&body),
            )
        }
        Cmd::Delete { theme, name, yes } => {
            require_yes(yes, &format!("widget template '{name}' in theme '{theme}'"))?;
            client.delete(
                &["widgettemplates", "theme", &theme, "template", &name],
                &[],
            )
        }
        Cmd::FieldTypes => client.get(&["themes", "widget-field-types"], &[]),
    }
}

pub fn parse_fields(arg: &str) -> Result<Value> {
    let v = json_value(arg)?;
    if v.is_array() {
        Ok(v)
    } else {
        Err(CliError::usage("--fields must be a JSON array of field definitions.")
            .with_hint("Example: [{\"developerName\":\"headline\",\"label\":\"Headline\",\"fieldType\":\"single_line_text\"}]"))
    }
}

pub fn get(client: &Client, theme: &str, name: &str) -> Result<Value> {
    client.get(&["widgettemplates", "theme", theme, "template", name], &[])
}

pub fn create(
    client: &Client,
    theme: &str,
    name: &str,
    label: Option<String>,
    content: String,
    fields: Option<Value>,
) -> Result<Value> {
    let mut b = Map::new();
    b.insert("developerName".into(), json!(name));
    b.insert(
        "label".into(),
        json!(label.unwrap_or_else(|| humanize(name))),
    );
    b.insert("content".into(), json!(content));
    b.insert("fields".into(), fields.unwrap_or_else(|| json!([])));
    client.post(
        &["widgettemplates", "theme", theme],
        Some(&Value::Object(b)),
    )
}

/// Raytha leaves the settings form untouched when `fields` is omitted, so only send it when the
/// caller supplied one.
pub fn edit_body(
    remote: &Value,
    label: Option<String>,
    content: Option<String>,
    fields: Option<Value>,
) -> Value {
    let mut b = Map::new();
    b.insert(
        "label".into(),
        json!(label.unwrap_or_else(|| str_of(remote, "label").unwrap_or_default().to_string())),
    );
    b.insert(
        "content".into(),
        json!(content.unwrap_or_else(|| str_of(remote, "content").unwrap_or_default().to_string())),
    );
    if let Some(f) = fields {
        b.insert("fields".into(), f);
    }
    Value::Object(b)
}
