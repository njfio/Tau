use crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, FocusPanel};
use super::chat::{ChatMessage, MessageRole};

pub(crate) fn handle_command_palette_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.focus = FocusPanel::Input;
            app.command_input.clear();
        }
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            app.focus = FocusPanel::Input;
            app.command_input.clear();
            execute_command(app, &cmd);
        }
        KeyCode::Char(c) => app.command_input.push(c),
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        _ => {}
    }
}

pub(crate) fn submit_input(app: &mut App) {
    let text = app.input.get_text();
    if text.trim().is_empty() {
        return;
    }

    if text.starts_with('/') {
        execute_command(app, text.trim_start_matches('/'));
        app.input.clear();
        return;
    }

    app.push_message(MessageRole::User, text.clone());
    app.chat.add_message(ChatMessage {
        role: MessageRole::Assistant,
        content: format!(
            "Received your message. (Model: {}, {} chars)",
            app.config.model,
            text.len()
        ),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });

    app.status.total_messages += 2;
    app.status.total_tokens += text.len() as u64 / 4;
    app.chat.scroll_to_bottom();
    app.input.clear();
}

fn execute_command(app: &mut App, cmd: &str) {
    match cmd.trim() {
        "quit" | "q" => app.should_quit = true,
        "clear" => app.chat.clear(),
        "help" => app.show_help = true,
        "tools" => app.show_tool_panel = !app.show_tool_panel,
        _ => app.chat.add_message(ChatMessage {
            role: MessageRole::System,
            content: format!("Unknown command: {cmd}"),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        }),
    }
}
