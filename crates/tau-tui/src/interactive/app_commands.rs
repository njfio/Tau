use crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, FocusPanel};
use super::app_copy_target::copy_latest_mutating_target;
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

    app.submit_prompt(text);
    app.input.clear();
}

fn execute_command(app: &mut App, cmd: &str) {
    let trimmed = cmd.trim();
    let mut parts = trimmed.split_whitespace();
    let Some(command) = parts.next() else {
        return;
    };

    match command {
        "quit" | "q" => app.should_quit = true,
        "clear" => app.chat.clear(),
        "help" => app.show_help = true,
        "tools" => app.show_tool_panel = !app.show_tool_panel,
        "copy-target" => copy_latest_mutating_target(app),
        "missions" => app.list_missions(),
        "mission" => {
            if let Some(mission_id) = parts.next() {
                app.show_mission(mission_id);
            } else {
                app.chat.add_message(ChatMessage {
                    role: MessageRole::System,
                    content: "Usage: /mission <mission-id>".to_string(),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        "resume" => {
            if let Some(mission_id) = parts.next() {
                app.resume_mission(mission_id);
            } else {
                app.chat.add_message(ChatMessage {
                    role: MessageRole::System,
                    content: "Usage: /resume <mission-id>".to_string(),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                });
            }
        }
        _ => app.chat.add_message(ChatMessage {
            role: MessageRole::System,
            content: format!("Unknown command: {trimmed}"),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        }),
    }
}
