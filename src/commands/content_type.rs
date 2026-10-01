//! Content types (the schema), their fields, and their views (filtered/sorted public lists).

use super::{ListArgs, list, require_yes, resolve_template, str_of};
use crate::client::Client;
use crate::error::{CliError, Result};
use crate::input::{BodyArgs, json_value, to_developer_name};
use clap::Subcommand;
use serde_json::{Map, Value, json};

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// List content types.
    List(ListArgs),
    /// Get a content type with its fields.
    Get { developer_name: String },
    /// Create a content type (it starts with `title` and `content` fields and a public list view).
    Create {
        developer_name: String,
        /// Plural label, for example "Blog posts".
        #[arg(long)]
        label_plural: String,
        /// Singular label, for example "Blog post".
        #[arg(long)]
        label_singular: String,
        #[arg(long)]
        description: Option<String>,
        /// URL pattern for new items: {PrimaryField}, {CurrentYear}.
        #[arg(long, default_value = "{PrimaryField}")]
        route_template: String,
    },
    /// Edit labels, description, route template, or the primary field.
    Edit {
        developer_name: String,
        #[arg(long)]
        label_plural: Option<String>,
        #[arg(long)]
        label_singular: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        route_template: Option<String>,
        /// Developer name of the field to use as the primary field (item title).
        #[arg(long)]
        primary_field: Option<String>,
    },
    /// Delete a content type with all its fields, views and items.
    Delete {
        developer_name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// List the field types a content type field can use.
    FieldTypes,
    /// Manage fields.
    #[command(subcommand)]
    Fields(FieldsCmd),
    /// Manage views (public filtered/sorted lists).
    #[command(subcommand)]
    Views(ViewsCmd),
}

