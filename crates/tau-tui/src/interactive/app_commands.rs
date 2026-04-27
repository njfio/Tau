use crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, FocusPanel};
use super::app_copy_target::{copy_last_assistant, copy_latest_mutating_target, copy_transcript};
use super::chat::{ChatMessage, MessageRole};

pub(crate) fn handle_command_palette_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.focus = FocusPanel::Input;
            clear_command_palette(app);
        }
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            app.focus = FocusPanel::Input;
            clear_command_palette(app);
            execute_command(app, &cmd);
        }
        KeyCode::Char(character) => insert_command_char(app, character),
        KeyCode::Backspace => delete_command_backward(app),
        KeyCode::Delete => delete_command_forward(app),
        KeyCode::Left => move_command_cursor_left(app),
        KeyCode::Right => move_command_cursor_right(app),
        KeyCode::Home => app.command_cursor_col = 0,
        KeyCode::End => app.command_cursor_col = command_char_count(app),
        KeyCode::Tab => jump_to_next_placeholder(app),
        _ => {}
    }
}

fn clear_command_palette(app: &mut App) {
    app.command_input.clear();
    app.command_cursor_col = 0;
}

fn insert_command_char(app: &mut App, character: char) {
    let byte_idx = char_to_byte(&app.command_input, app.command_cursor_col);
    app.command_input.insert(byte_idx, character);
    app.command_cursor_col += 1;
}

fn delete_command_backward(app: &mut App) {
    if app.command_cursor_col == 0 {
        return;
    }

    let byte_idx = char_to_byte(&app.command_input, app.command_cursor_col);
    let prev_byte_idx = char_to_byte(&app.command_input, app.command_cursor_col - 1);
    app.command_input.drain(prev_byte_idx..byte_idx);
    app.command_cursor_col -= 1;
}

fn delete_command_forward(app: &mut App) {
    let input_chars = command_char_count(app);
    if app.command_cursor_col >= input_chars {
        return;
    }

    let byte_idx = char_to_byte(&app.command_input, app.command_cursor_col);
    let next_byte_idx = char_to_byte(&app.command_input, app.command_cursor_col + 1);
    app.command_input.drain(byte_idx..next_byte_idx);
}

fn move_command_cursor_left(app: &mut App) {
    app.command_cursor_col = app.command_cursor_col.saturating_sub(1);
}

fn move_command_cursor_right(app: &mut App) {
    app.command_cursor_col = (app.command_cursor_col + 1).min(command_char_count(app));
}

fn jump_to_next_placeholder(app: &mut App) {
    let Some(next_cursor) = next_placeholder_start(&app.command_input, app.command_cursor_col)
    else {
        return;
    };
    app.command_cursor_col = next_cursor;
}

fn next_placeholder_start(input: &str, cursor_col: usize) -> Option<usize> {
    let spans = placeholder_spans(input);
    spans
        .iter()
        .find(|(start, _)| *start > cursor_col)
        .or_else(|| spans.first())
        .map(|(start, _)| *start)
}

fn placeholder_spans(input: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut open: Option<(char, usize)> = None;

    for (char_idx, character) in input.chars().enumerate() {
        match character {
            '<' | '{' => open = Some((character, char_idx)),
            '>' => {
                if let Some(('<', start)) = open.take() {
                    push_placeholder_span(&mut spans, start, char_idx);
                }
            }
            '}' => {
                if let Some(('{', start)) = open.take() {
                    push_placeholder_span(&mut spans, start, char_idx);
                }
            }
            _ => {}
        }
    }

    spans
}

fn push_placeholder_span(spans: &mut Vec<(usize, usize)>, open_col: usize, close_col: usize) {
    let start = open_col + 1;
    if start < close_col {
        spans.push((start, close_col));
    }
}

fn command_char_count(app: &App) -> usize {
    app.command_input.chars().count()
}

fn char_to_byte(input: &str, char_idx: usize) -> usize {
    input
        .char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or(input.len())
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
        "copy-last" => copy_last_assistant(app),
        "copy" => copy_transcript(app),
        "toggle-mouse" => app.toggle_mouse_capture(),
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
