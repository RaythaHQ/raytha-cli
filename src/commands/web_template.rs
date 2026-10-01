//! Web templates: liquid layouts and page/list/detail templates that belong to a theme.

use super::{ListArgs, content_type_names, list, require_yes, str_of, write_file};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{ContentSrc, humanize};
use clap::{Args, Subcommand};
use serde_json::{Map, Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List web templates, optionally for one theme.
    List {
        /// Theme developer name.
        #[arg(long)]
        theme: Option<String>,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Get one template, including its liquid content.
    Get {
        theme: String,
        name: String,
        /// Write the liquid content to this file instead of returning it inline.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Create a template from a liquid file.
    Create {
        theme: String,
        /// Developer name (lowercase letters, digits, underscores).
        name: String,
        /// Display label (defaults to a humanized developer name).
        #[arg(long)]
        label: Option<String>,
        #[command(flatten)]
        src: ContentSrc,
        #[command(flatten)]
        opts: Opts,
    },
    /// Edit a template. Omitted flags keep their current value.
    Edit {
        theme: String,
        name: String,
        #[arg(long)]
        label: Option<String>,
        #[command(flatten)]
        src: ContentSrc,
        #[command(flatten)]
        opts: Opts,
    },
    /// Check liquid syntax on the server without saving anything.
    ///
    /// Exits 0 with `{"valid":true}`, or fails with `validation_failed` and the parser's message
    /// (including line and column). Create and edit run the same check.
    Validate {
        #[command(flatten)]
        src: ContentSrc,
    },
    /// Render a template on the server and return the HTML, without publishing anything.
    ///
    /// With no flags the template (and its parent layout) renders against an empty target. Pass
    /// `--content-item` (drafts included) or `--view` to render with real data. A broken template
    /// fails with `validation_failed`; `error.message` names the template, and `error.line` and
    /// `error.column` point into it when Liquid reports a position.
    Preview {
        theme: String,
        name: String,
        /// Render this content item (id) with the template.
        #[arg(long, value_name = "ID", conflicts_with = "view")]
        content_item: Option<String>,
        /// Render this view (id) with the template.
        #[arg(long, value_name = "ID")]
        view: Option<String>,
        /// Write the HTML to this file and return only its size and `<title>`.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Delete a template.
    Delete {
        theme: String,
        name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Args, Debug, Default, Clone)]
pub struct Opts {
    /// Mark as a base layout (must contain `{% renderbody %}`). Inferred on create.
    #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
    pub base_layout: Option<bool>,
    /// Developer name of the parent (base layout) template. Use "" to clear.
    #[arg(long)]
    pub parent: Option<String>,
    /// Give content types created later access to this template.
    #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
    pub allow_new_content_types: Option<bool>,
    /// Content type developer names that may use this template (comma separated; "" clears).
    #[arg(long, value_delimiter = ',', value_name = "NAMES")]
    pub content_types: Option<Vec<String>>,
}

/// What the caller wants to change. `None` means "keep the current value".
#[derive(Debug, Default, Clone)]
pub struct Spec {
    pub label: Option<String>,
    pub content: Option<String>,
    pub base_layout: Option<bool>,
    /// `Some("")` clears the parent.
    pub parent: Option<String>,
    pub allow_new: Option<bool>,
    pub content_types: Option<Vec<String>>,
}

impl Spec {
    pub fn from_args(label: Option<String>, content: Option<String>, o: &Opts) -> Spec {
        Spec {
            label,
            content,
            base_layout: o.base_layout,
            parent: o.parent.clone(),
            allow_new: o.allow_new_content_types,
            content_types: o
                .content_types
                .as_ref()
                .map(|v| v.iter().filter(|s| !s.trim().is_empty()).cloned().collect()),
        }
    }
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List { theme, list: args } => {
            let mut extra = Vec::new();
            if let Some(t) = theme {
                extra.push(("themeDeveloperName", t));
            }
            list(client, &["webtemplates"], &extra, &args)
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
            opts,
        } => {
            let content = src.read()?.ok_or_else(|| {
                CliError::usage("Template content is required: pass --file <path> or --content.")
            })?;
            let spec = Spec::from_args(label, Some(content), &opts);
            create(client, &theme, &name, &spec)
        }
        Cmd::Edit {
            theme,
            name,
            label,
            src,
            opts,
        } => {
            let spec = Spec::from_args(label, src.read()?, &opts);
            let remote = get(client, &theme, &name)?;
            let body = edit_body(client, &remote, &spec)?;
            client.put(
                &["webtemplates", "theme", &theme, "template", &name],
                Some(&body),
            )
        }
        Cmd::Validate { src } => {
            let content = src.read()?.ok_or_else(|| {
                CliError::usage("Nothing to validate: pass --file <path> (or `-`) or --content.")
            })?;
            validate(client, &content)?;
            Ok(json!({ "valid": true }))
        }
        Cmd::Preview {
            theme,
            name,
            content_item,
            view,
            out,
        } => {
            let id = str_of(&get(client, &theme, &name)?, "id")
                .map(str::to_string)
                .ok_or_else(|| CliError::server("bad_response", "Template has no id."))?;
            let mut query = Vec::new();
            if let Some(c) = content_item {
                query.push(("contentItemId", c));
            }
            if let Some(v) = view {
                query.push(("viewId", v));
            }
            let html = client.get_text(&["webtemplates", &id, "render-preview"], &query)?;
            let title = html_title(&html);
            match out {
                Some(path) => {
                    write_file(&path, &html)?;
                    Ok(json!({
                        "file": path.display().to_string(),
                        "bytes": html.len(),
                        "title": title,
                    }))
                }
                None => Ok(json!({ "html": html, "bytes": html.len(), "title": title })),
            }
        }
        Cmd::Delete { theme, name, yes } => {
            require_yes(yes, &format!("web template '{name}' in theme '{theme}'"))?;
            client.delete(&["webtemplates", "theme", &theme, "template", &name], &[])
        }
    }
}

/// Asks Raytha to parse Liquid without saving it. Fails with the parser's message, line and column.
pub fn validate(client: &Client, content: &str) -> Result<()> {
    client.post(
        &["webtemplates", "validate"],
        Some(&json!({ "content": content })),
    )?;
    Ok(())
}

fn html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open_end = start + lower[start..].find('>')? + 1;
    let end = open_end + lower[open_end..].find("</title>")?;
    let title = html[open_end..end].trim();
    (!title.is_empty()).then(|| title.to_string())
}

pub fn get(client: &Client, theme: &str, name: &str) -> Result<Value> {
    client.get(&["webtemplates", "theme", theme, "template", name], &[])
}

pub fn create(client: &Client, theme: &str, name: &str, spec: &Spec) -> Result<Value> {
    let body = create_body(name, spec)?;
    client.post(&["webtemplates", "theme", theme], Some(&body))
}

pub fn create_body(name: &str, spec: &Spec) -> Result<Value> {
    let content = spec
        .content
        .clone()
        .ok_or_else(|| CliError::usage("Template content is required."))?;
    let is_base = spec
        .base_layout
        .unwrap_or_else(|| content.contains("renderbody"));
    let mut b = Map::new();
    b.insert("developerName".into(), json!(name));
    b.insert(
        "label".into(),
        json!(spec.label.clone().unwrap_or_else(|| humanize(name))),
    );
    b.insert("content".into(), json!(content));
    b.insert("isBaseLayout".into(), json!(is_base));
    if let Some(p) = spec.parent.as_ref().filter(|p| !p.is_empty()) {
        b.insert("parentTemplateDeveloperName".into(), json!(p));
    }
    b.insert(
        "allowAccessForNewContentTypes".into(),
        json!(spec.allow_new.unwrap_or(false)),
    );
    b.insert(
        "templateAccessToModelDefinitions".into(),
        json!(spec.content_types.clone().unwrap_or_default()),
    );
    Ok(Value::Object(b))
}

/// Builds a full PUT body: the remote's current values overlaid with `spec`. Editing with a
/// partial body would otherwise reset fields (Raytha's edit replaces the content type access
/// list), so unspecified values are carried over.
pub fn edit_body(client: &Client, remote: &Value, spec: &Spec) -> Result<Value> {
    let label = spec
        .label
        .clone()
        .or_else(|| str_of(remote, "label").map(str::to_string))
        .unwrap_or_default();
    let content = spec
        .content
        .clone()
        .or_else(|| str_of(remote, "content").map(str::to_string))
        .unwrap_or_default();
    let is_base = spec
        .base_layout
        .or_else(|| remote.get("isBaseLayout").and_then(Value::as_bool))
        .unwrap_or(false);
    let parent = match &spec.parent {
        Some(p) if p.is_empty() => None,
        Some(p) => Some(p.clone()),
        None => remote
            .get("parentTemplate")
            .and_then(|p| str_of(p, "developerName"))
            .map(str::to_string),
    };
    let allow = spec
        .allow_new
        .or_else(|| {
            remote
                .get("allowAccessForNewContentTypes")
                .and_then(Value::as_bool)
        })
        .unwrap_or(false);
    let types = match &spec.content_types {
        Some(t) => t.clone(),
        None => {
            let ids: Vec<String> = remote
                .get("templateAccessToModelDefinitions")
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            content_type_names(client, &ids)?
        }
    };

    let mut b = Map::new();
    b.insert("label".into(), json!(label));
    b.insert("content".into(), json!(content));
    b.insert("isBaseLayout".into(), json!(is_base));
    if let Some(p) = parent {
        b.insert("parentTemplateDeveloperName".into(), json!(p));
    }
    b.insert("allowAccessForNewContentTypes".into(), json!(allow));
    b.insert("templateAccessToModelDefinitions".into(), json!(types));
    Ok(Value::Object(b))
}

#[cfg(test)]
mod preview_tests {
    use super::html_title;

    #[test]
    fn finds_the_title() {
        assert_eq!(
            html_title("<html><HEAD><Title> Hi there </TITLE></head>").as_deref(),
            Some("Hi there")
        );
        assert_eq!(html_title("<p>no title</p>"), None);
    }
}