#[derive(Subcommand, Debug)]
pub enum FieldsCmd {
    /// Add a field.
    Create {
        content_type: String,
        /// Field developer name.
        name: String,
        /// Field type, from `raytha content-type field-types` (for example single_line_text).
        #[arg(long = "type")]
        field_type: String,
        #[arg(long)]
        label: String,
        #[arg(long)]
        required: bool,
        #[arg(long)]
        description: Option<String>,
        /// Choices for dropdown/radio/multiple-select, comma separated labels.
        #[arg(long, value_delimiter = ',')]
        choices: Option<Vec<String>>,
        /// Choices as JSON [{"label":"A","developerName":"a"}], inline/@path/-.
        #[arg(long, value_name = "JSON|@PATH|-")]
        choices_json: Option<String>,
        /// For one_to_one_relationship: the related content type's developer name.
        #[arg(long)]
        related_content_type: Option<String>,
        /// For repeater fields: sub-field definitions as JSON.
        #[arg(long, value_name = "JSON|@PATH|-")]
        sub_fields: Option<String>,
    },
    /// Edit a field. Omitted flags keep their current value.
    Edit {
        content_type: String,
        name: String,
        #[arg(long)]
        label: Option<String>,
        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
        required: Option<bool>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_delimiter = ',')]
        choices: Option<Vec<String>>,
        #[arg(long, value_name = "JSON|@PATH|-")]
        choices_json: Option<String>,
        #[arg(long, value_name = "JSON|@PATH|-")]
        sub_fields: Option<String>,
    },
    /// Delete a field (its developer name cannot be reused).
    Delete {
        content_type: String,
        name: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Move a field to a new position.
    Reorder {
        content_type: String,
        name: String,
        /// New 1-based position.
        #[arg(long)]
        position: i64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ViewsCmd {
    /// List views.
    List {
        content_type: String,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Get a view by id.
    Get { content_type: String, id: String },
    /// Create a view.
    Create {
        content_type: String,
        name: String,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// Copy filter, sort and columns from this view id.
        #[arg(long)]
        duplicate_from: Option<String>,
    },
    /// Public settings: publishing, URL path, template, page size.
    Settings {
        content_type: String,
        id: String,
        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
        published: Option<bool>,
        #[arg(long)]
        route_path: Option<String>,
        /// Web template developer name (ACTIVE theme).
        #[arg(long)]
        template: Option<String>,
        #[arg(long)]
        template_id: Option<String>,
        #[arg(long)]
        page_size: Option<i64>,
        #[arg(long)]
        max_page_size: Option<i64>,
        #[arg(long, num_args = 0..=1, default_missing_value = "true", value_name = "BOOL")]
        ignore_client_params: Option<bool>,
    },
    /// Set the view's filter conditions (JSON array; `[]` clears).
    Filter {
        content_type: String,
        id: String,
        /// Conditions: [{"id":"<uuid>","type":"filter_condition","groupOperator":"AND","field":"title","conditionOperator":"contains","value":"x"}].
        #[arg(long, value_name = "JSON|@PATH|-")]
        conditions: String,
    },
    /// Set a column's sort and visibility.
    Sort {
        content_type: String,
        id: String,
        #[command(flatten)]
        body: BodyArgs,
    },
    /// Reorder a sort column.
    SortReorder {
        content_type: String,
        id: String,
        #[command(flatten)]
        body: BodyArgs,
    },
    /// Delete a view.
    Delete {
        content_type: String,
        id: String,
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Make this view the site home page.
    SetHome { content_type: String, id: String },
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::List(args) => list(client, &["contenttypes"], &[], &args),
        Cmd::Get { developer_name } => client.get(&["contenttypes", &developer_name], &[]),
        Cmd::Delete {
            developer_name,
            yes,
        } => {
            require_yes(
                yes,
                &format!("content type '{developer_name}' with all its items"),
            )?;
            client.delete(&["contenttypes", &developer_name], &[])
        }
        Cmd::Create {
            developer_name,
            label_plural,
            label_singular,
            description,
            route_template,
        } => {
            let dev = to_developer_name(&developer_name);
            let body = json!({
                "developerName": dev,
                "labelPlural": label_plural,
                "labelSingular": label_singular,
                "description": description.unwrap_or_else(|| label_plural.clone()),
                "defaultRouteTemplate": route_template,
            });
            let mut res = client.post(&["contenttypes"], Some(&body))?;
            if let Some(o) = res.as_object_mut() {
                o.insert("developerName".into(), json!(dev));
            }
            Ok(res)
        }
        Cmd::Edit {
            developer_name,
            label_plural,
            label_singular,
            description,
            route_template,
            primary_field,
        } => {
            let remote = client.get(&["contenttypes", &developer_name], &[])?;
            let primary_id = match primary_field {
                Some(name) => field_id(&remote, &name)?,
                None => str_of(&remote, "primaryFieldId")
                    .unwrap_or_default()
                    .to_string(),
            };
            let pick = |new: Option<String>, key: &str| {
                new.unwrap_or_else(|| str_of(&remote, key).unwrap_or_default().to_string())
            };
            let body = json!({
                "labelPlural": pick(label_plural, "labelPlural"),
                "labelSingular": pick(label_singular, "labelSingular"),
                "description": pick(description, "description"),
                "defaultRouteTemplate": pick(route_template, "defaultRouteTemplate"),
                "primaryFieldId": primary_id,
            });
            client.put(&["contenttypes", &developer_name], Some(&body))
        }
        Cmd::FieldTypes => client.get(&["contenttypes", "field-types"], &[]),
        Cmd::Fields(f) => run_fields(client, f),
        Cmd::Views(v) => run_views(client, v),
    }
}

/// The id of a field, looked up by developer name in a content type document.
fn field_id(content_type: &Value, name: &str) -> Result<String> {
    find_field(content_type, name)
        .and_then(|f| str_of(f, "id"))
        .map(str::to_string)
        .ok_or_else(|| {
            let known: Vec<&str> = fields_of(content_type)
                .iter()
                .filter_map(|f| str_of(f, "developerName"))
                .collect();
            CliError::not_found(format!("Field '{name}' not found on this content type."))
                .with_hint(format!("Known fields: {}", known.join(", ")))
        })
}

fn fields_of(content_type: &Value) -> Vec<&Value> {
    content_type
        .get("contentTypeFields")
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn find_field<'a>(content_type: &'a Value, name: &str) -> Option<&'a Value> {
    fields_of(content_type)
        .into_iter()
        .find(|f| str_of(f, "developerName") == Some(name))
}

fn choices_value(list: &Option<Vec<String>>, json_arg: &Option<String>) -> Result<Option<Value>> {
    if let Some(j) = json_arg {
        let v = json_value(j)?;
        if !v.is_array() {
            return Err(CliError::usage("--choices-json must be a JSON array."));
        }
        return Ok(Some(v));
    }
    Ok(list.as_ref().map(|labels| {
        Value::Array(
            labels
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    json!({
                        "label": l.trim(),
                        "developerName": to_developer_name(l),
                        "disabled": false
                    })
                })
                .collect(),
        )
    }))
}

