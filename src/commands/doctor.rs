//! `raytha doctor`: one call that tells an agent whether it can work and what it is allowed to do.

use crate::client::Client;
use crate::error::Result;
use serde_json::{Value, json};

struct Probe {
    area: &'static str,
    path: &'static [&'static str],
    needs: &'static str,
}

const PROBES: &[Probe] = &[
    Probe {
        area: "templates",
        path: &["themes"],
        needs: "ManageTemplates (themes, web templates, widget templates)",
    },
    Probe {
        area: "sitePages",
        path: &["sitepages"],
        needs: "ManageSitePages",
    },
    Probe {
        area: "contentTypes",
        path: &["contenttypes"],
        needs: "ManageContentTypes (also menus)",
    },
    Probe {
        area: "media",
        path: &["mediaitems"],
        needs: "ManageMediaItems (listing) and UploadMediaItems (uploading)",
    },
    Probe {
        area: "users",
        path: &["users"],
        needs: "ManageUsers (users and user groups)",
    },
];

pub fn run(client: &Client) -> Result<Value> {
    // A failing ping (bad URL, bad key) is the error; everything after it is diagnostic.
    let ping = client.get(&["ping"], &[]).map_err(|e| {
        if e.exit == crate::error::EXIT_NOT_FOUND {
            let hint = format!(
                "No Raytha API answered at {}/raytha/api/v1/ping. RAYTHA_URL must be the site root \
                 (for example https://example.com), and the site must run a Raytha version with \
                 the v1 REST API.",
                client.site
            );
            e.with_hint(hint)
        } else {
            e
        }
    })?;

    let mut permissions = serde_json::Map::new();
    let mut missing = Vec::new();
    for probe in PROBES {
        let q = [
            ("pageSize", "1".to_string()),
            ("pageNumber", "1".to_string()),
        ];
        let allowed = match client.get(probe.path, &q) {
            Ok(_) => json!(true),
            Err(e) if e.exit == crate::error::EXIT_AUTH => {
                missing.push(format!("{}: needs {}", probe.area, probe.needs));
                json!(false)
            }
            Err(e) => json!({ "error": e.code, "message": e.message }),
        };
        permissions.insert(probe.area.to_string(), allowed);
    }

    let mut out = json!({
        "url": client.site,
        "reachable": true,
        "authenticated": true,
        "version": ping.get("version").cloned().unwrap_or(Value::Null),
        "organizationName": ping.get("organizationName").cloned().unwrap_or(Value::Null),
        "cliVersion": env!("CARGO_PKG_VERSION"),
        "permissions": Value::Object(permissions),
    });
    if !missing.is_empty() {
        out["missingPermissions"] = json!(missing);
        out["hint"] = json!(
            "Some command groups will return 'forbidden'. Content item access is per content \
             type and is not probed here."
        );
    }
    Ok(out)
}
