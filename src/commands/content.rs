//! Content items: the records of a content type (posts, products, team members, ...).
//!
//! `--data` is always the object of field values keyed by field developer name.

use super::content_import::{self, has_file_refs, resolve_files};
use super::{ListArgs, list, require_yes, resolve_template, str_of, web_template_id};

pub const DEFAULT_DETAIL_TEMPLATE: &str = "raytha_html_content_item_detail";
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{BodyArgs, read_text};
use clap::Subcommand;
use serde_json::{Map, Value, json};

const EMPTY_ID: &str = "AAAAAAAAAAAAAAAAAAAAAA";

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List items of a content type.
    List {
        content_type: String,
        /// Restrict to a view (its filter and sort apply).
        #[arg(long)]
        view_id: Option<String>,
        /// Filter expression, for example `title eq 'Hello'`. See `raytha guide content-types`.
        #[arg(long)]
        filter: Option<String>,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Get one item by id.
    Get { content_type: String, id: String },
    /// Get an item by its URL path.
    GetByPath {
        content_type: String,
        /// Route path, for example `blog/hello-world`.
        path: String,
    },
    /// Create an item. `--data` holds the field values.
    Create {
        content_type: String,
        #[command(flatten)]
        body: BodyArgs,
        /// Web template developer name (ACTIVE theme) used to render the item.
        #[arg(long)]
        template: Option<String>,
        /// Web template id, instead of --template.
        #[arg(long)]
        template_id: Option<String>,
        /// Save as an unpublished draft.
        #[arg(long)]
        draft: bool,
        /// URL path for the item (set after creation).
        #[arg(long)]
        route_path: Option<String>,
    },
    /// Edit an item's field values (published unless --draft).
    Edit {
        content_type: String,
        id: String,
        #[command(flatten)]
        body: BodyArgs,
        /// Overlay --data onto the current values instead of replacing them.
        #[arg(long)]
        merge: bool,
        /// Save as draft instead of publishing the change.
        #[arg(long)]
        draft: bool,
    },
    /// Change the item's URL path and/or template.
    Settings {
        content_type: String,
        id: String,
        #[arg(long)]
        route_path: Option<String>,
        #[arg(long)]
        template: Option<String>,
        #[arg(long)]
        template_id: Option<String>,
    },
    /// Publish the item's current draft (or re-publish its content).
    Publish { content_type: String, id: String },
    /// Take the item offline.
    Unpublish { content_type: String, id: String },
    /// Throw away unpublished draft changes.
    DiscardDraft { content_type: String, id: String },
    /// Make this item the site home page.
    SetHome { content_type: String, id: String },
    /// Move an item to the trash.
    Delete {
        content_type: String,
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Create many items at once from a JSON array or JSON Lines file.
    ///
    /// Each row is an object of field values, like `content create --data`. A relationship field
    /// may hold an item id, a route path, or the primary field value of the related item (a row
    /// may reference another row of the same file; that row is created first). A string value
    /// `@file:./img.jpg` uploads the file and stores its object key. Rows are created
    /// independently: failures are listed with their row `index` and the rest still import.
    Import {
        content_type: String,
        /// Items file: JSON array or JSON Lines (`-` for stdin).
        #[arg(long, value_name = "PATH|-")]
        file: String,
        /// Web template developer name (ACTIVE theme) for every item.
        #[arg(long)]
        template: Option<String>,
        /// Web template id, instead of --template.
        #[arg(long)]
        template_id: Option<String>,
        /// Save every item as an unpublished draft.
        #[arg(long)]
        draft: bool,
        /// Give up waiting after this many seconds (the import keeps running).
        #[arg(long, default_value_t = 300, value_name = "SECONDS")]
        wait_timeout: u64,
    },
    /// Move several items of one content type to the trash in a single call.
    ///
    /// Give the ids with `--ids a,b,c` and/or `--ids-file` (a JSON array, or one id per line;
    /// `-` reads stdin). Every id must belong to the content type or nothing is deleted.
    DeleteMany {
        content_type: String,
        /// Comma-separated item ids.
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
        /// File with ids: a JSON array, or ids separated by whitespace. `-` for stdin.
        #[arg(long, value_name = "PATH|-")]
        ids_file: Option<String>,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Point items at one detail template of the active theme (after a theme switch, say).
    ///
    /// Give `--ids`/`--ids-file` for specific items, or `--all` for every item of the type.
    AssignTemplate {
        content_type: String,
        /// Web template developer name in the ACTIVE theme.
        #[arg(long, conflicts_with = "template_id")]
        template: Option<String>,
        /// Web template id, instead of --template.
        #[arg(long)]
        template_id: Option<String>,
        /// Comma-separated item ids.
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
        /// File with ids: a JSON array, or ids separated by whitespace. `-` for stdin.
        #[arg(long, value_name = "PATH|-")]
        ids_file: Option<String>,
        /// Apply to every item of the content type.
        #[arg(long, conflicts_with_all = ["ids", "ids_file"])]
        all: bool,
    },
    /// List trashed items.
    Trash { content_type: String },
    /// Restore a trashed item.
    Restore { content_type: String, id: String },
    /// Permanently delete a trashed item.
    Purge {
        content_type: String,
        id: String,
        /// Confirm permanent deletion.
        #[arg(long)]
        yes: bool,
    },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List {
            content_type,
            view_id,
            filter,
            list: args,
        } => {
            let mut extra = Vec::new();
            if let Some(v) = view_id {
                extra.push(("viewId", v));
            }
            if let Some(f) = filter {
                extra.push(("filter", f));
            }
            list(client, &["contentitems", &content_type], &extra, &args)
        }
        Cmd::Get { content_type, id } => client.get(&["contentitems", &content_type, &id], &[]),
        Cmd::GetByPath { content_type, path } => {
            let path = path.trim_matches('/').to_string();
            let route = client.get(&["contentitems", &content_type, "route", &path], &[])?;
            match str_of(&route, "contentItemId") {
                Some(id) if id != EMPTY_ID && !id.is_empty() => {
                    client.get(&["contentitems", &content_type, id], &[])
                }
                _ => Ok(route),
            }
        }
        Cmd::Create {
            content_type,
            body,
            template,
            template_id,
            draft,
            route_path,
        } => {
            let content = body.require_object("Item content (field values)")?;
            let content = with_files(client, content)?;
            let mut b = Map::new();
            b.insert("saveAsDraft".into(), json!(draft));
            b.insert("content".into(), Value::Object(content));
            // The API insists on a template. Default to the built-in detail view, which every
            // theme has and which new content types may use.
            let template_id =
                match resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                {
                    Some(t) => t,
                    None => web_template_id(client, None, DEFAULT_DETAIL_TEMPLATE).map_err(|e| {
                        e.with_hint(format!(
                            "Content needs a template: pass --template <developer_name> (a detail \
                             template in the active theme; `raytha web-template list --theme <theme>`). \
                             The default '{DEFAULT_DETAIL_TEMPLATE}' could not be found."
                        ))
                    })?,
                };
            b.insert("templateId".into(), json!(template_id));
            let created = client.post(&["contentitems", &content_type], Some(&Value::Object(b)))?;
            if let Some(path) = route_path {
                let id = str_of(&created, "id").unwrap_or_default().to_string();
                let current = client.get(&["contentitems", &content_type, &id], &[])?;
                let mut s = Map::new();
                s.insert("routePath".into(), json!(path));
                if let Some(t) = str_of(&current, "webTemplateId") {
                    s.insert("templateId".into(), json!(t));
                }
                client.put(
                    &["contentitems", &content_type, &id, "settings"],
                    Some(&Value::Object(s)),
                )?;
                return client.get(&["contentitems", &content_type, &id], &[]);
            }
            Ok(created)
        }
        Cmd::Edit {
            content_type,
            id,
            body,
            merge,
            draft,
        } => {
            let new_fields = body.require_object("Item content (field values)")?;
            let new_fields = with_files(client, new_fields)?;
            let content = if merge {
                let item = client.get(&["contentitems", &content_type, &id], &[])?;
                let mut base = current_content(&item);
                base.extend(new_fields);
                base
            } else {
                new_fields
            };
            let b = json!({ "saveAsDraft": draft, "content": Value::Object(content) });
            client.put(&["contentitems", &content_type, &id], Some(&b))
        }
        Cmd::Settings {
            content_type,
            id,
            route_path,
            template,
            template_id,
        } => {
            let item = client.get(&["contentitems", &content_type, &id], &[])?;
            let template_id =
                resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                    .or_else(|| str_of(&item, "webTemplateId").map(str::to_string));
            let mut b = Map::new();
            b.insert(
                "routePath".into(),
                json!(
                    route_path.unwrap_or_else(|| str_of(&item, "routePath")
                        .unwrap_or_default()
                        .to_string())
                ),
            );
            if let Some(t) = template_id {
                b.insert("templateId".into(), json!(t));
            }
            client.put(
                &["contentitems", &content_type, &id, "settings"],
                Some(&Value::Object(b)),
            )
        }
        Cmd::Publish { content_type, id } => {
            // The API publishes by saving the content as non-draft.
            let item = client.get(&["contentitems", &content_type, &id], &[])?;
            let content = current_content(&item);
            if content.is_empty() {
                return Err(CliError::validation("The item has no content to publish."));
            }
            let b = json!({ "saveAsDraft": false, "content": Value::Object(content) });
            client.put(&["contentitems", &content_type, &id], Some(&b))
        }
        Cmd::Unpublish { content_type, id } => {
            client.put(&["contentitems", &content_type, &id, "unpublish"], None)
        }
        Cmd::DiscardDraft { content_type, id } => {
            client.put(&["contentitems", &content_type, &id, "discard-draft"], None)
        }
        Cmd::SetHome { content_type, id } => client.put(
            &["contentitems", &content_type, &id, "set-as-home-page"],
            None,
        ),
        Cmd::Delete {
            content_type,
            id,
            yes,
        } => {
            require_yes(yes, &format!("content item '{id}'"))?;
            client.delete(&["contentitems", &content_type, &id], &[])
        }
        Cmd::Import {
            content_type,
            file,
            template,
            template_id,
            draft,
            wait_timeout,
        } => content_import::run(
            client,
            content_import::ImportArgs {
                content_type,
                file,
                template,
                template_id,
                draft,
                wait_timeout,
            },
        ),
        Cmd::DeleteMany {
            content_type,
            mut ids,
            ids_file,
            yes,
        } => {
            if let Some(src) = ids_file {
                ids.extend(parse_ids(&read_text(&src)?)?);
            }
            ids.retain(|id| !id.is_empty());
            ids.sort();
            ids.dedup();
            if ids.is_empty() {
                return Err(CliError::usage("No item ids given.")
                    .with_hint("Pass --ids a,b,c or --ids-file ids.json."));
            }
            require_yes(yes, &format!("{} content items", ids.len()))?;
            client.delete_json(
                &["contentitems", &content_type, "items"],
                &json!({ "ids": ids }),
            )
        }
        Cmd::AssignTemplate {
            content_type,
            template,
            template_id,
            mut ids,
            ids_file,
            all,
        } => {
            let template_id =
                resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                    .ok_or_else(|| {
                        CliError::usage("A template is required: pass --template or --template-id.")
                    })?;
            if let Some(src) = ids_file {
                ids.extend(parse_ids(&read_text(&src)?)?);
            }
            ids.retain(|id| !id.is_empty());
            ids.sort();
            ids.dedup();
            let mut body = json!({ "templateId": template_id });
            if all {
                // omitting the ids moves every item of the content type
            } else if ids.is_empty() {
                return Err(CliError::usage("No item ids given.").with_hint(
                    "Pass --ids a,b,c, --ids-file ids.json, or --all for every item of the type.",
                ));
            } else {
                body["contentItemIds"] = json!(ids);
            }
            client.post(&["contentitems", &content_type, "template"], Some(&body))
        }
        Cmd::Trash { content_type } => client.get(&["contentitems", &content_type, "trash"], &[]),
        Cmd::Restore { content_type, id } => {
            client.put(&["contentitems", &content_type, &id, "restore"], None)
        }
        Cmd::Purge {
            content_type,
            id,
            yes,
        } => {
            require_yes(yes, &format!("trashed content item '{id}' permanently"))?;
            client.delete(&["contentitems", &content_type, "trash", &id], &[])
        }
    }
}

/// Uploads any `@file:` references in the values (relative to the working directory).
fn with_files(client: &Client, content: Map<String, Value>) -> Result<Map<String, Value>> {
    let mut v = Value::Object(content);
    if has_file_refs(&v) {
        resolve_files(client, &mut v, std::path::Path::new("."))?;
    }
    Ok(match v {
        Value::Object(m) => m,
        _ => Map::new(),
    })
}

/// Ids from a JSON array of strings, or any whitespace/comma separated list.
fn parse_ids(text: &str) -> Result<Vec<String>> {
    let trimmed = text.trim();
    if trimmed.starts_with('[') {
        let v: Vec<String> = serde_json::from_str(trimmed).map_err(|e| {
            CliError::usage(format!("ids file is not a JSON array of strings: {e}"))
        })?;
        return Ok(v);
    }
    Ok(trimmed
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// The content an edit should start from: the draft when there is one, else what is published.
pub fn current_content(item: &Value) -> Map<String, Value> {
    let is_draft = item
        .get("isDraft")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let pick = |key: &str| {
        item.get(key).and_then(Value::as_object).map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), writable_value(v)))
                .collect::<Map<String, Value>>()
        })
    };
    if is_draft && let Some(d) = pick("draftContent") {
        return d;
    }
    pick("publishedContent")
        .or_else(|| pick("draftContent"))
        .unwrap_or_default()
}

