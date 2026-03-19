use ratatui::{backend::TestBackend, Terminal};

use super::{
    app::{App, AppConfig},
    status::AgentStateDisplay,
    tools::ToolStatus,
    ui,
};

fn render_text(app: &App, width: u16, height: u16) -> String {
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

#[test]
fn red_spec_3592_main_shell_surfaces_running_tool_activity_without_tools_panel() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = false;
    app.status.agent_state = AgentStateDisplay::ToolExec;
    app.push_tool_event("bash".to_string(), ToolStatus::Running, "pwd".to_string());

    let rendered = render_text(&app, 100, 24);

    assert!(
        rendered.contains("Running tool: bash"),
        "expected running tool summary in main shell, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3592_main_shell_surfaces_last_failed_tool_summary() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = false;
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Failed,
        "permission denied".to_string(),
    );

    let rendered = render_text(&app, 100, 24);

    assert!(
        rendered.contains("Last tool failed: write"),
        "expected failed tool summary in main shell, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("permission denied"),
        "expected failed tool detail in main shell, rendered:\n{rendered}"
    );
}

#[test]
fn integration_spec_3592_real_render_path_keeps_tool_panel_and_main_shell_tool_state() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    app.status.agent_state = AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        ToolStatus::Success,
        "/Users/n/RustroverProjects/rust_pi-3592".to_string(),
    );

    let rendered = render_text(&app, 120, 28);

    assert!(
        rendered.contains("Tools (0 active / 1 total)"),
        "expected side tools panel to remain visible, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("Last tool: bash"),
        "expected main shell recent tool summary, rendered:\n{rendered}"
    );
}
