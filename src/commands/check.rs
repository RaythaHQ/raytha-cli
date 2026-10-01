//! `raytha check`: request every public route the way a visitor would and report what broke.
//!
//! A Liquid mistake often shows up only at render time, as an empty 500 or an empty page, so a
//! successful push says little about whether the site works.

use super::{ListArgs, list, str_of};
use crate::client::Client;
use crate::error::{CliError, Result};
use clap::Args;
use serde_json::{Value, json};

const NOT_FOUND_PROBE: &str = "/raytha-check-no-such-page";

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Only check these content types (comma separated developer names).
    #[arg(long, value_delimiter = ',', value_name = "NAMES")]
    types: Vec<String>,
    /// Items checked per content type.
    #[arg(long, default_value_t = 25, value_name = "N")]
    max_items: u32,
    /// Skip content item pages.
    #[arg(long)]
    no_items: bool,
    /// Skip site pages.
    #[arg(long)]
    no_pages: bool,
    /// List every route in the report, not only the ones with problems.
    #[arg(long)]
    all: bool,
}

struct Route {
    kind: &'static str,
    path: String,
}

pub fn run(client: &Client, args: &CheckArgs) -> Result<Value> {
    let mut routes = vec![Route {
        kind: "home",
        path: "/".into(),
    }];

    let types = content_types(client, &args.types)?;
    for ct in &types {
        let views = list(
            client,
            &["contenttypes", ct, "views"],
            &[],
            &ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )?;
        for v in items_of(&views) {
            if v.get("isPublished").and_then(Value::as_bool) == Some(true)
                && let Some(p) = str_of(v, "routePath").filter(|p| !p.is_empty())
            {
                routes.push(Route {
                    kind: "view",
                    path: format!("/{p}"),
                });
            }
        }
        if !args.no_items {
            let items = list(
                client,
                &["contentitems", ct],
                &[],
                &ListArgs {
                    page_size: args.max_items,
                    ..ListArgs::default()
                },
            )?;
            for i in items_of(&items) {
                if i.get("isPublished").and_then(Value::as_bool) == Some(true)
                    && let Some(p) = str_of(i, "routePath").filter(|p| !p.is_empty())
                {
                    routes.push(Route {
                        kind: "item",
                        path: format!("/{p}"),
                    });
                }
            }
        }
    }
    if !args.no_pages {
        let pages = list(
            client,
            &["sitepages"],
            &[],
            &ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )?;
        for p in items_of(&pages) {
            if p.get("isPublished").and_then(Value::as_bool) == Some(true)
                && let Some(path) = str_of(p, "routePath").filter(|p| !p.is_empty())
            {
                routes.push(Route {
                    kind: "site-page",
                    path: format!("/{path}"),
                });
            }
        }
    }
    routes.dedup_by(|a, b| a.path == b.path);

    let mut report = Vec::new();
    let (mut failed, mut warned) = (0usize, 0usize);
    for r in &routes {
        let (status, body) = client.fetch_public(&r.path)?;
        let problems = problems_for(status, &body);
        let bad = problems.iter().any(|p| p.fatal);
        failed += usize::from(bad);
        warned += usize::from(!bad && !problems.is_empty());
        if bad || !problems.is_empty() || args.all {
            let mut e = entry(r.kind, &r.path, status, body.len(), &problems);
            // A development server writes the failing template's name into the 500 body.
            if bad && status >= 400 && !body.trim().is_empty() {
                e["body"] = json!(body.trim().chars().take(300).collect::<String>());
            }
            report.push(e);
        }
    }

    // A missing page should be a real 404 with a body, not a 200 or an empty reply.
    let (status, body) = client.fetch_public(NOT_FOUND_PROBE)?;
    let mut not_found = Vec::new();
    if status != 404 {
        not_found.push(Problem::warn(format!(
            "an unknown path answered {status}, not 404"
        )));
    } else if body.trim().is_empty() {
        not_found.push(Problem::warn("the 404 page is empty".into()));
    }
    warned += usize::from(!not_found.is_empty());
    if !not_found.is_empty() || args.all {
        report.push(entry(
            "not-found",
            NOT_FOUND_PROBE,
            status,
            body.len(),
            &not_found,
        ));
    }

    let summary = json!({
        "checked": routes.len() + 1,
        "failed": failed,
        "warnings": warned,
        "routes": report,
    });
    if failed > 0 {
        return Err(CliError::server(
            "check_failed",
            format!("{failed} of {} route(s) failed.", routes.len()),
        )
        .with_hint("Each failing route is in error.fields.report with its status. Fix the template, then `raytha web-template preview <theme> <name> --view/--content-item <id>` names the failing tag; run `raytha check` again.")
        .with_fields(json!({ "report": summary })));
    }
    Ok(summary)
}

struct Problem {
    fatal: bool,
    text: String,
}

impl Problem {
    fn fatal(text: String) -> Self {
        Problem { fatal: true, text }
    }
    fn warn(text: String) -> Self {
        Problem { fatal: false, text }
    }
}

fn problems_for(status: u16, body: &str) -> Vec<Problem> {
    let mut out = Vec::new();
    if status >= 400 {
        out.push(Problem::fatal(format!("HTTP {status}")));
    } else if status == 200 && body.trim().is_empty() {
        out.push(Problem::fatal(
            "HTTP 200 with an empty body (a Liquid error renders as nothing in production)".into(),
        ));
    }
    if status < 400 && (body.contains("{{") || body.contains("{%")) {
        out.push(Problem::warn(
            "output still contains `{{` or `{%` (unrendered Liquid, or code samples)".into(),
        ));
    }
    out
}

fn entry(kind: &str, path: &str, status: u16, bytes: usize, problems: &[Problem]) -> Value {
    let mut v = json!({ "kind": kind, "path": path, "status": status, "bytes": bytes });
    if !problems.is_empty() {
        v["problems"] = json!(problems.iter().map(|p| p.text.clone()).collect::<Vec<_>>());
    }
    v
}

fn items_of(v: &Value) -> impl Iterator<Item = &Value> {
    v.get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn content_types(client: &Client, only: &[String]) -> Result<Vec<String>> {
    let all = list(
        client,
        &["contenttypes"],
        &[],
        &ListArgs {
            all: true,
            ..ListArgs::default()
        },
    )?;
    let names: Vec<String> = items_of(&all)
        .filter_map(|t| str_of(t, "developerName").map(str::to_string))
        .collect();
    if only.is_empty() {
        return Ok(names);
    }
    for o in only {
        if !names.contains(o) {
            return Err(
                CliError::not_found(format!("Content type '{o}' does not exist."))
                    .with_hint("List them with `raytha content-type list`."),
            );
        }
    }
    Ok(only.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_responses() {
        assert!(problems_for(500, "").iter().any(|p| p.fatal));
        assert!(problems_for(200, "  ").iter().any(|p| p.fatal));
        assert!(problems_for(200, "<p>ok</p>").is_empty());
        let w = problems_for(200, "<p>{{ x }}</p>");
        assert!(w.len() == 1 && !w[0].fatal);
        assert!(problems_for(302, "").is_empty());
    }
}