/// Turn a field as the API *returns* it into the shape the API *accepts*.
///
/// GET wraps every value as `{"value": ..., "text": ..., "hasValue": ...}` and expands a
/// relationship into the related item (`{"id": ..., "primaryField": ...}`). PUT wants the bare
/// value, and a relationship as the related item's id. Sending the wrapper back stores it as a
/// JSON string (the title becomes `{"text": ...}`) and wipes the relationship.
pub fn writable_value(v: &Value) -> Value {
    let Some(o) = v.as_object() else {
        return v.clone();
    };
    if o.contains_key("hasValue") && (o.contains_key("value") || o.contains_key("text")) {
        return match o.get("value") {
            Some(Value::Object(inner))
                if inner.contains_key("id") && inner.contains_key("primaryField") =>
            {
                inner["id"].clone()
            }
            Some(inner) => inner.clone(),
            None => Value::Null,
        };
    }
    if o.contains_key("id") && o.contains_key("primaryField") {
        return o["id"].clone();
    }
    v.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_from_json_or_plain_lists() {
        assert_eq!(parse_ids(r#"["a","-b"]"#).unwrap(), vec!["a", "-b"]);
        assert_eq!(parse_ids("a\n-b, c\n").unwrap(), vec!["a", "-b", "c"]);
        assert!(parse_ids("[1,2]").is_err());
    }

    #[test]
    fn prefers_draft_content() {
        let item = json!({"isDraft": true, "draftContent": {"a": 2}, "publishedContent": {"a": 1}});
        assert_eq!(current_content(&item)["a"], 2);
        let item = json!({"isDraft": false, "draftContent": null, "publishedContent": {"a": 1}});
        assert_eq!(current_content(&item)["a"], 1);
    }

    #[test]
    fn merge_base_unwraps_api_values() {
        // What GET /contentitems/{type}/{id} returns for a few field types.
        let item = json!({"isDraft": false, "publishedContent": {
            "title": {"value": "Hello", "text": "Hello", "hasValue": true},
            "hide_author_bio": {"value": true, "text": "True", "hasValue": true},
            "featured_image": {"value": null, "text": "", "hasValue": false},
            "tags": {"value": ["a", "b"], "text": "a, b", "hasValue": true},
            "author_1": {"id": "KIb9", "primaryField": "Zack", "routePath": "authors/zack"},
            "author_2": {"value": {"id": "AbCd", "primaryField": "Ann"}, "text": "Ann", "hasValue": true},
            "plain": "already bare"
        }});
        let c = current_content(&item);
        assert_eq!(c["title"], "Hello");
        assert_eq!(c["hide_author_bio"], true);
        assert_eq!(c["featured_image"], Value::Null);
        assert_eq!(c["tags"], json!(["a", "b"]));
        assert_eq!(c["author_1"], "KIb9");
        assert_eq!(c["author_2"], "AbCd");
        assert_eq!(c["plain"], "already bare");
    }
}
