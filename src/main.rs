// A CLI that exits after one command gains nothing from boxing its error type.
#![allow(clippy::result_large_err)]

mod client;
mod commands;
mod config;
mod error;
mod guide;
mod input;
mod output;
mod sync;

use clap::{Parser, Subcommand};
use error::{CliError, Result};
use serde_json::Value;

const ABOUT: &str =
    "Manage a Raytha site (themes, templates, pages, content, media, menus) from the \
command line. Built for LLM agents: JSON on stdout, stable exit codes, actionable errors.

START HERE: run `raytha guide build-a-site` (the agent playbook), then `raytha doctor`. \
`raytha guide` lists every topic; the docs are built into this binary.";

const AFTER_HELP: &str = "\
Configuration (environment only):
  RAYTHA_URL       Site root, e.g. https://example.com
  RAYTHA_API_KEY   API key (Settings > Administrators > (admin) > API Keys)

Output: one JSON document on stdout, {\"ok\":true,\"data\":...} or {\"ok\":false,\"error\":{...}}.
Exit codes: 0 ok, 2 usage/config, 3 auth/permission, 4 not found, 5 validation, 6 server/network.

Docs: `raytha guide` lists the built-in topics; read `raytha guide build-a-site` first.";

#[derive(Parser, Debug)]
#[command(name = "raytha", version, about = ABOUT, after_help = AFTER_HELP)]
struct Cli {
    /// Site root URL (overrides RAYTHA_URL).
    #[arg(long, global = true, env = "RAYTHA_URL", value_name = "URL")]
    url: Option<String>,

    /// API key (overrides RAYTHA_API_KEY).
    #[arg(
        long,
        global = true,
        env = "RAYTHA_API_KEY",
        hide_env_values = true,
        value_name = "KEY"
    )]
    api_key: Option<String>,

    /// Indent the JSON output.
    #[arg(long, global = true)]
    pretty: bool,

    /// Request timeout in seconds.
    #[arg(long, global = true, default_value_t = 60, value_name = "SECONDS")]
    timeout: u64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Check connectivity, the API key, and which command groups it may use.
    Doctor,
    /// Agent-oriented docs: `raytha guide [topic]`. Prints markdown, not JSON.
    Guide {
        /// Topic name; omit to list topics.
        topic: Option<String>,
        /// Wrap the markdown in the standard JSON envelope.
        #[arg(long)]
        json: bool,
    },
    /// Themes: create, edit, pull/push as files, media, activate.
    #[command(subcommand)]
    Theme(commands::theme::Cmd),
    /// Web templates (liquid layouts and page templates) inside a theme.
    #[command(subcommand, name = "web-template")]
    WebTemplate(commands::web_template::Cmd),
    /// Widget templates (reusable page blocks with a settings form) inside a theme.
    #[command(subcommand, name = "widget-template")]
    WidgetTemplate(commands::widget_template::Cmd),
    /// Site pages built from widgets: create, sections, widgets, publish.
    #[command(subcommand, name = "site-page")]
    SitePage(commands::site_page::Cmd),
    /// Content types: the schema, fields, and views.
    #[command(subcommand, name = "content-type")]
    ContentType(commands::content_type::Cmd),
    /// Content items: create, edit, publish, list, trash.
    #[command(subcommand)]
    Content(commands::content::Cmd),
    /// The site media library.
    #[command(subcommand)]
    Media(commands::media::Cmd),
    /// Navigation menus and items.
    #[command(subcommand)]
    Menu(commands::menu::Cmd),
    /// Public users (site visitors).
    #[command(subcommand)]
    User(commands::user::UserCmd),
    /// Public user groups.
    #[command(subcommand, name = "user-group")]
    UserGroup(commands::user::GroupCmd),
    /// Request every public route like a visitor and report broken ones (exit 6 if any fail).
    Check(commands::check::CheckArgs),
    /// Raytha Functions (JavaScript for HTTP requests, Liquid and content events).
    #[command(subcommand)]
    Function(commands::function::Cmd),
    /// The content model as one portable document: export, import, outline.
    #[command(subcommand)]
    Schema(commands::schema::Cmd),
    /// Background jobs started by other commands (theme copy, template matching, batch import).
    #[command(subcommand)]
    Task(commands::task::Cmd),
    /// Dump the live OpenAPI spec (for operations without a curated command).
    Spec(commands::spec::SpecArgs),
}

/// Raytha ids are URL-safe base64, so roughly one in 32 starts with `-`. Let id positionals accept that
/// instead of clap treating it as a flag.
fn hyphen_tolerant_ids(cmd: clap::Command) -> clap::Command {
    let ids: Vec<String> = cmd
        .get_positionals()
        .filter(|a| a.get_id() == "id" || a.get_id().as_str().ends_with("_id"))
        .map(|a| a.get_id().to_string())
        .collect();
    let mut cmd = ids
        .into_iter()
        .fold(cmd, |c, id| c.mut_arg(id, |a| a.allow_hyphen_values(true)));
    let names: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    for n in names {
        cmd = cmd.mut_subcommand(n, hyphen_tolerant_ids);
    }
    cmd
}

