//! Themes: the container for web templates, widget templates, and theme media.

use super::task::{self, WaitArgs};
use super::{ListArgs, list, require_yes, str_of};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{humanize, to_developer_name};
use crate::sync;
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List themes; `isActive` marks the one the public site uses.
    List(ListArgs),
    /// Get one theme by developer name.
    Get { developer_name: String },
    /// Create an empty theme.
    Create {
        /// Developer name (normalized to lowercase_with_underscores).
        developer_name: String,
        /// Display title (defaults to a humanized developer name).
        #[arg(long)]
        title: Option<String>,
        /// Description (defaults to the title).
        #[arg(long)]
        description: Option<String>,
        /// Also insert Raytha's default theme media (images).
        #[arg(long)]
        with_default_media: bool,
    },
    /// Edit title or description.
    Edit {
        developer_name: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a theme (the active theme cannot be deleted).
    Delete {
        developer_name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Tell Raytha which template of THIS theme replaces each template of the active theme.
    ///
    /// Views and content items stay bound to the active theme's templates until they are matched.
    /// `--map old=new` pairs an active-theme template developer name with one from this theme.
    /// Runs as a background job (returns its id) and then makes this theme the active one.
    /// Only templates that still have no binding in this theme are accepted.
    MatchTemplates {
        developer_name: String,
        /// `active_template=this_theme_template`, repeatable or comma separated.
        #[arg(long = "map", value_delimiter = ',', value_name = "OLD=NEW")]
        map: Vec<String>,
        /// JSON object `{"old":"new"}`: inline, `@path` or `-`.
        #[arg(long, value_name = "JSON|@PATH|-")]
        map_json: Option<String>,
        #[command(flatten)]
        wait: WaitArgs,
    },
    /// Copy a theme (templates, widget templates, media and view bindings) under a new name.
    ///
    /// Runs as a background job; the new theme appears when it finishes (use `--wait`).
    Duplicate {
        /// Theme to copy.
        developer_name: String,
        /// Developer name of the copy (normalized to lowercase_with_underscores).
        new_developer_name: String,
        /// Display title (defaults to a humanized developer name).
        #[arg(long)]
        title: Option<String>,
        /// Description (defaults to the title).
        #[arg(long)]
        description: Option<String>,
        #[command(flatten)]
        wait: WaitArgs,
    },
    /// Show which views, items, site pages and child templates use each web template.
    ///
    /// Use it before deleting or renaming a template, or `--unused` to list templates nothing uses.
    Usage {
        developer_name: String,
        /// Only this template (developer name).
        #[arg(long, value_name = "NAME")]
        template: Option<String>,
        /// Only custom templates that nothing uses (built-in templates are left out).
        #[arg(long, conflicts_with = "template")]
        unused: bool,
    },
    /// Make this theme the one the public site renders with.
    Activate { developer_name: String },
    /// Download a theme into a directory of plain files you can edit.
    Pull(PullArgs),
    /// Sync a directory of files up to Raytha (creates or updates; `--dry-run` to preview).
    Push(PushArgs),
    /// Theme media (images, css, js referenced by templates).
    #[command(subcommand)]
    Media(MediaCmd),
}

#[derive(Args, Debug)]
pub struct PullArgs {
    /// Theme developer name.
    pub developer_name: String,
    /// Target directory (created if missing; existing files are overwritten).
    pub dir: PathBuf,
    /// Skip downloading theme media.
    #[arg(long)]
    pub no_media: bool,
}

#[derive(Args, Debug)]
pub struct PushArgs {
    /// Theme directory containing theme.json (see `raytha guide themes`).
    pub dir: PathBuf,
    /// Show what would change without changing anything.
    #[arg(long)]
    pub dry_run: bool,
    /// Delete remote templates and media that are not present locally (built-ins are kept).
    #[arg(long)]
    pub prune: bool,
    /// Re-upload media whose size differs from the local file (delete then upload).
    #[arg(long)]
    pub replace_media: bool,
    /// Skip media entirely.
    #[arg(long)]
    pub no_media: bool,
    /// Activate the theme after a successful push.
    #[arg(long)]
    pub activate: bool,
    /// Push to this developer name instead of the one in theme.json.
    #[arg(long, value_name = "NAME")]
    pub theme: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum MediaCmd {
    /// List media belonging to a theme.
    List { theme: String },
    /// Upload a file to a theme.
    Upload {
        theme: String,
        /// Local file path.
        path: PathBuf,
    },
    /// Delete a theme media item by id.
    Delete {
        theme: String,
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => list(client, &["themes"], &[], &args),
        Cmd::Get { developer_name } => client.get(&["themes", &developer_name], &[]),
        Cmd::Create {
            developer_name,
            title,
            description,
            with_default_media,
        } => {
            let dev = to_developer_name(&developer_name);
            if dev.is_empty() {
                return Err(CliError::usage(
                    "Developer name must contain letters or digits.",
                ));
            }
            let title = title.unwrap_or_else(|| humanize(&dev));
            let description = description.unwrap_or_else(|| title.clone());
            let body = json!({
                "title": title,
                "developerName": dev,
                "description": description,
                "insertDefaultThemeMediaItems": with_default_media,
            });
            let mut res = client.post(&["themes"], Some(&body))?;
            if let Some(o) = res.as_object_mut() {
                o.insert("developerName".into(), json!(dev));
            }
            Ok(res)
        }
        Cmd::Edit {
            developer_name,
            title,
            description,
        } => {
            let remote = client.get(&["themes", &developer_name], &[])?;
            let body = json!({
                "title": title.unwrap_or_else(|| str_of(&remote, "title").unwrap_or_default().to_string()),
                "description": description.unwrap_or_else(|| str_of(&remote, "description").unwrap_or_default().to_string()),
            });
            client.put(&["themes", &developer_name], Some(&body))
        }
        Cmd::Delete {
            developer_name,
            yes,
        } => {
            require_yes(
                yes,
                &format!("theme '{developer_name}' and all its templates"),
            )?;
            client.delete(&["themes", &developer_name], &[])
        }
        Cmd::MatchTemplates {
            developer_name,
            map,
            map_json,
            wait,
        } => {
            let mut pairs = serde_json::Map::new();
            if let Some(raw) = map_json {
                match crate::input::json_value(&raw)? {
                    Value::Object(m) => pairs.extend(m),
                    _ => return Err(CliError::usage("--map-json must be a JSON object.")),
                }
            }
            for entry in map {
                let (old, new) = entry.split_once('=').ok_or_else(|| {
                    CliError::usage(format!("'{entry}' is not OLD=NEW."))
                        .with_hint("Example: --map raytha_html_content_item_list=aurora_list_posts")
                })?;
                pairs.insert(old.trim().to_string(), json!(new.trim()));
            }
            let started = client.post(
                &["themes", &developer_name, "match-web-templates"],
                Some(&json!({ "matchedWebTemplateDeveloperNames": pairs })),
            )?;
            task::finish(client, started, &wait)
        }
        Cmd::Duplicate {
            developer_name,
            new_developer_name,
            title,
            description,
            wait,
        } => {
            let new_name = to_developer_name(&new_developer_name);
            let title = title.unwrap_or_else(|| humanize(&new_name));
            let description = description.unwrap_or_else(|| title.clone());
            let started = client.post(
                &["themes", &developer_name, "duplicate"],
                Some(&json!({
                    "title": title,
                    "developerName": new_name,
                    "description": description,
                })),
            )?;
            let mut out = task::finish(client, started, &wait)?;
            out["theme"] = json!(new_name);
            Ok(out)
        }
        Cmd::Usage {
            developer_name,
            template,
            unused,
        } => {
            let mut usage = client.get(&["themes", &developer_name, "usage"], &[])?;
            if let Some(list) = usage.get_mut("templates").and_then(Value::as_array_mut) {
                list.retain(|t| {
                    if let Some(name) = &template {
                        return str_of(t, "developerName") == Some(name.as_str());
                    }
                    if unused {
                        let in_use = t.get("inUse").and_then(Value::as_bool).unwrap_or(true);
                        let built_in = t
                            .get("isBuiltInTemplate")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        return !in_use && !built_in;
                    }
                    true
                });
                if let Some(name) = &template
                    && list.is_empty()
                {
                    return Err(CliError::not_found(format!(
                        "Template '{name}' is not in theme '{developer_name}'."
                    ))
                    .with_hint("List them with `raytha web-template list --theme <theme>`."));
                }
            }
            Ok(usage)
        }
        Cmd::Activate { developer_name } => {
            client.post(&["themes", &developer_name, "set-active"], None)
        }
        Cmd::Pull(args) => sync::pull(client, &args),
        Cmd::Push(args) => sync::push(client, &args),
        Cmd::Media(m) => run_media(client, m),
    }
}

fn run_media(client: &Client, cmd: MediaCmd) -> Result<Value> {
    match cmd {
        MediaCmd::List { theme } => client.get(&["themes", &theme, "media"], &[]),
        MediaCmd::Upload { theme, path } => client.upload(&["themes", &theme, "media"], &path),
        MediaCmd::Delete { theme, id, yes } => {
            require_yes(yes, &format!("media item '{id}'"))?;
            client.delete(&["themes", &theme, "media", &id], &[])
        }
    }
}
