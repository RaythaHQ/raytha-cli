//! Site pages: free-form pages built from widgets placed in named sections of a web template.
//!
//! Widget edits are saved to a draft; `publish` makes them live.

use super::{ListArgs, list, require_yes, resolve_template, str_of};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{BodyArgs, json_value};
use clap::Subcommand;
use serde_json::{Map, Value, json};

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List site pages.
    List(ListArgs),
    /// Get a page with its sections and widgets (settings are returned as objects).
    Get { id: String },
    /// Create a page, optionally with route, sections of widgets, and publishing in one step.
    Create {
        #[arg(long)]
        title: String,
        /// Web template developer name in the ACTIVE theme (see `raytha web-template list`).
        #[arg(long)]
        template: Option<String>,
        /// Web template id, instead of --template.
        #[arg(long)]
        template_id: Option<String>,
        /// URL path for the page, for example `about` or `company/team`.
        #[arg(long)]
        route_path: Option<String>,
        /// Sections and widgets: {"main":[{"widgetType":"hero","settings":{...}}]}. JSON, @path, or -.
        #[arg(long, value_name = "JSON|@PATH|-")]
        sections: Option<String>,
        /// Leave the page as an unpublished draft.
        #[arg(long)]
        draft: bool,
        /// Make this the site's home page.
        #[arg(long)]
        home: bool,
    },
    /// Change title and/or template. Omitted flags keep their current value.
    Edit {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        template: Option<String>,
        #[arg(long)]
        template_id: Option<String>,
        /// Save as draft instead of publishing the change.
        #[arg(long)]
        draft: bool,
    },
    /// Change the page's URL path.
    Settings {
        id: String,
        #[arg(long)]
        route_path: String,
    },
    /// Replace or add whole sections of widgets (saved to draft).
    Sections {
        id: String,
        /// {"main":[...],"sidebar":[...]}. JSON, @path, or -.
        #[arg(long, value_name = "JSON|@PATH|-")]
        sections: String,
        /// Remove sections that are not listed.
        #[arg(long)]
        replace: bool,
        /// Publish after saving.
        #[arg(long)]
        publish: bool,
    },
    /// Publish the draft so it goes live.
    Publish { id: String },
    /// Take the page offline.
    Unpublish { id: String },
    /// Throw away unpublished draft changes.
    DiscardDraft { id: String },
    /// Make this page the home page.
    SetHome { id: String },
    /// Delete a page.
    Delete {
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// List widget types usable in the active theme, with their settings fields.
    WidgetDefinitions,
    /// Work with individual widgets.
    #[command(subcommand)]
    Widgets(WidgetsCmd),
}

#[derive(Subcommand, Debug)]
pub enum WidgetsCmd {
    /// Replace all widgets in one section (an empty list removes the section).
    Save {
        id: String,
        #[arg(long)]
        section: String,
        #[command(flatten)]
        body: BodyArgs,
        /// Publish after saving.
        #[arg(long)]
        publish: bool,
    },
    /// Change one widget's settings.
    Edit {
        id: String,
        widget_id: String,
        #[arg(long)]
        section: String,
        /// Settings object: JSON, @path, or -.
        #[arg(long, value_name = "JSON|@PATH|-")]
        settings: String,
        /// Overlay onto the widget's current settings instead of replacing them.
        #[arg(long)]
        merge: bool,
        /// Publish after saving.
        #[arg(long)]
        publish: bool,
    },
    /// Remove one widget.
    Delete {
        id: String,
        widget_id: String,
        #[arg(long)]
        section: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => {
            let mut res = list(client, &["sitepages"], &[], &args)?;
            if let Some(items) = res.get_mut("items").and_then(Value::as_array_mut) {
                items.iter_mut().for_each(expose_settings);
            }
            Ok(res)
        }
        Cmd::Get { id } => get(client, &id),
        Cmd::Create {
            title,
            template,
            template_id,
            route_path,
            sections,
            draft,
            home,
        } => {
            let template_id =
                resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                    .ok_or_else(template_required)?;
            let sections = sections.as_deref().map(json_value).transpose()?;
            if let Some(s) = &sections {
                validate_sections(s)?;
            }
            // Widgets are saved to a draft, so build as a draft and publish at the end.
            let as_draft = draft || sections.is_some();
            let body =
                json!({ "title": title, "saveAsDraft": as_draft, "templateId": template_id });
            let created = client.post(&["sitepages"], Some(&body))?;
            let id = str_of(&created, "id")
                .ok_or_else(|| CliError::server("unexpected_response", "Create returned no id."))?
                .to_string();
            if let Some(path) = route_path {
                client.put(
                    &["sitepages", &id, "settings"],
                    Some(&json!({ "routePath": path })),
                )?;
            }
            if let Some(s) = &sections {
                apply_sections(client, &id, s, false)?;
                if !draft {
                    client.put(&["sitepages", &id, "publish"], None)?;
                }
            }
            if home {
                client.put(&["sitepages", &id, "set-as-home-page"], None)?;
            }
            get(client, &id)
        }
        Cmd::Edit {
            id,
            title,
            template,
            template_id,
            draft,
        } => {
            let remote = client.get(&["sitepages", &id], &[])?;
            let template_id =
                resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                    .or_else(|| str_of(&remote, "webTemplateId").map(str::to_string))
                    .ok_or_else(template_required)?;
            let body = json!({
                "title": title.unwrap_or_else(|| str_of(&remote, "title").unwrap_or_default().to_string()),
                "saveAsDraft": draft,
                "templateId": template_id,
            });
            client.put(&["sitepages", &id], Some(&body))
        }
        Cmd::Settings { id, route_path } => client.put(
            &["sitepages", &id, "settings"],
            Some(&json!({ "routePath": route_path })),
        ),
        Cmd::Sections {
            id,
            sections,
            replace,
            publish,
        } => {
            let s = json_value(&sections)?;
            apply_sections(client, &id, &s, replace)?;
            if publish {
                client.put(&["sitepages", &id, "publish"], None)?;
            }
            get(client, &id)
        }
        Cmd::Publish { id } => client.put(&["sitepages", &id, "publish"], None),
        Cmd::Unpublish { id } => client.put(&["sitepages", &id, "unpublish"], None),
        Cmd::DiscardDraft { id } => client.put(&["sitepages", &id, "discard-draft"], None),
        Cmd::SetHome { id } => client.put(&["sitepages", &id, "set-as-home-page"], None),
        Cmd::Delete { id, yes } => {
            require_yes(yes, &format!("site page '{id}'"))?;
            client.delete(&["sitepages", &id], &[])
        }
        Cmd::WidgetDefinitions => client.get(&["sitepages", "widget-definitions"], &[]),
        Cmd::Widgets(w) => run_widgets(client, w),
    }
}

