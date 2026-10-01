//! Background tasks. Raytha runs theme copies, template matching and batch imports as jobs; the
//! API answers with a task id, and `GET /BackgroundTasks/{id}` reports progress.

use super::str_of;
use crate::client::Client;
use crate::error::{CliError, Result};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Show a background task's status (`enqueued`, `processing`, `complete` or `error`).
    Get { id: String },
    /// Block until a background task finishes. Fails if the task errors or the wait runs out.
    Wait {
        id: String,
        /// Give up after this many seconds (the task keeps running).
        #[arg(long, default_value_t = 300, value_name = "SECONDS")]
        wait_timeout: u64,
    },
}

/// `--wait` for commands that start a background task.
#[derive(Args, Debug, Clone)]
pub struct WaitArgs {
    /// Wait for the background task to finish and return its final status.
    #[arg(long)]
    pub wait: bool,
    /// With --wait: give up after this many seconds (the task keeps running).
    #[arg(long, default_value_t = 300, value_name = "SECONDS")]
    pub wait_timeout: u64,
}

pub fn run(client: &Client, cmd: Cmd) -> Result<Value> {
    match cmd {
        Cmd::Get { id } => get(client, &id).map(|t| slim(&t)),
        Cmd::Wait { id, wait_timeout } => wait(client, &id, wait_timeout),
    }
}

pub fn get(client: &Client, id: &str) -> Result<Value> {
    client.get(&["backgroundtasks", id], &[])
}

/// What an agent needs from a task: state, progress and the message.
pub fn slim(task: &Value) -> Value {
    json!({
        "id": task.get("id"),
        "status": status_of(task),
        "percentComplete": task.get("percentComplete"),
        "statusInfo": task.get("statusInfo"),
        "errorMessage": task.get("errorMessage"),
        "completionTime": task.get("completionTime"),
    })
}

fn status_of(task: &Value) -> String {
    task.get("status")
        .and_then(|s| str_of(s, "developerName"))
        .unwrap_or("unknown")
        .to_string()
}

/// Polls until the task is `complete`; returns the raw task. `error` and timeouts are errors.
pub fn wait_raw(client: &Client, id: &str, timeout_secs: u64) -> Result<Value> {
    let started = Instant::now();
    let mut delay = Duration::from_millis(300);
    loop {
        let task = get(client, id)?;
        match status_of(&task).as_str() {
            "complete" => return Ok(task),
            "error" => {
                let msg = str_of(&task, "errorMessage")
                    .filter(|m| !m.is_empty())
                    .or_else(|| str_of(&task, "statusInfo"))
                    .unwrap_or("The background task failed.");
                return Err(CliError::server("task_failed", msg.to_string())
                    .with_hint("Fix the cause and run the command again; the server log has the full exception."));
            }
            _ => {}
        }
        if started.elapsed() >= Duration::from_secs(timeout_secs) {
            return Err(CliError::server(
                "task_timeout",
                format!(
                    "Background task {id} is still {} after {timeout_secs}s.",
                    status_of(&task)
                ),
            )
            .with_hint(format!(
                "It keeps running. Check it with `raytha task get {id}` or wait longer with `raytha task wait {id}`."
            )));
        }
        sleep(delay);
        delay = (delay * 2).min(Duration::from_secs(2));
    }
}

pub fn wait(client: &Client, id: &str, timeout_secs: u64) -> Result<Value> {
    wait_raw(client, id, timeout_secs).map(|t| slim(&t))
}

/// Result of a command that starts a task: the bare id, or the final status with `--wait`.
pub fn finish(client: &Client, started: Value, args: &WaitArgs) -> Result<Value> {
    let Some(id) = str_of(&started, "id").map(str::to_string) else {
        return Ok(started);
    };
    if args.wait {
        let done = wait(client, &id, args.wait_timeout)?;
        Ok(done)
    } else {
        Ok(json!({
            "id": id,
            "async": true,
            "hint": format!("Running in the background. `raytha task wait {id}` blocks until it finishes."),
        }))
    }
}
