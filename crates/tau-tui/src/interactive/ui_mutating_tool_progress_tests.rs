use ratatui::{backend::TestBackend, Terminal};

use super::{
    app::{App, AppConfig},
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

fn build_app(prompt: &str, state: AgentStateDisplay) -> App {
    let mut app = App::new(AppConfig::default());
    app.push_message(MessageRole::User, prompt.to_string());
    app.status.agent_state = state;
    app
}

#[test]
fn red_spec_3607_build_turn_with_running_write_shows_live_mutating_progress() {
    let mut app = build_app(
        "create a playable breakout clone",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Running,
        "src/game.js".to_string(),
    );

    let text = render_text(&mut app);

    assert!(text.contains("Mutating now:"), "rendered:\n{text}");
    assert!(text.contains("src/game.js"), "rendered:\n{text}");
}

#[test]
fn red_spec_3607_build_turn_with_successful_write_shows_latest_mutating_target_path() {
    let mut app = build_app(
        "build a snake game",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Success,
        "src/snake.js".to_string(),
    );

    let text = render_text(&mut app);

    assert!(text.contains("Latest write target:"), "rendered:\n{text}");
    assert!(text.contains("src/snake.js"), "rendered:\n{text}");
}

#[test]
fn red_spec_3607_build_turn_with_only_read_activity_omits_mutating_progress_wording() {
    let mut app = build_app(
        "create a snake game",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "README.md".to_string(),
    );

    let text = render_text(&mut app);

    assert!(!text.contains("Mutating now:"), "rendered:\n{text}");
    assert!(!text.contains("Latest write target:"), "rendered:\n{text}");
}

#[test]
fn integration_spec_3607_build_turn_transition_from_read_to_edit_updates_live_progress() {
    let mut app = build_app(
        "create a frogger tetris mashup",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "README.md".to_string(),
    );
    let read_only = render_text(&mut app);
    assert!(read_only.contains("read-only so far"), "rendered:\n{read_only}");

    app.push_tool_event(
        "edit".to_string(),
        ToolStatus::Running,
        "src/game.rs".to_string(),
    );

    let mutating = render_text(&mut app);

    assert!(mutating.contains("Mutating now:"), "rendered:\n{mutating}");
    assert!(mutating.contains("src/game.rs"), "rendered:\n{mutating}");
}
