use std::sync::{Mutex, OnceLock};

use ratatui::{backend::TestBackend, Terminal};

use super::{
    app::{App, AppConfig},
    app_commands,
    chat::MessageRole,
    status::AgentStateDisplay,
    tools::ToolStatus,
    ui,
};

fn render_text(app: &mut App) -> String {
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|frame| ui::render(frame, app)).expect("draw");
    let mut lines = Vec::new();
    let buffer = terminal.backend().buffer();

    for y in 0..24 {
        let mut line = String::new();
        for x in 0..100 {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line);
    }

    lines.join("\n")
}

fn build_app(prompt: &str) -> App {
    let mut app = App::new(AppConfig::default());
    app.push_message(MessageRole::User, prompt.to_string());
    app.status.agent_state = AgentStateDisplay::ToolExec;
    app
}

fn set_input(app: &mut App, text: &str) {
    app.input.clear();
    for ch in text.chars() {
        app.input.insert_char(ch);
    }
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

fn last_system_message(app: &App) -> Option<&str> {
    app.chat
        .messages()
        .iter()
        .rev()
        .find(|msg| msg.role == MessageRole::System)
        .map(|msg| msg.content.as_str())
}

#[test]
fn red_spec_3609_mutating_transcript_entry_surfaces_target_breadcrumb() {
    let mut app = build_app("build a snake game");
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Success,
        "src/snake.js".to_string(),
    );

    let rendered = render_text(&mut app);

    assert!(
        rendered.contains("Mutating target:"),
        "rendered:\n{rendered}"
    );
    assert!(rendered.contains("src/snake.js"), "rendered:\n{rendered}");
}

#[test]
fn red_spec_3609_non_mutating_transcript_entry_stays_generic() {
    let mut app = build_app("inspect the repo");
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "README.md".to_string(),
    );

    let rendered = render_text(&mut app);

    assert!(
        !rendered.contains("Mutating target:"),
        "rendered:\n{rendered}"
    );
    assert!(rendered.contains("read ok"), "rendered:\n{rendered}");
}

#[test]
fn red_spec_3609_copy_target_command_exports_latest_mutating_target() {
    let _guard = env_lock();
    let export_path = std::env::temp_dir().join("tau-copy-target-test.txt");
    let _ = std::fs::remove_file(&export_path);
    std::env::set_var(
        "TAU_TUI_CLIPBOARD_COMMAND",
        format!("cat > {}", export_path.display()),
    );

    let mut app = build_app("build a snake game");
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Success,
        "src/snake.js".to_string(),
    );
    set_input(&mut app, "/copy-target");

    app_commands::submit_input(&mut app);

    let system = last_system_message(&app).unwrap_or_default().to_string();
    let exported = std::fs::read_to_string(&export_path).unwrap_or_default();
    std::env::remove_var("TAU_TUI_CLIPBOARD_COMMAND");
    let _ = std::fs::remove_file(&export_path);

    assert!(
        system.contains("Copied latest mutating target"),
        "system={system}"
    );
    assert_eq!(exported, "src/snake.js");
}

#[test]
fn red_spec_3609_copy_target_command_fails_loudly_without_mutating_target() {
    let _guard = env_lock();
    std::env::remove_var("TAU_TUI_CLIPBOARD_COMMAND");

    let mut app = build_app("build a snake game");
    set_input(&mut app, "/copy-target");

    app_commands::submit_input(&mut app);

    let system = last_system_message(&app).unwrap_or_default();
    assert!(
        system.contains("No successful mutating target available"),
        "system={system}"
    );
}
