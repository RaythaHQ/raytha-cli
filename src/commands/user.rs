//! Public users and user groups (site visitors who log in; not administrators).

use super::{ListArgs, list, require_yes, str_of};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{humanize, read_text, to_developer_name};
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand, Debug)]
pub enum UserCmd {
    /// List users.
    List(ListArgs),
    /// Get a user by id.
    Get { id: String },
    /// Create a user.
    Create {
        #[arg(long)]
        email: String,
        #[arg(long)]
        first_name: String,
        #[arg(long)]
        last_name: String,
        /// Group ids the user belongs to (comma separated).
        #[arg(long, value_delimiter = ',')]
        groups: Vec<String>,
        /// Email the user a login invitation.
        #[arg(long)]
        send_email: bool,
    },
    /// Edit a user. Omitted flags keep their current value.
    Edit {
        id: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        first_name: Option<String>,
        #[arg(long)]
        last_name: Option<String>,
        /// Replacement group ids (comma separated; "" removes all groups).
        #[arg(long, value_delimiter = ',')]
        groups: Option<Vec<String>>,
    },
    /// Delete a user.
    Delete {
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Set a user's password.
    Password {
        id: String,
        /// Read the new password from stdin (preferred; avoids shell history).
        #[arg(long)]
        password_stdin: bool,
        /// The new password as an argument.
        #[arg(long)]
        password: Option<String>,
        /// Email the user about the change.
        #[arg(long)]
        send_email: bool,
    },
    /// Activate or deactivate a user.
    SetActive {
        id: String,
        /// `true` to activate, `false` to deactivate.
        #[arg(long, action = clap::ArgAction::Set)]
        active: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum GroupCmd {
    /// List user groups.
    List(ListArgs),
    /// Get a group by id.
    Get { id: String },
    /// Create a group.
    Create {
        developer_name: String,
        #[arg(long)]
        label: Option<String>,
    },
    /// Rename a group.
    Edit {
        id: String,
        #[arg(long)]
        label: String,
    },
    /// Delete a group.
    Delete {
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
}

fn group_ids(user: &Value) -> Vec<Value> {
    user.get("userGroups")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|g| match g {
                    Value::String(s) => Some(json!(s)),
                    Value::Object(_) => str_of(g, "id").map(|s| json!(s)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn run_users(client: &Client, cmd: UserCmd) -> Result<Value> {
    match cmd {
        UserCmd::List(args) => list(client, &["users"], &[], &args),
        UserCmd::Get { id } => client.get(&["users", &id], &[]),
        UserCmd::Create {
            email,
            first_name,
            last_name,
            groups,
            send_email,
        } => client.post(
            &["users"],
            Some(&json!({
                "emailAddress": email,
                "firstName": first_name,
                "lastName": last_name,
                "userGroups": groups,
                "sendEmail": send_email,
            })),
        ),
        UserCmd::Edit {
            id,
            email,
            first_name,
            last_name,
            groups,
        } => {
            let remote = client.get(&["users", &id], &[])?;
            let s = |new: Option<String>, key: &str| {
                new.unwrap_or_else(|| str_of(&remote, key).unwrap_or_default().to_string())
            };
            let groups: Vec<Value> = match groups {
                Some(g) => g
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| json!(s))
                    .collect(),
                None => group_ids(&remote),
            };
            client.put(
                &["users", &id],
                Some(&json!({
                    "emailAddress": s(email, "emailAddress"),
                    "firstName": s(first_name, "firstName"),
                    "lastName": s(last_name, "lastName"),
                    "userGroups": groups,
                })),
            )
        }
        UserCmd::Delete { id, yes } => {
            require_yes(yes, &format!("user '{id}'"))?;
            client.delete(&["users", &id], &[])
        }
        UserCmd::Password {
            id,
            password_stdin,
            password,
            send_email,
        } => {
            let pw = match (password_stdin, password) {
                (true, None) => read_text("-")?.trim_end_matches(['\n', '\r']).to_string(),
                (false, Some(p)) => p,
                (true, Some(_)) => {
                    return Err(CliError::usage(
                        "Use either --password-stdin or --password.",
                    ));
                }
                (false, None) => {
                    return Err(CliError::usage(
                        "A password is required: --password-stdin (preferred) or --password.",
                    ));
                }
            };
            client.put(
                &["users", &id, "password"],
                Some(&json!({ "newPassword": pw, "sendEmail": send_email })),
            )
        }
        UserCmd::SetActive { id, active } => client.put(
            &["users", &id, "is-active"],
            Some(&json!({ "isActive": active })),
        ),
    }
}

pub fn run_groups(client: &Client, cmd: GroupCmd) -> Result<Value> {
    match cmd {
        GroupCmd::List(args) => list(client, &["usergroups"], &[], &args),
        GroupCmd::Get { id } => client.get(&["usergroups", &id], &[]),
        GroupCmd::Create {
            developer_name,
            label,
        } => {
            let dev = to_developer_name(&developer_name);
            client.post(
                &["usergroups"],
                Some(&json!({
                    "developerName": dev,
                    "label": label.unwrap_or_else(|| humanize(&dev)),
                })),
            )
        }
        GroupCmd::Edit { id, label } => {
            client.put(&["usergroups", &id], Some(&json!({ "label": label })))
        }
        GroupCmd::Delete { id, yes } => {
            require_yes(yes, &format!("user group '{id}'"))?;
            client.delete(&["usergroups", &id], &[])
        }
    }
}
