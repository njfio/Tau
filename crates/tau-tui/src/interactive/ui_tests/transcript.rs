use crate::interactive::app::{App, AppConfig};
use crate::interactive::chat::MessageRole;

use super::helpers::{render_app, submit_command};

#[test]
fn red_spec_3582_status_bar_surfaces_session_and_approval_context() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 10);

    assert!(rendered.contains("session"));
    assert!(rendered.contains("approval"));
}

#[test]
fn red_spec_3582_transcript_shows_live_activity_summary_above_messages() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Live activity"));
    assert!(rendered.contains("Thinking through the next step"));
}

#[test]
fn red_spec_3582_live_activity_surfaces_active_tool_count_and_name() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Inspecting repository layout".to_string(),
    );
    app.push_tool_event(
        "http".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Fetching page data".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("2 active"));
    assert!(rendered.contains("http"));
}

#[test]
fn red_spec_3582_transcript_surfaces_active_turn_card_with_prompt_context() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Thinking;
    app.last_submitted_input = Some("Research Aleo private apps platform".to_string());

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Turn active"));
    assert!(rendered.contains("Research Aleo private apps platform"));
    assert!(rendered.contains("thinking"));
}

#[test]
fn red_spec_3582_streaming_turn_card_surfaces_assistant_preview() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Streaming;
    app.last_submitted_input = Some("summarize the company".to_string());
    app.push_message(
        MessageRole::Assistant,
        "Aleo builds infrastructure for private applications and zero-knowledge workflows."
            .to_string(),
    );

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("streaming"));
    assert!(rendered.contains("assistant output arriving"));
    assert!(rendered.contains("Aleo builds infrastructure"));
}

#[test]
fn red_spec_3582_transcript_surfaces_running_tool_card_without_opening_drawer() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::ToolExec;
    app.push_tool_event(
        "bash".to_string(),
        crate::interactive::tools::ToolStatus::Running,
        "Inspecting repository layout".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("Running tool"));
    assert!(rendered.contains("bash"));
    assert!(rendered.contains("Inspecting repository layout"));
}

#[test]
fn integration_spec_3582_prompt_submission_surfaces_last_turn_summary_card() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "plan the tui redesign");

    let rendered = render_app(&mut app, 120, 30);

    assert!(rendered.contains("Last turn"));
    assert!(rendered.contains("plan the tui redesign"));
    assert!(rendered.contains("assistant reply ready"));
}

#[test]
fn red_spec_3582_transcript_renders_messages_as_cards() {
    let mut app = App::new(AppConfig::default());
    app.push_message(MessageRole::User, "Build me something useful.".to_string());
    app.push_message(
        MessageRole::Assistant,
        "I can do that. First I need repository context.".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(rendered.contains("╭─ You"));
    assert!(rendered.contains("╭─ Tau"));
    assert!(rendered.contains("│ I can do that."));
}