fn template_required() -> CliError {
    CliError::usage(
        "A web template is required: pass --template <developer_name> or --template-id <id>.",
    )
    .with_hint(
        "Find templates in the active theme with `raytha web-template list --theme <theme>` \
             (`raytha theme list` shows which theme is active). Page templates do not use \
             {% renderbody %}; base layouts do.",
    )
}

fn run_widgets(client: &Client, cmd: WidgetsCmd) -> Result<Value> {
    match cmd {
        WidgetsCmd::Save {
            id,
            section,
            body,
            publish,
        } => {
            let widgets = body.read()?.ok_or_else(|| {
                CliError::usage("Widgets are required: pass --data '[...]' or --file <path>.")
            })?;
            let widgets = widgets
                .as_array()
                .ok_or_else(|| CliError::usage("Widgets must be a JSON array."))?;
            save_section(client, &id, &section, widgets)?;
            if publish {
                client.put(&["sitepages", &id, "publish"], None)?;
            }
            get(client, &id)
        }
        WidgetsCmd::Edit {
            id,
            widget_id,
            section,
            settings,
            merge,
            publish,
        } => {
            let mut new_settings = match json_value(&settings)? {
                Value::Object(m) => m,
                _ => return Err(CliError::usage("--settings must be a JSON object.")),
            };
            if merge {
                let page = get(client, &id)?;
                let current = find_widget(&page, &section, &widget_id).ok_or_else(|| {
                    CliError::not_found(format!(
                        "Widget '{widget_id}' not found in section '{section}'."
                    ))
                    .with_hint("Run `raytha site-page get <id>` to see sections and widget ids.")
                })?;
                let mut merged = current
                    .get("settings")
                    .and_then(Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                merged.extend(new_settings);
                new_settings = merged;
            }
            let body = json!({
                "sectionName": section,
                "settingsJson": Value::Object(new_settings).to_string(),
            });
            client.put(&["sitepages", &id, "widgets", &widget_id], Some(&body))?;
            if publish {
                client.put(&["sitepages", &id, "publish"], None)?;
            }
            get(client, &id)
        }
        WidgetsCmd::Delete {
            id,
            widget_id,
            section,
            yes,
        } => {
            require_yes(yes, &format!("widget '{widget_id}'"))?;
            client.delete(
                &["sitepages", &id, "widgets", &widget_id],
                &[("sectionName", section)],
            )
        }
    }
}

pub fn get(client: &Client, id: &str) -> Result<Value> {
    let mut page = client.get(&["sitepages", id], &[])?;
    expose_settings(&mut page);
    Ok(page)
}

/// Replaces each widget's `settingsJson` string with a parsed `settings` object, in both
/// `widgets` (draft) and `publishedWidgets`. `sections`/`widgets save` accept `settings` back.
pub fn expose_settings(page: &mut Value) {
    for key in ["widgets", "publishedWidgets"] {
        let Some(sections) = page.get_mut(key).and_then(Value::as_object_mut) else {
            continue;
        };
        for widgets in sections.values_mut() {
            let Some(list) = widgets.as_array_mut() else {
                continue;
            };
            for w in list {
                let Some(obj) = w.as_object_mut() else {
                    continue;
                };
                if let Some(Value::String(s)) = obj.remove("settingsJson") {
                    let parsed = serde_json::from_str::<Value>(&s).unwrap_or(Value::String(s));
                    obj.insert("settings".into(), parsed);
                }
            }
        }
    }
}

fn find_widget<'a>(page: &'a Value, section: &str, widget_id: &str) -> Option<&'a Value> {
    page.get("widgets")?
        .get(section)?
        .as_array()?
        .iter()
        .find(|w| str_of(w, "id") == Some(widget_id))
}

