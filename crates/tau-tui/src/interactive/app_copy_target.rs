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

fn copy_to_clipboard(target: &str) -> Result<(), String> {
    let mut child = clipboard_command()
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("clipboard command failed to start: {err}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "clipboard command stdin unavailable".to_string())?;
    stdin
        .write_all(target.as_bytes())
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

fn clipboard_command() -> Command {
    if let Ok(command) = std::env::var(CLIPBOARD_COMMAND_ENV) {
        let mut child = Command::new("sh");
        child.arg("-c").arg(command);
        return child;
    }

    Command::new("pbcopy")
}
