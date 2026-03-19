//! Failure detection framework for identifying agent execution problems.
//!
//! Monitors turn history and metrics to detect repeated failures, conversation
//! loops, context exhaustion, and stalled progress.

use std::collections::HashSet;

use tau_ai::Message;

use crate::metrics::AgentMetrics;

/// Signals that indicate the agent is in a failure state.
#[derive(Debug, Clone)]
pub enum FailureSignal {
    /// Tool keeps failing with same error.
    RepeatedToolFailure {
        tool: String,
        count: usize,
        error: String,
    },
    /// Agent is looping (similar messages repeated).
    ConversationLoop { similarity: f64, loop_length: usize },
    /// Context exhaustion imminent.
    ContextExhaustion { utilization: f64 },
    /// No progress (N turns without meaningful tool output).
    NoProgress { turns_without_progress: usize },
    /// Budget approaching limit.
    BudgetExhaustion { utilization: f64 },
}

/// Configuration for failure detection thresholds.
#[derive(Debug, Clone)]
pub struct FailureDetectorConfig {
    pub repeated_failure_threshold: usize,
    pub loop_similarity_threshold: f64,
    pub no_progress_turn_limit: usize,
    pub budget_warning_threshold: f64,
}

impl Default for FailureDetectorConfig {
    fn default() -> Self {
        Self {
            repeated_failure_threshold: 3,
            loop_similarity_threshold: 0.85,
            no_progress_turn_limit: 3,
            budget_warning_threshold: 0.9,
        }
    }
}

/// Detects failure conditions in agent execution.
pub struct FailureDetector {
    config: FailureDetectorConfig,
}

impl FailureDetector {
    pub fn new(config: FailureDetectorConfig) -> Self {
        Self { config }
    }

    /// Analyze recent turn history for failure signals.
    pub fn detect(
        &self,
        history: &[Message],
        metrics: &AgentMetrics,
        budget_utilization: Option<f64>,
    ) -> Vec<FailureSignal> {
        let mut signals = Vec::new();

        if let Some(signal) = self.detect_repeated_tool_failures(metrics) {
            signals.push(signal);
        }

        if let Some(signal) = self.detect_loops(history) {
            signals.push(signal);
        }

        if let Some(signal) = self.detect_no_progress(history) {
            signals.push(signal);
        }

        if let Some(utilization) = budget_utilization {
            if utilization >= self.config.budget_warning_threshold {
                signals.push(FailureSignal::BudgetExhaustion { utilization });
            }
        }

        signals
    }

    fn detect_repeated_tool_failures(&self, metrics: &AgentMetrics) -> Option<FailureSignal> {
        let tool_health = metrics.tool_health.lock().ok()?;
        for (name, stats) in tool_health.iter() {
            if stats.failures >= self.config.repeated_failure_threshold as u64 {
                return Some(FailureSignal::RepeatedToolFailure {
                    tool: name.clone(),
                    count: stats.failures as usize,
                    error: format!(
                        "{} failures out of {} executions",
                        stats.failures, stats.executions
                    ),
                });
            }
        }
        None
    }

    fn detect_loops(&self, messages: &[Message]) -> Option<FailureSignal> {
        let assistant_texts: Vec<String> = messages
            .iter()
            .filter(|m| m.role == tau_ai::MessageRole::Assistant)
            .map(|m| m.text_content())
            .collect();

        if assistant_texts.len() < 3 {
            return None;
        }

        let last_n = &assistant_texts[assistant_texts.len().saturating_sub(4)..];
        for window in last_n.windows(2) {
            let sim = jaccard_similarity(&window[0], &window[1]);
            if sim >= self.config.loop_similarity_threshold {
                return Some(FailureSignal::ConversationLoop {
                    similarity: sim,
                    loop_length: 2,
                });
            }
        }

        None
    }

    fn detect_no_progress(&self, messages: &[Message]) -> Option<FailureSignal> {
        let recent = messages
            .iter()
            .rev()
            .take(self.config.no_progress_turn_limit * 3); // ~3 messages per turn

        let has_tool_output = recent
            .filter(|m| m.role == tau_ai::MessageRole::Tool && !m.is_error)
            .count()
            > 0;

        if !has_tool_output && messages.len() > self.config.no_progress_turn_limit * 3 {
            return Some(FailureSignal::NoProgress {
                turns_without_progress: self.config.no_progress_turn_limit,
            });
        }

        None
    }
}

/// Simple Jaccard similarity on word sets.
fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<&str> = a.split_whitespace().collect();
    let set_b: HashSet<&str> = b.split_whitespace().collect();

    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_identical_strings() {
        assert!((jaccard_similarity("hello world", "hello world") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_disjoint_strings() {
        assert!((jaccard_similarity("hello world", "foo bar")).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let sim = jaccard_similarity("hello world foo", "hello world bar");
        assert!(sim > 0.4 && sim < 0.8);
    }

    #[test]
    fn empty_strings_similarity() {
        assert!((jaccard_similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn detector_no_signals_on_empty() {
        let detector = FailureDetector::new(FailureDetectorConfig::default());
        let metrics = AgentMetrics::default();
        let signals = detector.detect(&[], &metrics, None);
        assert!(signals.is_empty());
    }

    #[test]
    fn detector_budget_exhaustion() {
        let detector = FailureDetector::new(FailureDetectorConfig::default());
        let metrics = AgentMetrics::default();
        let signals = detector.detect(&[], &metrics, Some(0.95));
        assert_eq!(signals.len(), 1);
        assert!(matches!(
            signals[0],
            FailureSignal::BudgetExhaustion { utilization } if utilization >= 0.9
        ));
    }
}
