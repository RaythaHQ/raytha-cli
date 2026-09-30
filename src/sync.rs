//! Theme pull/push: a theme as a directory of plain files.
//!
//! ```text
//! theme.json                      { "title", "developerName", "description" }
//! web-templates/<name>.liquid     template content
//! web-templates/<name>.json       optional: label, isBaseLayout, parent, allowAccessForNewContentTypes, contentTypes
//! widget-templates/<name>.liquid  widget markup
//! widget-templates/<name>.json    optional: label, fields
//! media/<file>                    uploaded as theme media
//! ```
//!
//! Push is declarative and idempotent: local files are the source of truth for what they describe,
//! unchanged items are left alone, and deletions only happen with `--prune`.

use crate::client::Client;
use crate::commands::theme::{PullArgs, PushArgs};
use crate::commands::{
    ListArgs, content_type_index, list, str_of, web_template as wt, widget_template as wg,
    write_file,
};
use crate::error::{CliError, EXIT_NOT_FOUND, Result};
use crate::input::{humanize, strip_empty, to_developer_name};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const WEB_DIR: &str = "web-templates";
const WIDGET_DIR: &str = "widget-templates";
const MEDIA_DIR: &str = "media";

// ---------------------------------------------------------------------------------------------
// Local model
// ---------------------------------------------------------------------------------------------

struct LocalWeb {
    name: String,
    spec: wt::Spec,
    parent: Option<String>,
}

struct LocalWidget {
    name: String,
    label: Option<String>,
    content: String,
    fields: Option<Value>,
}

struct LocalMedia {
    file_name: String,
    path: PathBuf,
    size: u64,
}

struct LocalTheme {
    dev: String,
    title: String,
    description: String,
    web: Vec<LocalWeb>,
    widgets: Vec<LocalWidget>,
    media: Vec<LocalMedia>,
}

fn read_json_file(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .map_err(|e| CliError::usage(format!("Cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|e| CliError::usage(format!("{} is not valid JSON: {e}", path.display())))
}

/// `(name, content, sidecar)` for every `*.liquid` file in `dir`, sorted by name.
fn read_templates(dir: &Path) -> Result<Vec<(String, String, Value)>> {
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("liquid"))
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let content = fs::read_to_string(&path)
            .map_err(|e| CliError::usage(format!("Cannot read {}: {e}", path.display())))?;
        let sidecar_path = path.with_extension("json");
        let sidecar = if sidecar_path.exists() {
            read_json_file(&sidecar_path)?
        } else {
            json!({})
        };
        out.push((name, content, sidecar));
    }
    Ok(out)
}

fn load(dir: &Path, override_name: Option<&str>, include_media: bool) -> Result<LocalTheme> {
    let meta_path = dir.join("theme.json");
    if !meta_path.exists() {
        return Err(CliError::usage(format!(
            "No theme.json in '{}'.",
            dir.display()
        ))
        .with_hint(
            "Create it with {\"title\":\"My Theme\",\"developerName\":\"my_theme\",\"description\":\"...\"}, \
             or start from an existing theme with `raytha theme pull <name> <dir>`.",
        ));
    }
    let meta = read_json_file(&meta_path)?;
    let raw_dev = override_name
        .map(str::to_string)
        .or_else(|| str_of(&meta, "developerName").map(str::to_string))
        .ok_or_else(|| CliError::usage("theme.json needs a developerName."))?;
    let dev = to_developer_name(&raw_dev);
    if dev.is_empty() {
        return Err(CliError::usage(
            "developerName must contain letters or digits.",
        ));
    }
    let title = str_of(&meta, "title")
        .map(str::to_string)
        .unwrap_or_else(|| humanize(&dev));
    let description = str_of(&meta, "description")
        .map(str::to_string)
        .unwrap_or_else(|| title.clone());

    let mut web = Vec::new();
    for (name, content, side) in read_templates(&dir.join(WEB_DIR))? {
        let is_base = side
            .get("isBaseLayout")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| content.contains("renderbody"));
        // Only what the sidecar states is enforced; anything it leaves out keeps the server's
        // value on update (and gets a default on create).
        let parent = str_of(&side, "parent")
            .map(str::to_string)
            .filter(|p| !p.is_empty());
        let spec = wt::Spec {
            label: str_of(&side, "label").map(str::to_string),
            content: Some(content),
            base_layout: Some(is_base),
            parent: str_of(&side, "parent").map(str::to_string),
            allow_new: side
                .get("allowAccessForNewContentTypes")
                .and_then(Value::as_bool),
            content_types: side.get("contentTypes").and_then(Value::as_array).map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            }),
        };
        web.push(LocalWeb { name, spec, parent });
    }

    let mut widgets = Vec::new();
    for (name, content, side) in read_templates(&dir.join(WIDGET_DIR))? {
        widgets.push(LocalWidget {
            label: str_of(&side, "label").map(str::to_string),
            name,
            content,
            fields: side.get("fields").cloned(),
        });
    }

    let mut media = Vec::new();
    let media_dir = dir.join(MEDIA_DIR);
    if include_media && media_dir.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&media_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();
        files.sort();
        for path in files {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if file_name.starts_with('.') {
                continue;
            }
            let size = fs::metadata(&path)?.len();
            media.push(LocalMedia {
                file_name,
                path,
                size,
            });
        }
    }

    Ok(LocalTheme {
        dev,
        title,
        description,
        web,
        widgets,
        media,
    })
}

