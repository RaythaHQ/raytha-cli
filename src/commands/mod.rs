//! Command implementations. Each module exposes a clap `Cmd` enum and a `run` function that
//! returns the JSON `data` payload.

pub mod content;
pub mod content_type;
pub mod doctor;
pub mod media;
pub mod menu;
pub mod site_page;
pub mod spec;
pub mod theme;
pub mod user;
pub mod web_template;
pub mod widget_template;

use crate::client::Client;
use crate::error::{CliError, Result};
use clap::Args;
use serde_json::{Map, Value, json};

/// Shared list flags: search, ordering, paging, and `--all` to fetch every page.
#[derive(Args, Clone, Debug)]
pub struct ListArgs {
    /// Free-text search.
    #[arg(long)]
    pub search: Option<String>,
    /// Sort expression, for example `CreationTime desc` or `Title asc`.
    #[arg(long)]
    pub order_by: Option<String>,
    /// 1-based page number.
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    /// Items per page.
    #[arg(long, default_value_t = 50)]
    pub page_size: u32,
    /// Fetch every page and return all items.
    #[arg(long)]
    pub all: bool,
}

impl Default for ListArgs {
    fn default() -> Self {
        ListArgs {
            search: None,
            order_by: None,
            page: 1,
            page_size: 50,
            all: false,
        }
    }
}

/// Lists a collection, adding `page`, `pageSize`, and `hasMore` so agents know when to continue.
pub fn list(
    client: &Client,
    segs: &[&str],
    extra: &[(&str, String)],
    args: &ListArgs,
) -> Result<Value> {
    let page_size = args.page_size.clamp(1, 500);
    let mut page = if args.all { 1 } else { args.page.max(1) };
    let mut all_items: Vec<Value> = Vec::new();
    loop {
        let mut q: Vec<(&str, String)> = extra.to_vec();
        if let Some(s) = &args.search {
            q.push(("search", s.clone()));
        }
        if let Some(o) = &args.order_by {
            q.push(("orderBy", o.clone()));
        }
        q.push(("pageNumber", page.to_string()));
        q.push(("pageSize", page_size.to_string()));
        let res = client.get(segs, &q)?;

        let items = res
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let total = res
            .get("totalCount")
            .and_then(Value::as_u64)
            .unwrap_or(items.len() as u64);

        if !args.all {
            let has_more = (page as u64) * (page_size as u64) < total;
            let mut out = Map::new();
            out.insert("items".into(), Value::Array(items));
            out.insert("totalCount".into(), json!(total));
            out.insert("page".into(), json!(page));
            out.insert("pageSize".into(), json!(page_size));
            out.insert("hasMore".into(), json!(has_more));
            return Ok(Value::Object(out));
        }

        let got = items.len();
        all_items.extend(items);
        if got == 0 || (all_items.len() as u64) >= total || all_items.len() >= 10_000 {
            return Ok(json!({ "items": all_items, "totalCount": total, "hasMore": false }));
        }
        page += 1;
    }
}

/// Destructive commands require an explicit `--yes`.
pub fn require_yes(yes: bool, what: &str) -> Result<()> {
    if yes {
        Ok(())
    } else {
        Err(
            CliError::usage(format!("Refusing to delete {what} without confirmation."))
                .with_hint("Re-run with --yes to confirm."),
        )
    }
}

pub fn str_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// The developer name of the currently active theme.
pub fn active_theme(client: &Client) -> Result<String> {
    let res = client.get(
        &["themes"],
        &[("pageSize", "500".into()), ("pageNumber", "1".into())],
    )?;
    let items = res.get("items").and_then(Value::as_array);
    items
        .and_then(|a| {
            a.iter()
                .find(|t| t.get("isActive").and_then(Value::as_bool) == Some(true))
        })
        .and_then(|t| str_of(t, "developerName"))
        .map(str::to_string)
        .ok_or_else(|| {
            CliError::not_found("No active theme found.")
                .with_hint("Create and activate one: `raytha theme activate <developer_name>`.")
        })
}

/// Resolves a web template developer name to its id. Site pages and content items reference
/// templates by id, and the template must belong to the active theme.
pub fn web_template_id(client: &Client, theme: Option<&str>, name: &str) -> Result<String> {
    let theme = match theme {
        Some(t) => t.to_string(),
        None => active_theme(client)?,
    };
    let t = client.get(&["webtemplates", "theme", &theme, "template", name], &[])?;
    str_of(&t, "id").map(str::to_string).ok_or_else(|| {
        CliError::not_found(format!(
            "Web template '{name}' in theme '{theme}' has no id."
        ))
    })
}

/// Picks `--template-id` or resolves `--template <developer_name>` in the active theme.
pub fn resolve_template(
    client: &Client,
    template: Option<&str>,
    template_id: Option<&str>,
    theme: Option<&str>,
) -> Result<Option<String>> {
    match (template, template_id) {
        (Some(_), Some(_)) => Err(CliError::usage(
            "Use either --template (developer name) or --template-id, not both.",
        )),
        (_, Some(id)) => Ok(Some(id.to_string())),
        (Some(name), None) => web_template_id(client, theme, name).map(Some),
        (None, None) => Ok(None),
    }
}

/// Maps content type id -> developer name. Needs the ManageContentTypes permission.
pub fn content_type_index(client: &Client) -> Result<std::collections::HashMap<String, String>> {
    let res = client
        .get(
            &["contenttypes"],
            &[("pageSize", "500".into()), ("pageNumber", "1".into())],
        )
        .map_err(|e| {
            if e.exit == crate::error::EXIT_AUTH {
                CliError::new(
                    &e.code,
                    "This key cannot read content types, which is needed to preserve the \
                     template's content type access list.",
                    e.exit,
                )
                .with_hint(
                    "Pass --content-types a,b explicitly (or --content-types \"\" to clear), \
                     or use a key with ManageContentTypes.",
                )
            } else {
                e
            }
        })?;
    Ok(res
        .get("items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|c| {
                    Some((
                        str_of(c, "id")?.to_string(),
                        str_of(c, "developerName")?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default())
}

/// Resolves content type ids to developer names (unknown ids are dropped).
pub fn content_type_names(client: &Client, ids: &[String]) -> Result<Vec<String>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let index = content_type_index(client)?;
    Ok(ids.iter().filter_map(|id| index.get(id).cloned()).collect())
}

/// Writes `content` to `path` (creating parent dirs), for `--out`.
pub fn write_file(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}
