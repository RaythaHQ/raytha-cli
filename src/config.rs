//! Configuration: the Raytha site URL and an API key. Nothing else, nothing on disk.

use crate::error::{CliError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    /// Site root without trailing slash and without `/raytha/...`.
    pub url: String,
    pub api_key: String,
}

pub const ENV_URL: &str = "RAYTHA_URL";
pub const ENV_KEY: &str = "RAYTHA_API_KEY";

pub fn resolve(url: Option<String>, api_key: Option<String>) -> Result<Config> {
    let url = url.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let api_key = api_key
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut missing = Vec::new();
    if url.is_none() {
        missing.push(ENV_URL);
    }
    if api_key.is_none() {
        missing.push(ENV_KEY);
    }
    if !missing.is_empty() {
        return Err(CliError::config(format!(
            "Missing configuration: {}.",
            missing.join(", ")
        ))
        .with_hint(format!(
            "Set {ENV_URL} to the site root (for example https://example.com) and {ENV_KEY} to an \
             API key. Create a key in the admin under Settings > Administrators > (admin) > API Keys."
        )));
    }

    Ok(Config {
        url: normalize_url(&url.unwrap())?,
        api_key: api_key.unwrap(),
    })
}

/// Accepts `example.com`, `https://example.com/`, or a pasted API URL and returns the site root.
pub fn normalize_url(raw: &str) -> Result<String> {
    let mut s = raw.trim().to_string();
    if !s.contains("://") {
        s = format!("https://{s}");
    }
    let parsed = reqwest::Url::parse(&s).map_err(|e| {
        CliError::config(format!("{ENV_URL} is not a valid URL: {e}"))
            .with_hint(format!("Example: {ENV_URL}=https://example.com"))
    })?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(CliError::config(format!(
            "{ENV_URL} must start with http:// or https://, got '{}'.",
            parsed.scheme()
        )));
    }
    let mut out = s.trim_end_matches('/').to_string();
    for suffix in ["/raytha/api/v1", "/raytha/api", "/raytha"] {
        if let Some(stripped) = out.strip_suffix(suffix) {
            out = stripped.to_string();
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_urls() {
        assert_eq!(normalize_url("example.com").unwrap(), "https://example.com");
        assert_eq!(
            normalize_url("https://example.com/").unwrap(),
            "https://example.com"
        );
        assert_eq!(
            normalize_url("http://localhost:5000/raytha/api/v1/").unwrap(),
            "http://localhost:5000"
        );
        assert_eq!(
            normalize_url("https://x.io/sub/raytha").unwrap(),
            "https://x.io/sub"
        );
    }

    #[test]
    fn reports_every_missing_value() {
        let e = resolve(None, None).unwrap_err();
        assert!(e.message.contains("RAYTHA_URL") && e.message.contains("RAYTHA_API_KEY"));
        let e = resolve(Some("https://a.io".into()), Some("  ".into())).unwrap_err();
        assert!(!e.message.contains("RAYTHA_URL"));
        assert!(e.message.contains("RAYTHA_API_KEY"));
    }
}
