use crate::interactive::{
    app::{App, AppConfig},
    status::AgentStateDisplay,
    tools::ToolStatus,
};

use super::helpers::render_app;

fn build_app(prompt: &str) -> App {
    let mut app = App::new(AppConfig::default());
    app.last_submitted_input = Some(prompt.to_string());
    app.status.agent_state = AgentStateDisplay::Thinking;
    app
}

#[test]
fn red_spec_3604_live_activity_flags_missing_mutating_evidence_for_build_prompt() {
    let mut app = build_app("create a phaser game prototype");

    let rendered = render_app(&mut app, 100, 24);

    assert!(
        rendered.contains("no mutating evidence yet"),
        "expected missing-evidence status in live activity, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3604_live_activity_flags_read_only_build_turn() {
    let mut app = build_app("build a phaser game prototype");
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "src/main.rs".to_string(),
    );

    let rendered = render_app(&mut app, 100, 24);

    assert!(
        rendered.contains("read-only so far"),
        "expected read-only status in live activity, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3604_run_state_flags_read_only_build_turn() {
    let mut app = build_app("create a phaser game prototype");
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "src/main.rs".to_string(),
    );

    let rendered = render_app(&mut app, 100, 24);

    assert!(
        rendered.contains("still read-only"),
        "expected read-only summary in run-state card, rendered:\n{rendered}"
    );
}

#[test]
fn red_spec_3604_real_render_path_confirms_mutating_evidence_with_side_panel_enabled() {
    let mut app = build_app("create a phaser game prototype");
    app.show_tool_panel = true;
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Success,
        "game.js".to_string(),
    );

    let rendered = render_app(&mut app, 120, 28);

    assert!(
        rendered.contains("mutating evidence confirmed"),
        "expected confirmed mutating-evidence status in live activity, rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("write/edit confirmed"),
        "expected confirmed mutating-evidence summary in run-state card, rendered:\n{rendered}"
    );
}

#[test]
fn integration_spec_3604_non_build_prompt_omits_mutating_evidence_status() {
    let mut app = build_app("what does the renderer do here?");
    app.push_tool_event(
        "read".to_string(),
        ToolStatus::Success,
        "src/main.rs".to_string(),
    );

    let rendered = render_app(&mut app, 100, 24);

    assert!(
        !rendered.contains("mutating evidence"),
        "did not expect mutating-evidence messaging for non-build prompt, rendered:\n{rendered}"
    );
    assert!(
        !rendered.contains("read-only so far"),
        "did not expect read-only build messaging for non-build prompt, rendered:\n{rendered}"
    );
}

#[test]
fn integration_spec_3604_idle_build_turn_omits_mutating_evidence_status() {
    let mut app = build_app("create a phaser game prototype");
    app.status.agent_state = AgentStateDisplay::Idle;
    app.push_tool_event(
        "write".to_string(),
        ToolStatus::Success,
        "game.js".to_string(),
    );

    let rendered = render_app(&mut app, 100, 24);

    assert!(
        !rendered.contains("mutating evidence confirmed"),
        "did not expect sticky live evidence after turn completion, rendered:\n{rendered}"
    );
    assert!(
        !rendered.contains("write/edit confirmed"),
        "did not expect sticky run-state evidence after turn completion, rendered:\n{rendered}"
    );
}