fn run_fields(client: &Client, cmd: FieldsCmd) -> Result<Value> {
    match cmd {
        FieldsCmd::Create {
            content_type,
            name,
            field_type,
            label,
            required,
            description,
            choices,
            choices_json,
            related_content_type,
            sub_fields,
        } => {
            let mut b = Map::new();
            b.insert("developerName".into(), json!(name));
            b.insert("fieldType".into(), json!(field_type));
            b.insert("label".into(), json!(label));
            b.insert("isRequired".into(), json!(required));
            b.insert("description".into(), json!(description.unwrap_or_default()));
            if let Some(c) = choices_value(&choices, &choices_json)? {
                b.insert("choices".into(), c);
            }
            if let Some(related) = related_content_type {
                let ct = client.get(&["contenttypes", &related], &[])?;
                if let Some(id) = str_of(&ct, "id") {
                    b.insert("relatedContentTypeId".into(), json!(id));
                }
            }
            if let Some(s) = sub_fields {
                b.insert("subFields".into(), json_value(&s)?);
            }
            client.post(
                &["contenttypes", &content_type, "fields"],
                Some(&Value::Object(b)),
            )
        }
        FieldsCmd::Edit {
            content_type,
            name,
            label,
            required,
            description,
            choices,
            choices_json,
            sub_fields,
        } => {
            let ct = client.get(&["contenttypes", &content_type], &[])?;
            let id = field_id(&ct, &name)?;
            let current = find_field(&ct, &name).cloned().unwrap_or(Value::Null);
            let mut b = Map::new();
            b.insert(
                "label".into(),
                json!(
                    label.unwrap_or_else(|| str_of(&current, "label")
                        .unwrap_or_default()
                        .to_string())
                ),
            );
            b.insert(
                "isRequired".into(),
                json!(required.unwrap_or_else(|| {
                    current
                        .get("isRequired")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })),
            );
            b.insert(
                "description".into(),
                json!(description.unwrap_or_else(|| {
                    str_of(&current, "description")
                        .unwrap_or_default()
                        .to_string()
                })),
            );
            let choices = choices_value(&choices, &choices_json)?
                .or_else(|| current.get("choices").filter(|c| c.is_array()).cloned());
            if let Some(c) = choices {
                b.insert("choices".into(), c);
            }
            let sub = match sub_fields {
                Some(s) => Some(json_value(&s)?),
                None => current.get("subFields").filter(|c| c.is_array()).cloned(),
            };
            if let Some(s) = sub {
                b.insert("subFields".into(), s);
            }
            client.put(
                &["contenttypes", &content_type, "fields", &id],
                Some(&Value::Object(b)),
            )
        }
        FieldsCmd::Delete {
            content_type,
            name,
            yes,
        } => {
            require_yes(yes, &format!("field '{name}' and all its data"))?;
            let ct = client.get(&["contenttypes", &content_type], &[])?;
            let id = field_id(&ct, &name)?;
            client.delete(&["contenttypes", &content_type, "fields", &id], &[])
        }
        FieldsCmd::Reorder {
            content_type,
            name,
            position,
        } => {
            let ct = client.get(&["contenttypes", &content_type], &[])?;
            let id = field_id(&ct, &name)?;
            client.post(
                &["contenttypes", &content_type, "fields", &id, "reorder"],
                Some(&json!({ "newFieldOrder": position })),
            )
        }
    }
}

