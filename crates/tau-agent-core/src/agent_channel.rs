//! Inter-agent message bus for branch worker communication.
//!
//! Provides a shared broadcast channel that branch workers can use to share
//! discoveries, coordinate work, and report partial results.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

/// Message types for inter-agent communication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentMessageType {
    /// Share a discovery (file found, error identified).
    Discovery,
    /// Signal completion of a sub-task.
    StepCompleted,
    /// Request help or coordination.
    CoordinationRequest,
    /// Share partial results.
    PartialResult,
}

/// A message exchanged between agents within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub message_type: AgentMessageType,
    pub payload: Value,
    pub timestamp_ms: u64,
}

impl AgentMessage {
    /// Create a new broadcast message from the given agent.
    pub fn broadcast(
        from_agent_id: impl Into<String>,
        message_type: AgentMessageType,
        payload: Value,
    ) -> Self {
        Self {
            from_agent_id: from_agent_id.into(),
            to_agent_id: None,
            message_type,
            payload,
            timestamp_ms: current_time_ms(),
        }
    }

    /// Create a directed message to a specific agent.
    pub fn directed(
        from_agent_id: impl Into<String>,
        to_agent_id: impl Into<String>,
        message_type: AgentMessageType,
        payload: Value,
    ) -> Self {
        Self {
            from_agent_id: from_agent_id.into(),
            to_agent_id: Some(to_agent_id.into()),
            message_type,
            payload,
            timestamp_ms: current_time_ms(),
        }
    }
}

/// Shared message bus for inter-agent communication within a session.
pub struct AgentMessageBus {
    sender: broadcast::Sender<AgentMessage>,
}

impl AgentMessageBus {
    /// Create a new message bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    /// Subscribe to the message bus. Returns a receiver for incoming messages.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.sender.subscribe()
    }

    /// Send a message on the bus. Returns Ok(()) on success.
    pub fn send(&self, message: AgentMessage) -> Result<(), AgentMessage> {
        self.sender.send(message).map(|_| ()).map_err(|e| e.0)
    }

    /// Returns the number of active subscribers.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn broadcast_message_has_no_target() {
        let msg = AgentMessage::broadcast(
            "worker-1",
            AgentMessageType::Discovery,
            json!({"file": "main.rs"}),
        );
        assert!(msg.to_agent_id.is_none());
        assert_eq!(msg.from_agent_id, "worker-1");
    }

    #[test]
    fn directed_message_has_target() {
        let msg = AgentMessage::directed(
            "worker-1",
            "worker-2",
            AgentMessageType::CoordinationRequest,
            json!({"need": "help"}),
        );
        assert_eq!(msg.to_agent_id.as_deref(), Some("worker-2"));
    }

    #[tokio::test]
    async fn bus_send_receive() {
        let bus = AgentMessageBus::new(16);
        let mut rx = bus.subscribe();
        let msg = AgentMessage::broadcast(
            "test",
            AgentMessageType::StepCompleted,
            json!({"step": 1}),
        );
        bus.send(msg).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.from_agent_id, "test");
        assert_eq!(received.message_type, AgentMessageType::StepCompleted);
    }
}
