use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

use crate::interactive::app::{App, AppConfig};
use crate::interactive::chat::MessageRole;

use super::render;

#[test]
fn red_spec_3582_default_layout_collapses_detail_drawer_until_requested() {
    let mut app = App::new(AppConfig::default());
    app.push_message(
        MessageRole::Assistant,
        "Transcript should own the main canvas.".to_string(),
    );

    let rendered = render_app(&mut app, 120, 32);

    assert!(!rendered.contains("Tools ("));
}

#[test]
fn red_spec_3582_status_bar_surfaces_session_and_approval_context() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 10);

    assert!(rendered.contains("session"));
    assert!(rendered.contains("approval"));
}

#[test]
fn red_spec_3582_composer_hints_expose_interrupt_retry_and_details_actions() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("interrupt"));
    assert!(rendered.contains("retry"));
    assert!(rendered.contains("details"));
}

#[test]
fn red_spec_3582_composer_uses_action_chips_instead_of_instruction_sentence() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("[/] commands"));
    assert!(rendered.contains("[Enter] send"));
    assert!(!rendered.contains("Press / for commands"));
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
fn red_spec_3582_error_attention_strip_exposes_retry_and_details_actions() {
    let mut app = App::new(AppConfig::default());
    app.status.agent_state = crate::interactive::status::AgentStateDisplay::Error;
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Retry turn"));
    assert!(rendered.contains("Open details"));
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
fn integration_spec_3582_approve_command_resolves_pending_approval_through_real_input_path() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/approval-needed");
    submit_command(&mut app, "/approve");

    let rendered = render_app(&mut app, 120, 28);

    assert!(!rendered.contains("Approval required"));
    assert!(rendered.contains("Approval approved"));
}

#[test]
fn integration_spec_3582_approval_shortcuts_resolve_request_without_leaving_insert_mode() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/approval-needed");
    app.handle_key(key(KeyCode::Char('y')));

    let rendered = render_app(&mut app, 120, 28);

    assert!(!rendered.contains("Approval required"));
    assert!(rendered.contains("Approval approved"));
}

#[test]
fn red_spec_3582_approval_attention_strip_exposes_approve_and_reject_actions() {
    let mut app = App::new(AppConfig::default());
    for ch in "/approval-needed".chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));

    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Approval required"));
    assert!(rendered.contains("Approve"));
    assert!(rendered.contains("Reject"));
    assert!(rendered.contains("[Y] approve"));
    assert!(rendered.contains("[N] reject"));
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

#[test]
fn red_spec_3582_memory_detail_surfaces_degraded_state_marker() {
    let mut app = App::new(AppConfig::default());
    submit_command(&mut app, "/memory");

    let rendered = render_app(&mut app, 140, 32);

    assert!(rendered.contains("degraded"));
    assert!(rendered.contains("shared state unavailable"));
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

fn render_app(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal.draw(|frame| render(frame, app)).expect("draw");
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn submit_command(app: &mut App, command: &str) {
    for ch in command.chars() {
        app.handle_key(key(KeyCode::Char(ch)));
    }
    app.handle_key(key(KeyCode::Enter));
}