fn run_views(client: &Client, cmd: ViewsCmd) -> Result<Value> {
    match cmd {
        ViewsCmd::List {
            content_type,
            list: args,
        } => list(
            client,
            &["contenttypes", &content_type, "views"],
            &[],
            &args,
        ),
        ViewsCmd::Get { content_type, id } => {
            client.get(&["contenttypes", &content_type, "views", &id], &[])
        }
        ViewsCmd::Create {
            content_type,
            name,
            label,
            description,
            duplicate_from,
        } => {
            let mut b = Map::new();
            b.insert("developerName".into(), json!(to_developer_name(&name)));
            b.insert(
                "label".into(),
                json!(label.unwrap_or_else(|| crate::input::humanize(&name))),
            );
            b.insert("description".into(), json!(description.unwrap_or_default()));
            if let Some(d) = duplicate_from {
                b.insert("duplicateFromId".into(), json!(d));
            }
            client.post(
                &["contenttypes", &content_type, "views"],
                Some(&Value::Object(b)),
            )
        }
        ViewsCmd::Settings {
            content_type,
            id,
            published,
            route_path,
            template,
            template_id,
            page_size,
            max_page_size,
            ignore_client_params,
        } => {
            let view = client.get(&["contenttypes", &content_type, "views", &id], &[])?;
            let template_id =
                resolve_template(client, template.as_deref(), template_id.as_deref(), None)?
                    .or_else(|| {
                        str_of(&view, "webTemplateId")
                            .or_else(|| view.get("webTemplate").and_then(|t| str_of(t, "id")))
                            .map(str::to_string)
                    });
            let num =
                |key: &str, default: i64| view.get(key).and_then(Value::as_i64).unwrap_or(default);
            let mut b = Map::new();
            b.insert(
                "isPublished".into(),
                json!(published.unwrap_or_else(|| {
                    view.get("isPublished")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })),
            );
            b.insert(
                "routePath".into(),
                json!(
                    route_path.unwrap_or_else(|| str_of(&view, "routePath")
                        .unwrap_or_default()
                        .to_string())
                ),
            );
            if let Some(t) = template_id {
                b.insert("templateId".into(), json!(t));
            }
            b.insert(
                "defaultNumberOfItemsPerPage".into(),
                json!(page_size.unwrap_or_else(|| num("defaultNumberOfItemsPerPage", 50))),
            );
            b.insert(
                "maxNumberOfItemsPerPage".into(),
                json!(max_page_size.unwrap_or_else(|| num("maxNumberOfItemsPerPage", 100))),
            );
            b.insert(
                "ignoreClientFilterAndSortQueryParams".into(),
                json!(ignore_client_params.unwrap_or_else(|| {
                    view.get("ignoreClientFilterAndSortQueryParams")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })),
            );
            client.put(
                &[
                    "contenttypes",
                    &content_type,
                    "views",
                    &id,
                    "public-settings",
                ],
                Some(&Value::Object(b)),
            )
        }
        ViewsCmd::Filter {
            content_type,
            id,
            conditions,
        } => {
            let v = json_value(&conditions)?;
            if !v.is_array() {
                return Err(CliError::usage("--conditions must be a JSON array."));
            }
            client.put(
                &["contenttypes", &content_type, "views", &id, "filter"],
                Some(&json!({ "filter": v })),
            )
        }
        ViewsCmd::Sort {
            content_type,
            id,
            body,
        } => {
            let b = body.require_object("Sort settings")?;
            client.put(
                &["contenttypes", &content_type, "views", &id, "sort"],
                Some(&Value::Object(b)),
            )
        }
        ViewsCmd::SortReorder {
            content_type,
            id,
            body,
        } => {
            let b = body.require_object("Sort reorder settings")?;
            client.post(
                &[
                    "contenttypes",
                    &content_type,
                    "views",
                    &id,
                    "sort",
                    "reorder",
                ],
                Some(&Value::Object(b)),
            )
        }
        ViewsCmd::Delete {
            content_type,
            id,
            yes,
        } => {
            require_yes(yes, &format!("view '{id}'"))?;
            client.delete(&["contenttypes", &content_type, "views", &id], &[])
        }
        ViewsCmd::SetHome { content_type, id } => client.post(
            &[
                "contenttypes",
                &content_type,
                "views",
                &id,
                "set-as-home-page",
            ],
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_choices_from_labels() {
        let v = choices_value(&Some(vec!["Red Wine".into(), "White".into()]), &None)
            .unwrap()
            .unwrap();
        assert_eq!(v[0]["developerName"], "red_wine");
        assert_eq!(v[1]["label"], "White");
    }

    #[test]
    fn unknown_field_lists_known_ones() {
        let ct = json!({"contentTypeFields": [{"id": "x", "developerName": "title"}]});
        let e = field_id(&ct, "nope").unwrap_err();
        assert!(e.hint.unwrap().contains("title"));
        assert_eq!(field_id(&ct, "title").unwrap(), "x");
    }
}
