use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;

use tau_ai::{ChatRequest, ChatResponse, ChatUsage, LlmClient, Message, TauAiError};
use tau_memory::memory_contract::{MemoryEntry, MemoryScope};

use crate::{Cortex, CortexConfig};

#[derive(Debug, Clone)]
struct QueueClient {
    replies: Arc<AsyncMutex<VecDeque<Result<String, TauAiError>>>>,
}

impl QueueClient {
    fn from_replies(replies: Vec<Result<String, TauAiError>>) -> Self {
        Self {
            replies: Arc::new(AsyncMutex::new(replies.into_iter().collect())),
        }
    }
}

#[async_trait]
impl LlmClient for QueueClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        let mut replies = self.replies.lock().await;
        let Some(next) = replies.pop_front() else {
            return Err(TauAiError::InvalidResponse(
                "queue client exhausted replies".to_string(),
            ));
        };
        let text = next?;
        Ok(ChatResponse {
            message: Message::assistant_text(text),
            finish_reason: Some("stop".to_string()),
            usage: ChatUsage::default(),
        })
    }
}

fn write_memory(session_root: &Path, memory_id: &str, summary: &str, importance: f32) {
    let store = tau_memory::runtime::FileMemoryStore::new(session_root);
    let scope = MemoryScope {
        workspace_id: "workspace-cortex".to_string(),
        channel_id: session_root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("channel")
            .to_string(),
        actor_id: "assistant".to_string(),
    };
    let entry = MemoryEntry {
        memory_id: memory_id.to_string(),
        summary: summary.to_string(),
        tags: vec!["cortex".to_string()],
        facts: vec!["fact".to_string()],
        source_event_key: format!("source-{memory_id}"),
        recency_weight_bps: 0,
        confidence_bps: 1_000,
    };
    store
        .write_entry_with_metadata(&scope, entry, None, Some(importance))
        .expect("write memory");
}

#[tokio::test]
async fn integration_spec_2717_c01_c02_cortex_refresh_scans_cross_session_memory_and_uses_llm_bulletin(
) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("openresponses/memory-store");
    let alpha = root.join("alpha");
    let beta = root.join("beta");
    std::fs::create_dir_all(&alpha).expect("create alpha");
    std::fs::create_dir_all(&beta).expect("create beta");

    write_memory(&alpha, "alpha-1", "alpha summary", 0.9);
    write_memory(&beta, "beta-1", "beta summary", 0.7);

    let cortex = Cortex::new(CortexConfig::new(root.clone()));
    let client =
        QueueClient::from_replies(vec![Ok("Bulletin: prioritize beta then alpha".to_string())]);
    let report = cortex.refresh_once(&client, "openai/gpt-4o-mini").await;

    assert_eq!(report.sessions_scanned, 2);
    assert!(report.records_scanned >= 2);
    assert_eq!(report.reason_code, "cortex_bulletin_llm_applied");
    let bulletin = cortex.bulletin_snapshot();
    assert!(bulletin.contains("Bulletin: prioritize beta then alpha"));
}

#[tokio::test]
async fn regression_spec_2717_c03_cortex_refresh_uses_deterministic_fallback_when_llm_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("openresponses/memory-store");
    let alpha = root.join("alpha");
    std::fs::create_dir_all(&alpha).expect("create alpha");
    write_memory(&alpha, "alpha-1", "alpha fallback summary", 0.8);

    let cortex = Cortex::new(CortexConfig::new(root.clone()));
    let client = QueueClient::from_replies(vec![Err(TauAiError::InvalidResponse(
        "forced llm failure".to_string(),
    ))]);
    let report = cortex.refresh_once(&client, "openai/gpt-4o-mini").await;

    assert_eq!(report.reason_code, "cortex_bulletin_llm_error_fallback");
    let bulletin = cortex.bulletin_snapshot();
    assert!(bulletin.contains("## Cortex Memory Bulletin"));
    assert!(bulletin.contains("alpha fallback summary"));
}
