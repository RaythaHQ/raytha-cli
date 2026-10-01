//! Error type shared by every command.
//!
//! Every failure is rendered as a JSON envelope on stdout and mapped to a stable exit code, so an
//! agent can branch on `error.code` or on the exit status without parsing prose.

use serde_json::{Map, Value, json};

/// Bad flags, missing configuration, unreadable input files.
pub const EXIT_USAGE: i32 = 2;
/// 401/403: the key is missing, invalid, or lacks a permission.
pub const EXIT_AUTH: i32 = 3;
/// 404: the named resource does not exist.
pub const EXIT_NOT_FOUND: i32 = 4;
/// 400/422: Raytha rejected the request body or identifier.
pub const EXIT_VALIDATION: i32 = 5;
/// 5xx, network failures, timeouts, non-Raytha responses.
pub const EXIT_SERVER: i32 = 6;

#[derive(Debug, Clone)]
pub struct CliError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
    pub fields: Option<Value>,
    /// Line and column in the submitted Liquid, when Raytha's parser reports one.
    pub location: Option<(u64, u64)>,
    pub status: Option<u16>,
    pub exit: i32,
}

pub type Result<T> = std::result::Result<T, CliError>;

impl CliError {
    pub fn new(code: &str, message: impl Into<String>, exit: i32) -> Self {
        CliError {
            code: code.to_string(),
            message: message.into(),
            hint: None,
            fields: None,
            location: None,
            status: None,
            exit,
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", message, EXIT_USAGE)
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new("config_error", message, EXIT_USAGE)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("validation_failed", message, EXIT_VALIDATION)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message, EXIT_NOT_FOUND)
    }

    pub fn server(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, message, EXIT_SERVER)
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_fields(mut self, fields: Value) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_location(mut self, line: u64, column: u64) -> Self {
        self.location = Some((line, column));
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    pub fn to_json(&self) -> Value {
        let mut error = Map::new();
        error.insert("code".into(), json!(self.code));
        error.insert("message".into(), json!(self.message));
        if let Some(hint) = &self.hint {
            error.insert("hint".into(), json!(hint));
        }
        if let Some(fields) = &self.fields {
            error.insert("fields".into(), fields.clone());
        }
        if let Some((line, column)) = self.location {
            error.insert("line".into(), json!(line));
            error.insert("column".into(), json!(column));
        }
        if let Some(status) = self.status {
            error.insert("status".into(), json!(status));
        }
        json!({ "ok": false, "error": Value::Object(error) })
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::new("io_error", e.to_string(), EXIT_USAGE)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::new("invalid_json", format!("Invalid JSON: {e}"), EXIT_USAGE)
            .with_hint("Check the JSON syntax. Prefer --file or @path over shell-quoted JSON.")
    }
}
