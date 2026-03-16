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
fn red_spec_3582_composer_uses_prompt_shell_instead_of_bordered_panel_title() {
    let mut app = App::new(AppConfig::default());
    let rendered = render_app(&mut app, 120, 24);

    assert!(rendered.contains("Press / for commands"));
    assert!(!rendered.contains("Composer"));
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
fn red_spec_3582_narrow_layout_collapses_details_drawer_first() {
    let mut app = App::new(AppConfig::default());
    app.show_tool_panel = true;
    let rendered = render_app(&mut app, 72, 22);

    assert!(!rendered.contains(" Details "));
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
