use std::path::PathBuf;

use super::{
    app::{App, AppConfig},
    app_runtime::seed_welcome_message,
    chat::{ChatMessage, MessageRole},
    gateway_client::GatewayRuntimeConfig,
    transcript_state::{
        load_interactive_transcript_state, save_interactive_transcript_state,
        InteractiveTranscriptState,
    },
};

fn build_app_with_paths(local_state_path: PathBuf, local_transcript_path: PathBuf) -> App {
    App::new(AppConfig {
        local_state_path,
        local_transcript_path,
        gateway: GatewayRuntimeConfig::default(),
        ..AppConfig::default()
    })
}

fn fixture_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: MessageRole::System,
            content: "Welcome back".to_string(),
            timestamp: "09:00:00".to_string(),
        },
        ChatMessage {
            role: MessageRole::User,
            content: "build the gateway loop".to_string(),
            timestamp: "09:01:00".to_string(),
        },
        ChatMessage {
            role: MessageRole::Assistant,
            content: "working on it".to_string(),
            timestamp: "09:01:10".to_string(),
        },
        ChatMessage {
            role: MessageRole::Tool,
            content: "read ok".to_string(),
            timestamp: "09:01:12".to_string(),
        },
    ]
}

#[test]
fn red_spec_3681_saved_transcript_restores_messages_into_new_app() {
    let temp = tempfile::tempdir().expect("tempdir");
    let local_state_path = temp.path().join("interactive-session.json");
    let local_transcript_path = temp.path().join("interactive-transcript.json");
    let transcript = InteractiveTranscriptState {
        schema_version: 1,
        messages: fixture_messages(),
    };
    save_interactive_transcript_state(&local_transcript_path, &transcript)
        .expect("save transcript state");

    let loaded =
        load_interactive_transcript_state(&local_transcript_path).expect("load transcript state");
    let mut app = build_app_with_paths(local_state_path, local_transcript_path);
    app.restore_local_transcript_state(loaded);

    assert_eq!(app.chat.messages(), transcript.messages.as_slice());
    app.search_transcript("gateway");
    assert!(app.chat.search_state().is_some());
}

#[test]
fn red_spec_3681_save_writes_interactive_transcript_json_snapshot() {
    let temp = tempfile::tempdir().expect("tempdir");
    let local_transcript_path = temp
        .path()
        .join(".tau")
        .join("tui")
        .join("interactive-transcript.json");
    let transcript = InteractiveTranscriptState {
        schema_version: 1,
        messages: fixture_messages(),
    };

    save_interactive_transcript_state(&local_transcript_path, &transcript)
        .expect("save transcript state");

    let body = std::fs::read_to_string(&local_transcript_path).expect("read transcript body");
    assert!(body.contains("\"schema_version\": 1"), "body={body}");
    assert!(body.contains("\"role\": \"user\""), "body={body}");
    assert!(body.contains("\"build the gateway loop\""), "body={body}");
    assert!(body.contains("\"role\": \"tool\""), "body={body}");
}

#[test]
fn red_spec_3681_invalid_or_missing_transcript_state_fails_soft() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing_path = temp.path().join("missing.json");
    assert_eq!(load_interactive_transcript_state(&missing_path), None);

    let invalid_path = temp.path().join("invalid.json");
    std::fs::write(&invalid_path, "{ not-json").expect("write invalid state");
    assert_eq!(load_interactive_transcript_state(&invalid_path), None);

    let app = build_app_with_paths(temp.path().join("state.json"), invalid_path);
    assert!(app.chat.is_empty());
}

#[test]
fn red_spec_3681_seed_welcome_message_does_not_duplicate_restored_transcript() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut app = build_app_with_paths(
        temp.path().join("interactive-session.json"),
        temp.path().join("interactive-transcript.json"),
    );
    app.restore_local_transcript_state(InteractiveTranscriptState {
        schema_version: 1,
        messages: fixture_messages(),
    });

    let original_len = app.chat.len();
    seed_welcome_message(&mut app);

    assert_eq!(app.chat.len(), original_len);
    assert_eq!(app.chat.messages()[0].content, "Welcome back");
}
