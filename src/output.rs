//! stdout is always exactly one JSON document (except `guide` and `--help`).

use crate::error::CliError;
use serde_json::{Value, json};

fn render(v: &Value, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
    } else {
        v.to_string()
    }
}

/// Writes to stdout and ignores a closed pipe (`raytha ... | head`) instead of panicking.
pub fn emit(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

pub fn ok(data: Value, pretty: bool) {
    emit(&format!(
        "{}\n",
        render(&json!({ "ok": true, "data": data }), pretty)
    ));
}

pub fn err(e: &CliError, pretty: bool) {
    emit(&format!("{}\n", render(&e.to_json(), pretty)));
}
