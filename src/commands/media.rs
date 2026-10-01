//! Site media library (uploaded images and files used by content and pages).
//! Theme-scoped media lives under `raytha theme media`.

use super::{ListArgs, list, require_yes, str_of};
use crate::client::Client;
use crate::error::Result;
use clap::Subcommand;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List media items (needs ManageMediaItems).
    List(ListArgs),
    /// Resolve an object key to a download URL.
    GetUrl { object_key: String },
    /// Delete a media item (content that still references its object key will show a broken file).
    Delete {
        object_key: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Upload a file. Returns its object key and URL.
    Upload {
        /// Local file path.
        path: PathBuf,
    },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => list(client, &["mediaitems"], &[], &args),
        Cmd::GetUrl { object_key } => {
            let url = client.get(&["mediaitems", &object_key], &[])?;
            Ok(json!({ "objectKey": object_key, "url": url }))
        }
        Cmd::Delete { object_key, yes } => {
            require_yes(yes, &format!("media item '{object_key}'"))?;
            client.delete(&["mediaitems", &object_key], &[])
        }
        Cmd::Upload { path } => {
            let key = upload_key(client, &path)?;
            let url = client.get(&["mediaitems", &key], &[]).ok();
            Ok(json!({ "objectKey": key, "url": url }))
        }
    }
}

/// Uploads a file to the media library and returns its object key.
pub fn upload_key(client: &Client, path: &std::path::Path) -> Result<String> {
    let uploaded = client.upload(&["mediaitems", "upload-direct"], path)?;
    uploaded
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| str_of(&uploaded, "objectKey"))
        .map(str::to_string)
        .ok_or_else(|| {
            crate::error::CliError::server("unexpected_response", "Upload returned no object key.")
        })
}
