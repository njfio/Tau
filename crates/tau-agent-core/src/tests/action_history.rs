use std::{collections::VecDeque, path::Path};

use serde_json::json;
use tau_ai::{ChatResponse, ChatUsage, ContentBlock, Message};
use tau_memory::action_history::{
    ActionFilter, ActionHistoryConfig, ActionHistoryStore, ActionType,
};

use super::*;

fn action_history_agent_config(path: &Path) -> AgentConfig {
    AgentConfig {
        action_history_enabled: true,
        action_history_store_path: Some(path.to_path_buf()),
        ..AgentConfig::default()
    }
}

fn load_history(path: &Path) -> ActionHistoryStore {
    ActionHistoryStore::load(
        path,
        30,
        ActionHistoryConfig {
            store_path: path.to_path_buf(),
            max_records_per_session: 1000,
            max_total_records: 1000,
        },
    )
    .expect("load action history")
}

fn scripted_tool_call(id: &str, name: &str, arguments: serde_json::Value) -> ChatResponse {
    ChatResponse {
        message: Message::assistant_blocks(vec![ContentBlock::ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments,
        }]),
        finish_reason: Some("tool_calls".to_string()),
        usage: ChatUsage::default(),
    }
}

#[tokio::test]
async fn regression_prompt_persists_action_history_store_when_enabled() {
    let temp = tempfile::tempdir().expect("tempdir");
    let history_path = temp.path().join("action_history.jsonl");
    let client = Arc::new(MockClient {
        responses: AsyncMutex::new(VecDeque::from([ChatResponse {
            message: Message::assistant_text("done"),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        }])),
    });

    let mut agent = Agent::new(client, action_history_agent_config(&history_path));
    let _ = agent.prompt("hello").await.expect("prompt should succeed");

    assert!(
        history_path.exists(),
        "prompt() should persist the action-history store path"
    );
}

#[tokio::test]
async fn regression_prompt_with_stream_persists_action_history_store_when_enabled() {
    let temp = tempfile::tempdir().expect("tempdir");
    let history_path = temp.path().join("action_history.jsonl");
    let client = Arc::new(StreamingMockClient {
        response: ChatResponse {
            message: Message::assistant_text("done"),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        },
        deltas: vec!["d".to_string(), "o".to_string(), "ne".to_string()],
    });

    let mut agent = Agent::new(client, action_history_agent_config(&history_path));
    let sink = Arc::new(|_delta: String| {});
    let _ = agent
        .prompt_with_stream("hello", Some(sink))
        .await
        .expect("streaming prompt should succeed");

    assert!(
        history_path.exists(),
        "prompt_with_stream() should persist the action-history store path"
    );
}

#[tokio::test]
async fn regression_prompt_json_persists_action_history_store_when_enabled() {
    let temp = tempfile::tempdir().expect("tempdir");
    let history_path = temp.path().join("action_history.jsonl");
    let client = Arc::new(MockClient {
        responses: AsyncMutex::new(VecDeque::from([ChatResponse {
            message: Message::assistant_text(r#"{"status":"ok"}"#),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        }])),
    });
    let schema = json!({
        "type": "object",
        "properties": {
            "status": { "type": "string" }
        },
        "required": ["status"],
        "additionalProperties": false
    });

    let mut agent = Agent::new(client, action_history_agent_config(&history_path));
    let parsed = agent
        .prompt_json("return json", &schema)
        .await
        .expect("prompt_json should succeed");

    assert_eq!(parsed["status"], "ok");
    assert!(
        history_path.exists(),
        "prompt_json() should persist the action-history store path"
    );
}

#[tokio::test]
async fn spec_3624_c03_tool_history_records_real_turn_and_latency_values() {
    let temp = tempfile::tempdir().expect("tempdir");
    let history_path = temp.path().join("action_history.jsonl");
    let client = Arc::new(MockClient {
        responses: AsyncMutex::new(VecDeque::from([
            scripted_tool_call("call-read", "read", json!({ "path": "README.md" })),
            ChatResponse {
                message: Message::assistant_text("done"),
                finish_reason: Some("stop".to_string()),
                usage: ChatUsage::default(),
            },
        ])),
    });

    let mut agent = Agent::new(client, action_history_agent_config(&history_path));
    agent.register_tool(ReadTool);
    let _ = agent
        .prompt("read the file")
        .await
        .expect("prompt should succeed");

    let store = load_history(&history_path);
    let records = store.query(&ActionFilter {
        action_type: Some(ActionType::ToolExecution),
        ..Default::default()
    });
    assert_eq!(
        records.len(),
        1,
        "expected exactly one recorded tool execution"
    );
    let record = records[0];
    assert_eq!(record.tool_name.as_deref(), Some("read"));
    assert_eq!(record.turn, 1, "tool execution should record its real turn");
    assert!(
        record.latency_ms > 0,
        "tool execution latency should be measured rather than stored as a placeholder"
    );
}
