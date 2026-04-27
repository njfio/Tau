use ratatui::{backend::TestBackend, Terminal};

use super::{
    app::{App, AppConfig},
    chat::{ChatMessage, MessageRole},
    status::AgentStateDisplay,
    tools::ToolStatus,
    ui,
};

fn render_text(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|frame| ui::render(frame, app)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let mut lines = Vec::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buffer.cell((x, y)).expect("cell").symbol());
        }
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
}

fn set_input(app: &mut App, text: &str) {
    app.input.clear();
    for ch in text.chars() {
        app.input.insert_char(ch);
    }
}

#[test]
fn transcript_first_layout_labels_primary_panel_and_keeps_shell_regions_visible() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    app.status.active_mission_id = Some("mission-layout".to_string());
    app.status.agent_state = AgentStateDisplay::ToolExec;
    app.chat.add_message(ChatMessage {
        role: MessageRole::Assistant,
        content: "snapshot says ready".to_string(),
        timestamp: "12:00:00".to_string(),
    });
    app.push_tool_event(
        "read_file".to_string(),
        ToolStatus::Running,
        "docs/architecture/tui-operator-state-consumption-v1.md".to_string(),
    );
    set_input(&mut app, "continue layout work");

    let rendered = render_text(&mut app, 120, 28);

    assert!(
        rendered.contains("Transcript"),
        "expected primary panel to be transcript-first, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("snapshot says ready"),
        "expected assistant transcript content to remain visible, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("Mission: mission-layout"),
        "expected status bar mission chip to remain visible, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("Input"),
        "expected input panel to remain visible, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("continue layout work"),
        "expected typed input to remain visible, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("Tools (1 active / 1 total)"),
        "expected secondary tool panel to remain visible, rendered:\n{rendered}"
    );
}

#[test]
fn transcript_first_layout_surfaces_current_turn_status_inside_transcript() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    app.status.agent_state = AgentStateDisplay::ToolExec;
    app.chat.add_message(ChatMessage {
        role: MessageRole::User,
        content: "inspect the contract".to_string(),
        timestamp: "12:00:00".to_string(),
    });
    app.push_tool_event(
        "read_file".to_string(),
        ToolStatus::Running,
        "crates/tau-contract/src/operator_state.rs".to_string(),
    );

    let rendered = render_text(&mut app, 100, 24);

    assert!(
        rendered.contains("Current turn: TOOL"),
        "expected transcript panel to surface current turn status, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("Active tool: read_file"),
        "expected current turn strip to surface active tool progress, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("inspect the contract"),
        "expected transcript content to remain primary, rendered:\n{rendered}"
    );
}
