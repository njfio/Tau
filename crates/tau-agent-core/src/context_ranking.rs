//! Semantic message importance scoring for context compaction.
//!
//! Assigns a [0.0, 1.0] importance score to each message, allowing compaction
//! logic to preferentially drop low-value messages instead of oldest-first.

use tau_ai::{Message, MessageRole};

/// Importance score with a classified reason.
#[derive(Debug, Clone, Copy)]
pub struct MessageImportance {
    pub score: f64,
    pub reason: ImportanceReason,
}

/// Classification of why a message received its importance score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportanceReason {
    /// System messages are always retained.
    SystemMessage,
    /// Recent messages receive a recency bonus.
    RecentTurn,
    /// Contains decision-related keywords.
    ContainsDecision,
    /// Contains error information.
    ContainsError,
    /// Tool result output — often bulky.
    ToolResult,
    /// Low information density conversational content.
    ConversationalFiller,
}

/// Score a single message for compaction priority.
pub fn score_message_importance(
    message: &Message,
    position: usize,
    total_messages: usize,
) -> MessageImportance {
    // System messages are always critical
    if message.role == MessageRole::System {
        return MessageImportance {
            score: 1.0,
            reason: ImportanceReason::SystemMessage,
        };
    }

    let mut score = 0.5_f64;
    let mut reason = ImportanceReason::ConversationalFiller;

    // Recency: last 30% of messages get a bonus
    let recency_ratio = if total_messages > 0 {
        position as f64 / total_messages as f64
    } else {
        0.0
    };
    if recency_ratio > 0.7 {
        let bonus = 0.3 * (recency_ratio - 0.7) / 0.3;
        score += bonus;
        reason = ImportanceReason::RecentTurn;
    }

    // Tool results are often bulky — lower base priority
    if message.role == MessageRole::Tool {
        score = 0.3;
        reason = ImportanceReason::ToolResult;
        if message.is_error {
            score = 0.7;
            reason = ImportanceReason::ContainsError;
        }
    }

    // Check for decision keywords
    let text = message.text_content();
    if contains_decision_keywords(&text) {
        score = (score + 0.2).min(1.0);
        if reason == ImportanceReason::ConversationalFiller {
            reason = ImportanceReason::ContainsDecision;
        }
    }

    // Check for error indicators in assistant messages
    if message.role == MessageRole::Assistant && contains_error_keywords(&text) {
        score = (score + 0.15).min(1.0);
        if reason == ImportanceReason::ConversationalFiller {
            reason = ImportanceReason::ContainsError;
        }
    }

    MessageImportance { score, reason }
}

/// Score all messages and return indices sorted by ascending importance.
pub fn rank_messages_by_importance(messages: &[Message]) -> Vec<(usize, MessageImportance)> {
    let total = messages.len();
    let mut scored: Vec<(usize, MessageImportance)> = messages
        .iter()
        .enumerate()
        .map(|(i, msg)| (i, score_message_importance(msg, i, total)))
        .collect();
    scored.sort_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn contains_decision_keywords(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("decided")
        || lower.contains("chose")
        || lower.contains("plan:")
        || lower.contains("strategy")
        || lower.contains("approach:")
        || lower.contains("conclusion")
}

fn contains_error_keywords(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("error:")
        || lower.contains("failed")
        || lower.contains("exception")
        || lower.contains("traceback")
        || lower.contains("panic")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tau_ai::Message;

    #[test]
    fn system_messages_always_critical() {
        let msg = Message::system("You are a helpful assistant.");
        let imp = score_message_importance(&msg, 0, 10);
        assert!((imp.score - 1.0).abs() < f64::EPSILON);
        assert_eq!(imp.reason, ImportanceReason::SystemMessage);
    }

    #[test]
    fn recent_messages_get_bonus() {
        let msg = Message::user("hello");
        let early = score_message_importance(&msg, 1, 10);
        let late = score_message_importance(&msg, 9, 10);
        assert!(late.score > early.score);
    }

    #[test]
    fn decision_keywords_boost_score() {
        let msg = Message::user("I decided to use approach A");
        let imp = score_message_importance(&msg, 5, 10);
        assert!(imp.score > 0.5);
    }

    #[test]
    fn rank_returns_ascending_order() {
        let messages = vec![
            Message::system("system"),
            Message::user("hello"),
            Message::user("I decided to refactor"),
        ];
        let ranked = rank_messages_by_importance(&messages);
        assert!(ranked[0].1.score <= ranked[1].1.score);
        assert!(ranked[1].1.score <= ranked[2].1.score);
    }
}
