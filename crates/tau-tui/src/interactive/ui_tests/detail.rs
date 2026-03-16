use crossterm::event::KeyCode;

use crate::interactive::app::{App, AppConfig};

use super::helpers::{key, render_app, submit_command};

#[test]
fn red_spec_3582_default_layout_collapses_detail_drawer_until_requested() {
    let mut app = App::new(AppConfig::default());
    app.push_message(
        crate::interactive::chat::MessageRole::Assistant,
        "Transcript should own the main canvas.".to_string(),
    );

    let rendered = render_app(&mut app, 120, 32);

    assert!(!rendered.contains("Tools ("));
}

#[test]
fn red_spec_3582_details_drawer_exposes_context_sections_beyond_tools() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("Memory"));
    assert!(rendered.contains("Cortex"));
    assert!(rendered.contains("Sessions"));
}

#[test]
fn red_spec_3582_details_drawer_uses_tabbed_context_navigation() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("[tools]"));
    assert!(rendered.contains("memory"));
    assert!(rendered.contains("cortex"));
    assert!(rendered.contains("sessions"));
}

#[test]
fn red_spec_3582_narrow_layout_uses_detail_overlay() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    let rendered = render_app(&mut app, 72, 22);

    assert!(rendered.contains("Quick details"));
    assert!(rendered.contains("[tools]"));
}

#[test]
fn integration_spec_3582_memory_command_switches_detail_context_through_real_input_path() {
    let mut app = App::new(AppConfig::default());
    for ch in "/memory".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("[memory]"));
    assert!(rendered.contains("No stored memory yet."));
}

#[test]
fn red_spec_3582_bare_memory_command_switches_detail_context() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "memory");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("[memory]"));
}

#[test]
fn red_spec_3582_memory_detail_surfaces_degraded_state_marker() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/memory");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("degraded"));
    assert!(rendered.contains("shared state unavailable"));
}

#[test]
fn red_spec_3582_memory_detail_surfaces_recent_user_context() {
    let mut app = App::new(AppConfig::default());
    app.push_message(
        crate::interactive::chat::MessageRole::User,
        "I like Python, Rust, and TypeScript.".to_string(),
    );
    submit_command(&mut app, "/memory");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("Recent user context"));
    assert!(rendered.contains("Python, Rust, and TypeScript"));
}

#[test]
fn red_spec_3582_cortex_detail_surfaces_live_runtime_posture() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Inspecting repository layout".to_string(),
    );
    submit_command(&mut app, "/approval-needed");
    submit_command(&mut app, "/cortex");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("Observer posture"));
    assert!(rendered.contains("State: tool"));
    assert!(rendered.contains("Active tools: 1"));
    assert!(rendered.contains("Pending approval: yes"));
}

#[test]
fn integration_spec_3582_sessions_detail_surfaces_real_session_metrics() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "testing");
    submit_command(&mut app, "/sessions");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("Messages"));
    assert!(rendered.contains("Tokens"));
    assert!(rendered.contains("Approvals pending"));
}

#[test]
fn red_spec_3582_bare_sessions_and_cortex_commands_switch_detail_context() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "sessions");
    let sessions = render_app(&mut app, 140, 32);
    assert!(sessions.contains("[sessions]"));

    submit_command(&mut app, "cortex");
    let cortex = render_app(&mut app, 140, 32);
    assert!(cortex.contains("[cortex]"));
}

#[test]
fn red_spec_3582_sessions_detail_surfaces_last_prompt_and_assistant_count() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "testing");
    submit_command(&mut app, "/sessions");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("Last prompt"));
    assert!(rendered.contains("testing"));
    assert!(rendered.contains("Assistant msgs"));
}
