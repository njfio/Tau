use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, FocusPanel, InputMode};
use super::app_commands::{handle_command_palette_key, submit_input};

pub(crate) fn handle_key(app: &mut App, key: KeyEvent) {
    if handle_global_shortcut(app, key) {
        return;
    }

    if app.focus == FocusPanel::CommandPalette {
        handle_command_palette_key(app, key);
        return;
    }

    if app.show_help {
        app.show_help = false;
        return;
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Insert => handle_insert_mode(app, key),
    }
}

fn handle_global_shortcut(app: &mut App, key: KeyEvent) -> bool {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => app.should_quit = true,
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => app.chat.clear(),
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => app.show_tool_panel = !app.show_tool_panel,
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => toggle_command_palette(app),
        _ => return false,
    }
    true
}

fn toggle_command_palette(app: &mut App) {
    if app.focus == FocusPanel::CommandPalette {
        app.focus = FocusPanel::Input;
        return;
    }

    app.focus = FocusPanel::CommandPalette;
    app.command_input.clear();
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('i') => set_insert_focus(app),
        KeyCode::Char('a') => {
            set_insert_focus(app);
            app.input.move_end();
        }
        KeyCode::Char('o') => {
            set_insert_focus(app);
            app.input.new_line();
        }
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.show_help = true,
        KeyCode::Char('j') | KeyCode::Down if app.focus == FocusPanel::Chat => {
            app.chat.scroll_down(1)
        }
        KeyCode::Char('k') | KeyCode::Up if app.focus == FocusPanel::Chat => app.chat.scroll_up(1),
        KeyCode::Char('G') if app.focus == FocusPanel::Chat => app.chat.scroll_to_bottom(),
        KeyCode::Char('g') if app.focus == FocusPanel::Chat => app.chat.scroll_to_top(),
        KeyCode::Char('d')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.focus == FocusPanel::Chat =>
        {
            app.chat.scroll_down(10);
        }
        KeyCode::Char('u')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.focus == FocusPanel::Chat =>
        {
            app.chat.scroll_up(10);
        }
        KeyCode::Char('y') if app.focus == FocusPanel::Chat => copy_last_assistant_message(app),
        KeyCode::Char('Y') if app.focus == FocusPanel::Chat => copy_all_messages(app),
        KeyCode::Tab => app.focus = next_focus_from_normal(app),
        KeyCode::Char('1') => app.focus = FocusPanel::Chat,
        KeyCode::Char('2') => app.focus = FocusPanel::Input,
        KeyCode::Char('3') if app.show_tool_panel => app.focus = FocusPanel::Tools,
        _ => {}
    }
}

fn handle_insert_mode(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => app.input_mode = InputMode::Normal,
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Enter) => submit_or_new_line(app, key),
        (KeyModifiers::ALT, KeyCode::Enter) => app.input.new_line(),
        (_, KeyCode::Char(c)) => app.input.insert_char(c),
        (_, KeyCode::Backspace) => app.input.delete_backward(),
        (_, KeyCode::Delete) => app.input.delete_forward(),
        (_, KeyCode::Left) => app.input.move_left(),
        (_, KeyCode::Right) => app.input.move_right(),
        (_, KeyCode::Up) => app.input.move_up(),
        (_, KeyCode::Down) => app.input.move_down(),
        (_, KeyCode::Home) => app.input.move_home(),
        (_, KeyCode::End) => app.input.move_end(),
        (_, KeyCode::Tab) => app.focus = next_focus_from_insert(app),
        _ => {}
    }
}

fn submit_or_new_line(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        app.input.new_line();
        return;
    }
    submit_input(app);
}

fn set_insert_focus(app: &mut App) {
    app.input_mode = InputMode::Insert;
    app.focus = FocusPanel::Input;
}

fn next_focus_from_normal(app: &App) -> FocusPanel {
    match app.focus {
        FocusPanel::Chat => FocusPanel::Input,
        FocusPanel::Input => {
            if app.show_tool_panel {
                FocusPanel::Tools
            } else {
                FocusPanel::Chat
            }
        }
        FocusPanel::Tools => FocusPanel::Chat,
        FocusPanel::CommandPalette => FocusPanel::Input,
    }
}

fn next_focus_from_insert(app: &App) -> FocusPanel {
    match app.focus {
        FocusPanel::Input => FocusPanel::Chat,
        FocusPanel::Chat => {
            if app.show_tool_panel {
                FocusPanel::Tools
            } else {
                FocusPanel::Input
            }
        }
        FocusPanel::Tools => FocusPanel::Input,
        FocusPanel::CommandPalette => FocusPanel::Input,
    }
}

fn copy_last_assistant_message(app: &mut App) {
    let last = app
        .chat
        .messages()
        .iter()
        .rev()
        .find(|m| m.role == super::chat::MessageRole::Assistant);
    if let Some(msg) = last {
        copy_to_clipboard(&msg.content);
        app.push_timestamped_message(
            super::chat::MessageRole::System,
            "Copied last response to clipboard.".to_string(),
        );
    }
}

fn copy_all_messages(app: &mut App) {
    let text = app
        .chat
        .messages()
        .iter()
        .map(|m| format!("[{}] {}: {}", m.timestamp, m.role.label(), m.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    copy_to_clipboard(&text);
    app.push_timestamped_message(
        super::chat::MessageRole::System,
        "Copied all messages to clipboard.".to_string(),
    );
}

fn copy_to_clipboard(text: &str) {
    // Use platform clipboard via subprocess — avoids heavy native dependencies
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        if let Ok(mut child) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}
