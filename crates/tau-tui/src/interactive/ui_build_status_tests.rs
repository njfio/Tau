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
fn integration_spec_3604_active_build_turn_without_tool_evidence_shows_missing_mutation_status() {
    let mut app = build_app(
        "create a snake and tetris mashup game using phaserjs",
        AgentStateDisplay::Thinking,
    );

    let text = render_text(&mut app);

    assert!(text.contains("no mutating evidence yet"));
}

#[test]
fn integration_spec_3604_active_build_turn_with_successful_read_shows_read_only_status() {
    let mut app = build_app(
        "build a playable frogger clone",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event("read".to_string(), ToolStatus::Success, "src/main.rs".to_string());

    let text = render_text(&mut app);

    assert!(text.contains("read-only so far"));
}

#[test]
fn integration_spec_3604_active_build_turn_with_successful_write_shows_mutating_status() {
    let mut app = build_app(
        "create a playable breakout game",
        AgentStateDisplay::ToolExec,
    );
    app.push_tool_event("write".to_string(), ToolStatus::Success, "game.js".to_string());

    let text = render_text(&mut app);

    assert!(text.contains("mutating evidence confirmed"));
}

#[test]
fn integration_spec_3604_idle_turn_omits_mutating_evidence_status() {
    let mut app = build_app(
        "create a playable breakout game",
        AgentStateDisplay::Idle,
    );

    let text = render_text(&mut app);

    assert!(!text.contains("mutating evidence"));
    assert!(!text.contains("read-only so far"));
}

#[test]
fn integration_spec_3604_non_build_turn_omits_mutating_evidence_status() {
    let mut app = build_app("what is blue?", AgentStateDisplay::Thinking);
    app.push_tool_event("read".to_string(), ToolStatus::Success, "README.md".to_string());

    let text = render_text(&mut app);

    assert!(!text.contains("mutating evidence"));
    assert!(!text.contains("read-only so far"));
}