// ---------------------------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
struct Report {
    actions: Vec<Value>,
    failures: Vec<CliError>,
}

impl Report {
    fn add(&mut self, kind: &str, name: &str, action: &str, detail: Option<String>) {
        let mut m = Map::new();
        m.insert("kind".into(), json!(kind));
        m.insert("name".into(), json!(name));
        m.insert("action".into(), json!(action));
        if let Some(d) = detail {
            m.insert("detail".into(), json!(d));
        }
        self.actions.push(Value::Object(m));
    }

    fn fail(&mut self, kind: &str, name: &str, e: CliError) {
        self.add(kind, name, "failed", Some(e.message.clone()));
        self.failures.push(e);
    }

    fn summary(&self) -> Value {
        let mut counts: Map<String, Value> = Map::new();
        for a in &self.actions {
            let k = a["action"].as_str().unwrap_or("").to_string();
            let n = counts.get(&k).and_then(Value::as_u64).unwrap_or(0) + 1;
            counts.insert(k, json!(n));
        }
        Value::Object(counts)
    }
}

// ---------------------------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------------------------

fn list_items(client: &Client, segs: &[&str], extra: &[(&str, String)]) -> Result<Vec<Value>> {
    let args = ListArgs {
        all: true,
        page_size: 100,
        ..ListArgs::default()
    };
    let res = list(client, segs, extra, &args)?;
    Ok(res
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn names_of(items: &[Value]) -> HashSet<String> {
    items
        .iter()
        .filter_map(|i| str_of(i, "developerName").map(str::to_string))
        .collect()
}

/// Orders templates so a parent is always created before its children.
fn topo_order(web: &[LocalWeb], remote: &HashSet<String>) -> Result<Vec<usize>> {
    let local: HashSet<&str> = web.iter().map(|w| w.name.as_str()).collect();
    for w in web {
        if let Some(p) = &w.parent
            && !p.is_empty()
            && !local.contains(p.as_str())
            && !remote.contains(p)
        {
            return Err(CliError::validation(format!(
                "Template '{}' has parent '{p}', which exists neither locally nor on the server.",
                w.name
            ))
            .with_hint("Add web-templates/<parent>.liquid or fix `parent` in the sidecar json."));
        }
    }
    let mut placed: HashSet<&str> = HashSet::new();
    let mut order = Vec::new();
    while order.len() < web.len() {
        let before = order.len();
        for (i, w) in web.iter().enumerate() {
            if placed.contains(w.name.as_str()) {
                continue;
            }
            let ready = match &w.parent {
                None => true,
                Some(p) if p.is_empty() => true,
                Some(p) => !local.contains(p.as_str()) || placed.contains(p.as_str()),
            };
            if ready {
                placed.insert(w.name.as_str());
                order.push(i);
            }
        }
        if order.len() == before {
            return Err(CliError::validation(
                "Circular parent relationship between local web templates.",
            ));
        }
    }
    Ok(order)
}

pub fn push(client: &Client, args: &PushArgs) -> Result<Value> {
    let local = load(&args.dir, args.theme.as_deref(), !args.no_media)?;
    let dry = args.dry_run;
    let mut rep = Report::default();
    let dev = local.dev.as_str();

    // Theme itself.
    let remote_theme = match client.get(&["themes", dev], &[]) {
        Ok(t) => Some(t),
        Err(e) if e.exit == EXIT_NOT_FOUND => None,
        Err(e) => return Err(e),
    };
    let mut theme_exists = remote_theme.is_some();
    match &remote_theme {
        None => {
            // Raytha gives every new theme its built-in templates, so after creating it the
            // remote lists below are read back rather than assumed empty.
            let note = dry.then(|| {
                "new theme: Raytha adds the built-in templates itself, so pushed files with built-in names become updates"
                    .to_string()
            });
            rep.add("theme", dev, "create", note);
            if !dry {
                let body = json!({
                    "title": local.title,
                    "developerName": dev,
                    "description": local.description,
                    "insertDefaultThemeMediaItems": false,
                });
                client.post(&["themes"], Some(&body))?;
                theme_exists = true;
            }
        }
        Some(t) => {
            if str_of(t, "title") == Some(local.title.as_str())
                && str_of(t, "description") == Some(local.description.as_str())
            {
                rep.add("theme", dev, "unchanged", None);
            } else {
                rep.add("theme", dev, "update", None);
                if !dry {
                    let body = json!({ "title": local.title, "description": local.description });
                    client.put(&["themes", dev], Some(&body))?;
                }
            }
        }
    }

    // Web templates.
    let remote_web = if theme_exists {
        list_items(
            client,
            &["webtemplates"],
            &[("themeDeveloperName", dev.to_string())],
        )?
    } else {
        vec![]
    };
    let remote_web_names = names_of(&remote_web);
    for i in topo_order(&local.web, &remote_web_names)? {
        let w = &local.web[i];
        let result = push_web_template(client, dev, w, remote_web_names.contains(&w.name), dry);
        match result {
            Ok((action, detail)) => rep.add("web-template", &w.name, action, detail),
            Err(e) => rep.fail("web-template", &w.name, e),
        }
    }

    // Widget templates.
    let remote_widgets = if theme_exists {
        list_items(
            client,
            &["widgettemplates"],
            &[("themeDeveloperName", dev.to_string())],
        )?
    } else {
        vec![]
    };
    let remote_widget_names = names_of(&remote_widgets);
    for w in &local.widgets {
        let result =
            push_widget_template(client, dev, w, remote_widget_names.contains(&w.name), dry);
        match result {
            Ok((action, detail)) => rep.add("widget-template", &w.name, action, detail),
            Err(e) => rep.fail("widget-template", &w.name, e),
        }
    }

    // Media.
    let mut remote_media: Vec<Value> = Vec::new();
    if !args.no_media {
        if theme_exists {
            remote_media = client
                .get(&["themes", dev, "media"], &[])?
                .as_array()
                .cloned()
                .unwrap_or_default();
        }
        push_media(client, dev, &local, &remote_media, args, &mut rep);
    }

    // Prune.
    if args.prune {
        let local_web: HashSet<&str> = local.web.iter().map(|w| w.name.as_str()).collect();
        let doomed: Vec<String> = remote_web
            .iter()
            .filter_map(|i| str_of(i, "developerName"))
            .filter(|n| !local_web.contains(n))
            .map(str::to_string)
            .collect();
        prune_templates(
            client,
            dev,
            "web-template",
            "webtemplates",
            doomed,
            dry,
            &mut rep,
            |c, n| wt::get(c, dev, n),
        );

        let local_widgets: HashSet<&str> = local.widgets.iter().map(|w| w.name.as_str()).collect();
        let doomed: Vec<String> = remote_widgets
            .iter()
            .filter_map(|i| str_of(i, "developerName"))
            .filter(|n| !local_widgets.contains(n))
            .map(str::to_string)
            .collect();
        prune_templates(
            client,
            dev,
            "widget-template",
            "widgettemplates",
            doomed,
            dry,
            &mut rep,
            |c, n| wg::get(c, dev, n),
        );
    }

    if args.activate {
        rep.add("theme", dev, "activate", None);
        if !dry && rep.failures.is_empty() {
            client.post(&["themes", dev, "set-active"], None)?;
        }
    }

    let report = json!({
        "theme": dev,
        "dryRun": dry,
        "summary": rep.summary(),
        "actions": rep.actions,
    });
    if let Some(first) = rep.failures.first() {
        let mut err = CliError::new(
            "push_incomplete",
            format!(
                "{} item(s) failed to sync. Successful items were applied; see error.fields.report.",
                rep.failures.len()
            ),
            first.exit,
        )
        .with_hint("Fix the failed items (see each action's detail) and run the same push again; it is idempotent.")
        .with_fields(json!({ "report": report }));
        err.status = first.status;
        return Err(err);
    }
    Ok(report)
}

type Outcome = (&'static str, Option<String>);

fn push_web_template(
    client: &Client,
    theme: &str,
    w: &LocalWeb,
    exists: bool,
    dry: bool,
) -> Result<Outcome> {
    if !exists {
        if !dry {
            wt::create(client, theme, &w.name, &w.spec)?;
        }
        return Ok(("create", None));
    }
    let remote = wt::get(client, theme, &w.name)?;
    let current = wt::edit_body(client, &remote, &wt::Spec::default())?;
    let desired = wt::edit_body(client, &remote, &w.spec)?;
    if normalize_body(&current) == normalize_body(&desired) {
        return Ok(("unchanged", None));
    }
    if !dry {
        client.put(
            &["webtemplates", "theme", theme, "template", &w.name],
            Some(&desired),
        )?;
    }
    Ok(("update", changed_keys(&current, &desired)))
}

fn push_widget_template(
    client: &Client,
    theme: &str,
    w: &LocalWidget,
    exists: bool,
    dry: bool,
) -> Result<Outcome> {
    if !exists {
        if !dry {
            wg::create(
                client,
                theme,
                &w.name,
                w.label.clone(),
                w.content.clone(),
                w.fields.clone(),
            )?;
        }
        return Ok(("create", None));
    }
    let remote = wg::get(client, theme, &w.name)?;
    let remote_fields = strip_empty(remote.get("fields").unwrap_or(&json!([])));
    let same_fields = match &w.fields {
        Some(f) => strip_empty(f) == remote_fields,
        None => true,
    };
    let same_label = match &w.label {
        Some(l) => str_of(&remote, "label") == Some(l.as_str()),
        None => true,
    };
    if same_label && str_of(&remote, "content") == Some(w.content.as_str()) && same_fields {
        return Ok(("unchanged", None));
    }
    if !dry {
        let body = wg::edit_body(
            &remote,
            w.label.clone(),
            Some(w.content.clone()),
            w.fields.clone(),
        );
        client.put(
            &["widgettemplates", "theme", theme, "template", &w.name],
            Some(&body),
        )?;
    }
    let mut changed = Vec::new();
    if !same_label {
        changed.push("label");
    }
    if str_of(&remote, "content") != Some(w.content.as_str()) {
        changed.push("content");
    }
    if !same_fields {
        changed.push("fields");
    }
    Ok(("update", Some(format!("changed: {}", changed.join(", ")))))
}

fn normalize_body(v: &Value) -> Value {
    let mut v = v.clone();
    if let Some(types) = v
        .get_mut("templateAccessToModelDefinitions")
        .and_then(Value::as_array_mut)
    {
        types.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
    v
}

fn changed_keys(current: &Value, desired: &Value) -> Option<String> {
    let (Some(c), Some(d)) = (current.as_object(), desired.as_object()) else {
        return None;
    };
    let keys: Vec<&str> = d
        .iter()
        .filter(|(k, v)| {
            normalize_body(&json!({ k.as_str(): v }))
                != normalize_body(
                    &json!({ k.as_str(): c.get(k.as_str()).cloned().unwrap_or(Value::Null) }),
                )
        })
        .map(|(k, _)| k.as_str())
        .collect();
    if keys.is_empty() {
        None
    } else {
        Some(format!("changed: {}", keys.join(", ")))
    }
}

fn push_media(
    client: &Client,
    theme: &str,
    local: &LocalTheme,
    remote: &[Value],
    args: &PushArgs,
    rep: &mut Report,
) {
    let dry = args.dry_run;
    let by_name: HashMap<&str, &Value> = remote
        .iter()
        .filter_map(|m| str_of(m, "fileName").map(|n| (n, m)))
        .collect();

    for m in &local.media {
        let outcome: Result<Outcome> = (|| match by_name.get(m.file_name.as_str()) {
            None => {
                if !dry {
                    client.upload(&["themes", theme, "media"], &m.path)?;
                }
                Ok(("create", None))
            }
            Some(r) => {
                let remote_len = r.get("length").and_then(Value::as_u64).unwrap_or(0);
                if remote_len == m.size {
                    return Ok(("unchanged", None));
                }
                if !args.replace_media {
                    return Ok((
                        "skip",
                        Some(format!(
                            "size differs (local {} bytes, remote {remote_len}); use --replace-media to overwrite",
                            m.size
                        )),
                    ));
                }
                if !dry {
                    if let Some(id) = str_of(r, "id") {
                        client.delete(&["themes", theme, "media", id], &[])?;
                    }
                    client.upload(&["themes", theme, "media"], &m.path)?;
                }
                Ok(("update", Some("replaced".into())))
            }
        })();
        match outcome {
            Ok((a, d)) => rep.add("media", &m.file_name, a, d),
            Err(e) => rep.fail("media", &m.file_name, e),
        }
    }

    if args.prune {
        let local_names: HashSet<&str> = local.media.iter().map(|m| m.file_name.as_str()).collect();
        for r in remote {
            let Some(name) = str_of(r, "fileName") else {
                continue;
            };
            if local_names.contains(name) {
                continue;
            }
            let res: Result<()> = (|| {
                if !dry && let Some(id) = str_of(r, "id") {
                    client.delete(&["themes", theme, "media", id], &[])?;
                }
                Ok(())
            })();
            match res {
                Ok(()) => rep.add("media", name, "delete", None),
                Err(e) => rep.fail("media", name, e),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn prune_templates(
    client: &Client,
    theme: &str,
    kind: &str,
    route: &str,
    names: Vec<String>,
    dry: bool,
    rep: &mut Report,
    fetch: impl Fn(&Client, &str) -> Result<Value>,
) {
    // Children must go before their parents, which is unknowable up front for web templates, so
    // retry failures for a few passes.
    let mut pending: Vec<String> = Vec::new();
    for n in names {
        match fetch(client, &n) {
            Ok(t) if t.get("isBuiltInTemplate").and_then(Value::as_bool) == Some(true) => {
                rep.add(
                    kind,
                    &n,
                    "skip",
                    Some("built-in templates are never pruned".into()),
                );
            }
            Ok(_) => pending.push(n),
            Err(e) => rep.fail(kind, &n, e),
        }
    }
    if dry {
        for n in pending {
            rep.add(kind, &n, "delete", None);
        }
        return;
    }
    let mut last_errors: HashMap<String, CliError> = HashMap::new();
    for _ in 0..4 {
        let mut still = Vec::new();
        for n in pending {
            match client.delete(&[route, "theme", theme, "template", &n], &[]) {
                Ok(_) => {
                    last_errors.remove(&n);
                    rep.add(kind, &n, "delete", None);
                }
                Err(e) => {
                    last_errors.insert(n.clone(), e);
                    still.push(n);
                }
            }
        }
        pending = still;
        if pending.is_empty() {
            break;
        }
    }
    for n in pending {
        if let Some(e) = last_errors.remove(&n) {
            rep.fail(kind, &n, e);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Pull
// ---------------------------------------------------------------------------------------------

fn pretty(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string());
    s.push('\n');
    s
}

pub fn pull(client: &Client, args: &PullArgs) -> Result<Value> {
    let dev = args.developer_name.as_str();
    let dir = &args.dir;
    let theme = client.get(&["themes", dev], &[])?;
    let mut files: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let record = |files: &mut Vec<String>, path: PathBuf, content: &str| -> Result<()> {
        write_file(&path, content)?;
        files.push(
            path.strip_prefix(dir)
                .unwrap_or(&path)
                .display()
                .to_string(),
        );
        Ok(())
    };

    record(
        &mut files,
        dir.join("theme.json"),
        &pretty(&json!({
            "title": str_of(&theme, "title").unwrap_or_default(),
            "developerName": dev,
            "description": str_of(&theme, "description").unwrap_or_default(),
        })),
    )?;

    // Web templates.
    let items = list_items(
        client,
        &["webtemplates"],
        &[("themeDeveloperName", dev.to_string())],
    )?;
    let mut fulls = Vec::new();
    for item in &items {
        if let Some(name) = str_of(item, "developerName") {
            fulls.push((name.to_string(), wt::get(client, dev, name)?));
        }
    }
    let ids: Vec<String> = fulls
        .iter()
        .flat_map(|(_, t)| {
            t.get("templateAccessToModelDefinitions")
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let type_names: Option<HashMap<String, String>> = if ids.is_empty() {
        Some(HashMap::new())
    } else {
        match content_type_index(client) {
            Ok(map) => Some(map),
            Err(e) => {
                warnings.push(format!(
                    "Could not resolve content type access lists ({}); contentTypes omitted from sidecars, so push will preserve the server's lists.",
                    e.message
                ));
                None
            }
        }
    };
    for (name, t) in &fulls {
        record(
            &mut files,
            dir.join(WEB_DIR).join(format!("{name}.liquid")),
            str_of(t, "content").unwrap_or_default(),
        )?;
        let mut side = Map::new();
        side.insert(
            "label".into(),
            json!(str_of(t, "label").unwrap_or_default()),
        );
        side.insert(
            "isBaseLayout".into(),
            json!(
                t.get("isBaseLayout")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            ),
        );
        if let Some(p) = t
            .get("parentTemplate")
            .and_then(|p| str_of(p, "developerName"))
        {
            side.insert("parent".into(), json!(p));
        }
        side.insert(
            "allowAccessForNewContentTypes".into(),
            json!(
                t.get("allowAccessForNewContentTypes")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            ),
        );
        if let Some(map) = &type_names {
            let mut names: Vec<&String> = t
                .get("templateAccessToModelDefinitions")
                .and_then(Value::as_object)
                .map(|m| m.keys().filter_map(|id| map.get(id)).collect())
                .unwrap_or_default();
            names.sort();
            side.insert("contentTypes".into(), json!(names));
        }
        record(
            &mut files,
            dir.join(WEB_DIR).join(format!("{name}.json")),
            &pretty(&Value::Object(side)),
        )?;
    }

    // Widget templates.
    let items = list_items(
        client,
        &["widgettemplates"],
        &[("themeDeveloperName", dev.to_string())],
    )?;
    for item in &items {
        let Some(name) = str_of(item, "developerName") else {
            continue;
        };
        let t = wg::get(client, dev, name)?;
        record(
            &mut files,
            dir.join(WIDGET_DIR).join(format!("{name}.liquid")),
            str_of(&t, "content").unwrap_or_default(),
        )?;
        record(
            &mut files,
            dir.join(WIDGET_DIR).join(format!("{name}.json")),
            &pretty(&json!({
                "label": str_of(&t, "label").unwrap_or_default(),
                "fields": strip_empty(t.get("fields").unwrap_or(&json!([]))),
            })),
        )?;
    }

    // Media.
    if !args.no_media {
        let media = client
            .get(&["themes", dev, "media"], &[])?
            .as_array()
            .cloned()
            .unwrap_or_default();
        for m in media {
            let (Some(file_name), Some(url)) = (str_of(&m, "fileName"), str_of(&m, "url")) else {
                continue;
            };
            let safe = Path::new(file_name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_name);
            match client.download(url) {
                Ok(bytes) => {
                    let path = dir.join(MEDIA_DIR).join(safe);
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&path, bytes)?;
                    files.push(format!("{MEDIA_DIR}/{safe}"));
                }
                Err(e) => warnings.push(format!("Skipped media '{file_name}': {}", e.message)),
            }
        }
    }

    Ok(json!({
        "theme": dev,
        "dir": dir.display().to_string(),
        "fileCount": files.len(),
        "files": files,
        "warnings": warnings,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lw(name: &str, parent: Option<&str>) -> LocalWeb {
        LocalWeb {
            name: name.into(),
            spec: wt::Spec::default(),
            parent: parent.map(str::to_string),
        }
    }

    #[test]
    fn parents_come_first() {
        let web = vec![lw("page", Some("base")), lw("base", None)];
        let order = topo_order(&web, &HashSet::new()).unwrap();
        assert_eq!(order, vec![1, 0]);
    }

    #[test]
    fn missing_parent_is_reported() {
        let web = vec![lw("page", Some("ghost"))];
        assert!(topo_order(&web, &HashSet::new()).is_err());
        let remote: HashSet<String> = ["ghost".to_string()].into();
        assert!(topo_order(&web, &remote).is_ok());
    }

    #[test]
    fn cycles_are_rejected() {
        let web = vec![lw("a", Some("b")), lw("b", Some("a"))];
        assert!(topo_order(&web, &HashSet::new()).is_err());
    }
}
