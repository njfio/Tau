use std::io::Write;
use std::process::{Command, Stdio};

use super::app::App;
use super::chat::MessageRole;

const CLIPBOARD_COMMAND_ENV: &str = "TAU_TUI_CLIPBOARD_COMMAND";

pub(crate) fn copy_latest_mutating_target(app: &mut App) {
    let Some(target) = app.tools.latest_successful_mutating_target() else {
        app.push_message(
            MessageRole::System,
            "No successful mutating target available".to_string(),
        );
        return;
    };

    if let Err(err) = copy_to_clipboard(target) {
        app.push_message(
            MessageRole::System,
            format!("Failed to copy latest mutating target: {err}"),
        );
        return;
    }

    app.push_message(
        MessageRole::System,
        format!("Copied latest mutating target: {target}"),
    );
}

pub(crate) fn copy_last_assistant(app: &mut App) {
    let Some(content) = app.chat.last_assistant_content() else {
        app.push_message(
            MessageRole::System,
            "No assistant message to copy".to_string(),
        );
        return;
    };
    let text = content.to_string();
    if let Err(err) = copy_to_clipboard(&text) {
        app.push_message(MessageRole::System, format!("Failed to copy: {err}"));
        return;
    }
    app.push_message(
        MessageRole::System,
        "Copied last assistant message to clipboard".to_string(),
    );
}

pub(crate) fn copy_transcript(app: &mut App) {
    let text = app.chat.transcript_text();
    if text.is_empty() {
        app.push_message(MessageRole::System, "No messages to copy".to_string());
        return;
    }
    if let Err(err) = copy_to_clipboard(&text) {
        app.push_message(
            MessageRole::System,
            format!("Failed to copy transcript: {err}"),
        );
        return;
    }
    app.push_message(
        MessageRole::System,
        "Copied full transcript to clipboard".to_string(),
    );
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // When an explicit clipboard command is configured, use it directly (skip OSC 52)
    if std::env::var(CLIPBOARD_COMMAND_ENV).is_err() && osc52_copy(text).is_ok() {
        return Ok(());
    }

    let mut child = clipboard_command()
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("clipboard command failed to start: {err}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "clipboard command stdin unavailable".to_string())?;
    stdin
        .write_all(text.as_bytes())
        .map_err(|err| format!("clipboard write failed: {err}"))?;
    drop(stdin);
    let status = child
        .wait()
        .map_err(|err| format!("clipboard command failed: {err}"))?;
    if status.success() {
        return Ok(());
    }

    Err(format!("clipboard command exited with status {status}"))
}

fn osc52_copy(text: &str) -> Result<(), std::io::Error> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    let sequence = format!("\x1b]52;c;{encoded}\x07");
    std::io::stdout().write_all(sequence.as_bytes())?;
    std::io::stdout().flush()
}

fn clipboard_command() -> Command {
    if let Ok(command) = std::env::var(CLIPBOARD_COMMAND_ENV) {
        let mut child = Command::new("sh");
        child.arg("-c").arg(command);
        return child;
    }

    Command::new("pbcopy")
}