fn parse_cli() -> std::result::Result<Cli, clap::Error> {
    use clap::{CommandFactory, FromArgMatches};
    let matches = hyphen_tolerant_ids(Cli::command()).try_get_matches()?;
    Cli::from_arg_matches(&matches)
}

fn main() {
    // `ring` rather than aws-lc: it cross-compiles to static musl targets without a C++/cmake toolchain.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let pretty = std::env::args().any(|a| a == "--pretty");
    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(e) => {
            use clap::error::ErrorKind;
            match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    output::emit(&e.to_string());
                    std::process::exit(0);
                }
                _ => {
                    let err = CliError::usage(e.render().to_string().trim().to_string())
                        .with_hint("Run the command with --help for its flags, or `raytha guide`.");
                    output::err(&err, pretty);
                    std::process::exit(err.exit);
                }
            }
        }
    };

    match run(cli) {
        Ok(Some(data)) => output::ok(data, pretty),
        Ok(None) => {}
        Err(e) => {
            output::err(&e, pretty);
            std::process::exit(e.exit);
        }
    }
}

fn run(cli: Cli) -> Result<Option<Value>> {
    if let Command::Guide { topic, json } = &cli.command {
        return run_guide(topic.as_deref(), *json);
    }

    let cfg = config::resolve(cli.url, cli.api_key)?;
    let client = client::Client::new(&cfg, cli.timeout)?;

    let data = match cli.command {
        Command::Doctor => commands::doctor::run(&client)?,
        Command::Spec(args) => commands::spec::run(&client, &args)?,
        Command::Theme(c) => commands::theme::run(&client, c)?,
        Command::WebTemplate(c) => commands::web_template::run(&client, c)?,
        Command::WidgetTemplate(c) => commands::widget_template::run(&client, c)?,
        Command::SitePage(c) => commands::site_page::run(&client, c)?,
        Command::ContentType(c) => commands::content_type::run(&client, c)?,
        Command::Content(c) => commands::content::run(&client, c)?,
        Command::Media(c) => commands::media::run(&client, c)?,
        Command::Menu(c) => commands::menu::run(&client, c)?,
        Command::User(c) => commands::user::run_users(&client, c)?,
        Command::UserGroup(c) => commands::user::run_groups(&client, c)?,
        Command::Check(a) => commands::check::run(&client, &a)?,
        Command::Function(c) => commands::function::run(&client, c)?,
        Command::Schema(c) => commands::schema::run(&client, c)?,
        Command::Task(c) => commands::task::run(&client, c)?,
        Command::Guide { .. } => unreachable!("handled above"),
    };
    Ok(Some(data))
}

fn run_guide(topic: Option<&str>, json: bool) -> Result<Option<Value>> {
    let (name, body) = match topic {
        None => ("index".to_string(), guide::index()),
        Some(t) => match guide::find(t) {
            Some(topic) => (topic.name.to_string(), topic.body.to_string()),
            None => {
                let names: Vec<_> = guide::TOPICS.iter().map(|t| t.name).collect();
                return Err(CliError::usage(format!("Unknown guide topic '{t}'."))
                    .with_hint(format!("Available topics: {}", names.join(", "))));
            }
        },
    };
    if json {
        Ok(Some(serde_json::json!({ "topic": name, "markdown": body })))
    } else {
        output::emit(&body);
        if !body.ends_with('\n') {
            output::emit("\n");
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    /// Every `raytha ...` line inside a ```bash fence of a guide must parse, so the docs
    /// cannot drift from the real flags.
    #[test]
    fn guide_commands_parse() {
        let mut checked = 0;
        for topic in guide::TOPICS {
            let mut in_fence = false;
            for line in topic.body.lines() {
                if let Some(rest) = line.trim_start().strip_prefix("```") {
                    // Only ```bash fences hold runnable commands.
                    in_fence = !in_fence && rest.trim() == "bash";
                    continue;
                }
                let line = line.trim();
                if !in_fence || !line.starts_with("raytha ") {
                    continue;
                }
                let words = shlex::split(line)
                    .unwrap_or_else(|| panic!("[{}] unbalanced quotes: {line}", topic.name));
                if let Err(e) = Cli::try_parse_from(&words)
                    && !matches!(
                        e.kind(),
                        clap::error::ErrorKind::DisplayHelp
                            | clap::error::ErrorKind::DisplayVersion
                    )
                {
                    panic!("[{}] does not parse: {line}\n{e}", topic.name);
                }
                checked += 1;
            }
        }
        assert!(checked > 40, "only {checked} guide commands were checked");
    }

    #[test]
    fn every_topic_has_content() {
        for topic in guide::TOPICS {
            assert!(
                topic.body.len() > 500,
                "{} looks like a placeholder",
                topic.name
            );
            assert!(!topic.body.contains("(draft)"), "{}", topic.name);
        }
    }
}