/// Turns friendly widget input into the API shape: `settings` objects become the `settingsJson`
/// string, `type` is accepted for `widgetType`, and unset grid positions stack vertically.
pub fn normalize_widget(index: usize, widget: &Value) -> Result<Value> {
    let Some(src) = widget.as_object() else {
        return Err(CliError::usage(format!(
            "Widget #{index} must be a JSON object."
        )));
    };
    let mut out = Map::new();
    for (k, v) in src {
        match k.as_str() {
            "type" => {
                out.entry("widgetType").or_insert(v.clone());
            }
            "settings" => {
                out.insert("settingsJson".into(), json!(v.to_string()));
            }
            "settingsJson" => {
                let s = match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                out.entry("settingsJson").or_insert(json!(s));
            }
            _ => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    if !out.contains_key("widgetType") {
        return Err(CliError::usage(format!(
            "Widget #{index} needs a widgetType (see `raytha site-page widget-definitions`)."
        )));
    }
    out.entry("settingsJson").or_insert(json!("{}"));
    out.entry("row").or_insert(json!(index));
    out.entry("column").or_insert(json!(0));
    out.entry("columnSpan").or_insert(json!(12));
    Ok(Value::Object(out))
}

fn save_section(client: &Client, id: &str, section: &str, widgets: &[Value]) -> Result<Value> {
    let normalized: Vec<Value> = widgets
        .iter()
        .enumerate()
        .map(|(i, w)| normalize_widget(i, w))
        .collect::<Result<_>>()?;
    let body = json!({ "sectionName": section, "widgets": normalized });
    client.put(&["sitepages", id, "widgets"], Some(&body))
}

/// Checks a whole `{section: [widget, ...]}` payload so nothing is written when part of it is bad.
fn validate_sections(sections: &Value) -> Result<&serde_json::Map<String, Value>> {
    let map = sections.as_object().ok_or_else(|| {
        CliError::usage("Sections must be a JSON object mapping section name to a widget array.")
            .with_hint("Example: {\"main\":[{\"widgetType\":\"hero\",\"settings\":{\"headline\":\"Hi\"}}]}")
    })?;
    for (name, widgets) in map {
        let list = widgets.as_array().ok_or_else(|| {
            CliError::usage(format!("Section '{name}' must be an array of widgets."))
        })?;
        for (i, w) in list.iter().enumerate() {
            normalize_widget(i, w)?;
        }
    }
    Ok(map)
}

fn apply_sections(client: &Client, id: &str, sections: &Value, replace: bool) -> Result<()> {
    let map = validate_sections(sections)?;
    let existing: Vec<String> = if replace {
        let page = client.get(&["sitepages", id], &[])?;
        page.get("widgets")
            .and_then(Value::as_object)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    } else {
        vec![]
    };
    for (name, widgets) in map {
        save_section(client, id, name, widgets.as_array().unwrap())?;
    }
    for name in existing {
        if !map.contains_key(&name) {
            save_section(client, id, &name, &[])?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_friendly_widgets() {
        let w =
            normalize_widget(2, &json!({"type": "hero", "settings": {"headline": "Hi"}})).unwrap();
        assert_eq!(w["widgetType"], "hero");
        assert_eq!(w["settingsJson"], "{\"headline\":\"Hi\"}");
        assert_eq!(w["row"], 2);
        assert_eq!(w["columnSpan"], 12);
        assert!(w.get("settings").is_none());
    }

    #[test]
    fn rejects_widget_without_type() {
        assert!(normalize_widget(0, &json!({"settings": {}})).is_err());
    }

    #[test]
    fn exposes_settings_as_objects() {
        let mut page = json!({"widgets": {"main": [{"id": "a", "settingsJson": "{\"x\":1}"}]}});
        expose_settings(&mut page);
        assert_eq!(page["widgets"]["main"][0]["settings"]["x"], 1);
        assert!(page["widgets"]["main"][0].get("settingsJson").is_none());
    }
}
