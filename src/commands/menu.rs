//! Navigation menus and their items.

use super::{ListArgs, list, require_yes, str_of};
use crate::client::Client;
use crate::error::Result;
use crate::input::{humanize, to_developer_name};
use clap::Subcommand;
use serde_json::{Map, Value, json};

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List menus.
    List(ListArgs),
    /// Get a menu with its nested items.
    Get { developer_name: String },
    /// Create a menu.
    Create {
        developer_name: String,
        #[arg(long)]
        label: Option<String>,
    },
    /// Rename a menu.
    Edit {
        developer_name: String,
        #[arg(long)]
        label: String,
    },
    /// Delete a menu.
    Delete {
        developer_name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Make this the main menu (available as `CurrentOrganization.MainMenu` in templates).
    SetMain { developer_name: String },
    /// Manage items in a menu.
    #[command(subcommand)]
    Items(ItemsCmd),
}

#[derive(Subcommand, Debug)]
pub enum ItemsCmd {
    /// List a menu's items.
    List { menu: String },
    /// Add an item.
    Create {
        menu: String,
        #[arg(long)]
        label: String,
        /// Link target, for example `/about` or `https://example.com`.
        #[arg(long)]
        link: String,
        /// Parent item id, to nest under another item.
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        new_tab: bool,
        #[arg(long)]
        disabled: bool,
        #[arg(long)]
        css_class: Option<String>,
    },
    /// Edit an item. Omitted flags keep their current value.
    Edit {
        menu: String,
        item_id: String,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        link: Option<String>,
        /// Parent item id ("" moves it to the top level).
        #[arg(long)]
        parent: Option<String>,
        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
        new_tab: Option<bool>,
        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
        disabled: Option<bool>,
        #[arg(long)]
        css_class: Option<String>,
    },
    /// Delete an item.
    Delete {
        menu: String,
        item_id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Move an item to a new position among its siblings.
    Reorder {
        menu: String,
        item_id: String,
        /// New 1-based position.
        #[arg(long)]
        position: i64,
    },
}

fn menu_id(client: &Client, menu: &str) -> Result<String> {
    let m = client.get(&["menus", menu], &[])?;
    Ok(str_of(&m, "id").unwrap_or_default().to_string())
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => list(client, &["menus"], &[], &args),
        Cmd::Get { developer_name } => {
            let mut menu = client.get(&["menus", &developer_name], &[])?;
            if let Some(o) = menu.as_object_mut()
                && !o.contains_key("items")
                && !o.contains_key("menuItems")
            {
                let items = client.get(&["menus", &developer_name, "menu-items"], &[])?;
                o.insert("items".into(), items);
            }
            Ok(menu)
        }
        Cmd::Create {
            developer_name,
            label,
        } => {
            let dev = to_developer_name(&developer_name);
            let body = json!({
                "developerName": dev,
                "label": label.unwrap_or_else(|| humanize(&dev)),
            });
            let mut res = client.post(&["menus"], Some(&body))?;
            if let Some(o) = res.as_object_mut() {
                o.insert("developerName".into(), json!(dev));
            }
            Ok(res)
        }
        Cmd::Edit {
            developer_name,
            label,
        } => client.put(
            &["menus", &developer_name],
            Some(&json!({ "label": label })),
        ),
        Cmd::Delete {
            developer_name,
            yes,
        } => {
            require_yes(yes, &format!("menu '{developer_name}'"))?;
            client.delete(&["menus", &developer_name], &[])
        }
        Cmd::SetMain { developer_name } => {
            client.post(&["menus", &developer_name, "set-main"], None)
        }
        Cmd::Items(i) => run_items(client, i),
    }
}

fn run_items(client: &Client, cmd: ItemsCmd) -> Result<Value> {
    match cmd {
        ItemsCmd::List { menu } => client.get(&["menus", &menu, "menu-items"], &[]),
        ItemsCmd::Create {
            menu,
            label,
            link,
            parent,
            new_tab,
            disabled,
            css_class,
        } => {
            let mut b = Map::new();
            b.insert("navigationMenuId".into(), json!(menu_id(client, &menu)?));
            b.insert("label".into(), json!(label));
            b.insert("url".into(), json!(link));
            b.insert("openInNewTab".into(), json!(new_tab));
            b.insert("isDisabled".into(), json!(disabled));
            if let Some(c) = css_class {
                b.insert("cssClassName".into(), json!(c));
            }
            if let Some(p) = parent.filter(|p| !p.is_empty()) {
                b.insert("parentNavigationMenuItemId".into(), json!(p));
            }
            client.post(&["menus", &menu, "menu-items"], Some(&Value::Object(b)))
        }
        ItemsCmd::Edit {
            menu,
            item_id,
            label,
            link,
            parent,
            new_tab,
            disabled,
            css_class,
        } => {
            let item = client.get(&["menus", &menu, "menu-items", &item_id], &[])?;
            let s = |new: Option<String>, key: &str| {
                new.unwrap_or_else(|| str_of(&item, key).unwrap_or_default().to_string())
            };
            let flag = |new: Option<bool>, key: &str| {
                new.unwrap_or_else(|| item.get(key).and_then(Value::as_bool).unwrap_or(false))
            };
            let mut b = Map::new();
            b.insert("navigationMenuId".into(), json!(menu_id(client, &menu)?));
            b.insert("label".into(), json!(s(label, "label")));
            b.insert("url".into(), json!(s(link, "url")));
            b.insert("openInNewTab".into(), json!(flag(new_tab, "openInNewTab")));
            b.insert("isDisabled".into(), json!(flag(disabled, "isDisabled")));
            let css = s(css_class, "cssClassName");
            if !css.is_empty() {
                b.insert("cssClassName".into(), json!(css));
            }
            let parent_id = match parent {
                Some(p) if p.is_empty() => None,
                Some(p) => Some(p),
                None => str_of(&item, "parentNavigationMenuItemId")
                    .filter(|p| *p != "AAAAAAAAAAAAAAAAAAAAAA")
                    .map(str::to_string),
            };
            if let Some(p) = parent_id {
                b.insert("parentNavigationMenuItemId".into(), json!(p));
            }
            client.put(
                &["menus", &menu, "menu-items", &item_id],
                Some(&Value::Object(b)),
            )
        }
        ItemsCmd::Delete { menu, item_id, yes } => {
            require_yes(yes, &format!("menu item '{item_id}'"))?;
            client.delete(&["menus", &menu, "menu-items", &item_id], &[])
        }
        ItemsCmd::Reorder {
            menu,
            item_id,
            position,
        } => client.post(
            &["menus", &menu, "menu-items", &item_id, "reorder"],
            Some(&json!({ "ordinal": position })),
        ),
    }
}
