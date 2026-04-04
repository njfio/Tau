use std::path::PathBuf;

use super::{
    app::{App, AppConfig},
    gateway_client::GatewayRuntimeConfig,
    session_state::{
        load_interactive_session_state, save_interactive_session_state, InteractiveSessionState,
    },
};

fn build_app_with_state_path(local_state_path: PathBuf) -> App {
    App::new(AppConfig {
        local_state_path,
        gateway: GatewayRuntimeConfig {
            session_key: "session-default".to_string(),
            ..GatewayRuntimeConfig::default()
        },
        ..AppConfig::default()
    })
}

#[test]
fn red_spec_3680_saved_snapshot_restores_app_state_into_new_instance() {
    let temp = tempfile::tempdir().expect("tempdir");
    let local_state_path = temp.path().join("interactive-session.json");

    let mut original = build_app_with_state_path(local_state_path.clone());
    original.restore_local_session_state(InteractiveSessionState {
        schema_version: 1,
        input_draft: "rebuild the gateway retry loop".to_string(),
        prompt_history: vec!["first prompt".to_string(), "second prompt".to_string()],
        gateway_session_key: Some("session-alpha".to_string()),
        active_mission_id: Some("mission-alpha".to_string()),
    });

    let snapshot = original.local_session_state();
    save_interactive_session_state(&local_state_path, &snapshot).expect("save local state");

    let restored_state =
        load_interactive_session_state(&local_state_path).expect("load saved session state");
    let mut restored = build_app_with_state_path(local_state_path);
    restored.restore_local_session_state(restored_state);

    let restored_snapshot = restored.local_session_state();
    assert_eq!(restored.input.get_text(), "rebuild the gateway retry loop");
    assert_eq!(
        restored_snapshot.prompt_history,
        vec!["first prompt".to_string(), "second prompt".to_string()]
    );
    assert_eq!(
        restored.config.gateway.session_key, "session-alpha",
        "session key should restore"
    );
    assert_eq!(
        restored.config.gateway.mission_id.as_deref(),
        Some("mission-alpha")
    );
    assert_eq!(
        restored.status.active_mission_id.as_deref(),
        Some("mission-alpha")
    );
}

#[test]
fn red_spec_3680_save_writes_interactive_session_json_snapshot() {
    let temp = tempfile::tempdir().expect("tempdir");
    let local_state_path = temp
        .path()
        .join(".tau")
        .join("tui")
        .join("interactive-session.json");
    let state = InteractiveSessionState {
        schema_version: 1,
        input_draft: "draft prompt".to_string(),
        prompt_history: vec!["alpha".to_string(), "beta".to_string()],
        gateway_session_key: Some("session-beta".to_string()),
        active_mission_id: Some("mission-beta".to_string()),
    };

    save_interactive_session_state(&local_state_path, &state).expect("save local state");

    let body = std::fs::read_to_string(&local_state_path).expect("read saved local state");
    assert!(body.contains("\"schema_version\": 1"), "body={body}");
    assert!(
        body.contains("\"input_draft\": \"draft prompt\""),
        "body={body}"
    );
    assert!(body.contains("\"alpha\""), "body={body}");
    assert!(body.contains("\"session-beta\""), "body={body}");
    assert!(body.contains("\"mission-beta\""), "body={body}");
}

#[test]
fn red_spec_3680_invalid_or_missing_local_state_fails_soft() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing_path = temp.path().join("missing.json");
    assert_eq!(load_interactive_session_state(&missing_path), None);

    let invalid_path = temp.path().join("invalid.json");
    std::fs::write(&invalid_path, "{ not-json").expect("write invalid state");
    assert_eq!(load_interactive_session_state(&invalid_path), None);

    let mut app = build_app_with_state_path(invalid_path);
    if let Some(state) = load_interactive_session_state(app.config.local_state_path.as_path()) {
        app.restore_local_session_state(state);
    }

    let snapshot = app.local_session_state();
    assert_eq!(snapshot.input_draft, "");
    assert!(snapshot.prompt_history.is_empty());
    assert_eq!(
        snapshot.gateway_session_key.as_deref(),
        Some("session-default")
    );
    assert_eq!(snapshot.active_mission_id, None);
}

#[test]
fn red_spec_3680_default_app_config_uses_interactive_session_state_path() {
    let config = AppConfig::default();
    assert_eq!(
        config.local_state_path,
        PathBuf::from(".tau/tui/interactive-session.json")
    );
}
