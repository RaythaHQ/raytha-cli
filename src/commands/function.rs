//! Raytha Functions: the JavaScript that answers an HTTP request, renders in Liquid, or reacts to
//! a content item event.

use super::{ListArgs, list, require_yes, str_of, write_file};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{ContentSrc, humanize, to_developer_name};
use clap::{Args, Subcommand};
use serde_json::{Map, Value, json};
use std::path::PathBuf;

const TRIGGERS: &str = "http_request, liquid_template, content_item_created, content_item_updated, content_item_deleted";

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List functions.
    List(ListArgs),
    /// Get one function, including its code.
    Get {
        developer_name: String,
        /// Write the code to this file instead of returning it inline.
        #[arg(long, value_name = "PATH")]
        out: Option<PathBuf>,
    },
    /// Create a function from a JavaScript file.
    Create {
        /// Developer name (lowercase letters, digits, underscores; dots allowed, e.g. `llms.txt`).
        developer_name: String,
        /// Display name (defaults to a humanized developer name).
        #[arg(long)]
        name: Option<String>,
        /// What runs it: http_request, liquid_template, content_item_created, content_item_updated
        /// or content_item_deleted.
        #[arg(long, value_name = "TRIGGER")]
        trigger: String,
        #[command(flatten)]
        src: ContentSrc,
        #[command(flatten)]
        opts: Opts,
    },
    /// Edit a function. Omitted flags keep their current value; changed code keeps a revision.
    Edit {
        developer_name: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_name = "TRIGGER")]
        trigger: Option<String>,
        #[command(flatten)]
        src: ContentSrc,
        #[command(flatten)]
        opts: Opts,
    },
    /// Delete a function.
    Delete {
        developer_name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Earlier versions of a function's code, newest first.
    Revisions {
        developer_name: String,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Restore a revision's code (the current code is saved as a new revision first).
    Revert {
        developer_name: String,
        revision_id: String,
    },
}

#[derive(Args, Debug, Default, Clone)]
pub struct Opts {
    /// Turn the function on or off. A new function is active unless `--active false`.
    #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
    pub active: Option<bool>,
    /// Public path an http_request function also answers at, such as `llms.txt` ("" removes it).
    #[arg(long, value_name = "PATH")]
    pub route_path: Option<String>,
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => list(client, &["functions"], &[], &args),
        Cmd::Get {
            developer_name,
            out,
        } => {
            let mut f = get(client, &developer_name)?;
            if let Some(path) = out {
                let code = str_of(&f, "code").unwrap_or_default().to_string();
                write_file(&path, &code)?;
                if let Some(o) = f.as_object_mut() {
                    o.remove("code");
                    o.insert("codeFile".into(), json!(path.display().to_string()));
                }
            }
            Ok(f)
        }
        Cmd::Create {
            developer_name,
            name,
            trigger,
            src,
            opts,
        } => {
            let code = src.read()?.ok_or_else(|| {
                CliError::usage("Function code is required: pass --file <path> or --content.")
            })?;
            let trigger = check_trigger(&trigger)?;
            let dev = to_developer_name_dotted(&developer_name);
            let mut b = Map::new();
            b.insert("name".into(), json!(name.unwrap_or_else(|| humanize(&dev))));
            b.insert("developerName".into(), json!(dev));
            b.insert("triggerType".into(), json!(trigger));
            b.insert("isActive".into(), json!(opts.active.unwrap_or(true)));
            b.insert("code".into(), json!(code));
            if let Some(p) = opts.route_path {
                b.insert("routePath".into(), json!(p));
            }
            client.post(&["functions"], Some(&Value::Object(b)))
        }
        Cmd::Edit {
            developer_name,
            name,
            trigger,
            src,
            opts,
        } => {
            let current = get(client, &developer_name)?;
            let trigger = match trigger {
                Some(t) => check_trigger(&t)?,
                None => current
                    .get("triggerType")
                    .and_then(|t| str_of(t, "developerName"))
                    .unwrap_or("http_request")
                    .to_string(),
            };
            let code = match src.read()? {
                Some(c) => c,
                None => str_of(&current, "code").unwrap_or_default().to_string(),
            };
            let body = json!({
                "name": name.or_else(|| str_of(&current, "name").map(str::to_string)),
                "triggerType": trigger,
                "isActive": opts
                    .active
                    .or_else(|| current.get("isActive").and_then(Value::as_bool))
                    .unwrap_or(true),
                "code": code,
                "routePath": opts
                    .route_path
                    .or_else(|| str_of(&current, "routePath").map(str::to_string))
                    .unwrap_or_default(),
            });
            client.put(&["functions", &developer_name], Some(&body))
        }
        Cmd::Delete {
            developer_name,
            yes,
        } => {
            require_yes(yes, &format!("function '{developer_name}'"))?;
            client.delete(&["functions", &developer_name], &[])
        }
        Cmd::Revisions {
            developer_name,
            list: args,
        } => list(
            client,
            &["functions", &developer_name, "revisions"],
            &[],
            &args,
        ),
        Cmd::Revert {
            developer_name,
            revision_id,
        } => client.post(
            &[
                "functions",
                &developer_name,
                "revisions",
                &revision_id,
                "revert",
            ],
            None,
        ),
    }
}

fn get(client: &Client, developer_name: &str) -> Result<Value> {
    client.get(&["functions", developer_name], &[])
}

fn check_trigger(t: &str) -> Result<String> {
    let t = t.trim().to_lowercase().replace('-', "_");
    if TRIGGERS.split(", ").any(|k| k == t) {
        Ok(t)
    } else {
        Err(CliError::usage(format!("Unknown trigger '{t}'."))
            .with_hint(format!("Use one of: {TRIGGERS}.")))
    }
}

/// Function developer names may contain dots (`llms.txt`); everything else normalizes as usual.
fn to_developer_name_dotted(s: &str) -> String {
    s.split('.')
        .map(to_developer_name)
        .collect::<Vec<_>>()
        .join(".")
}
