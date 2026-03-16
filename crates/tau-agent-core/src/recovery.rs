//! Recovery strategies for detected failure conditions.
//!
//! Maps failure signals to appropriate recovery actions, escalating through
//! progressively stronger interventions.

use crate::failure_detector::FailureSignal;

/// Available recovery strategies, ordered from least to most disruptive.
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with a hint injected into context.
    RetryWithHint { hint: String },
    /// Abandon current path and try alternative.
    AlternativeApproach { new_prompt: String },
    /// Compact context aggressively and retry.
    CompactAndRetry,
    /// Escalate to user for guidance.
    EscalateToUser { message: String },
    /// Terminate gracefully with partial results.
    GracefulTermination { summary: String },
}

/// Select the best recovery strategy for a given failure signal and attempt count.
pub fn select_recovery_strategy(signal: &FailureSignal, attempt: usize) -> RecoveryStrategy {
    match signal {
        FailureSignal::RepeatedToolFailure { tool, error, .. } => {
            if attempt == 0 {
                RecoveryStrategy::RetryWithHint {
                    hint: format!(
                        "Tool '{}' failed with: {}. Try an alternative approach.",
                        tool, error
                    ),
                }
            } else {
                RecoveryStrategy::EscalateToUser {
                    message: format!(
                        "Tool '{}' is repeatedly failing. Need guidance.",
                        tool
                    ),
                }
            }
        }
        FailureSignal::ConversationLoop { .. } => RecoveryStrategy::AlternativeApproach {
            new_prompt: "You appear to be repeating the same actions. Step back, reassess the problem, and try a fundamentally different approach.".to_string(),
        },
        FailureSignal::ContextExhaustion { .. } => RecoveryStrategy::CompactAndRetry,
        FailureSignal::NoProgress { .. } => {
            if attempt == 0 {
                RecoveryStrategy::RetryWithHint {
                    hint: "No progress detected. Break the problem into smaller steps and verify each step.".to_string(),
                }
            } else {
                RecoveryStrategy::EscalateToUser {
                    message: "Unable to make progress on this task. Would you like to provide more guidance?".to_string(),
                }
            }
        }
        FailureSignal::BudgetExhaustion { .. } => RecoveryStrategy::GracefulTermination {
            summary: "Approaching budget limit. Here is what has been accomplished so far."
                .to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_tool_failure_first_attempt_gives_hint() {
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "bash".to_string(),
            count: 3,
            error: "permission denied".to_string(),
        };
        let strategy = select_recovery_strategy(&signal, 0);
        assert!(matches!(strategy, RecoveryStrategy::RetryWithHint { .. }));
    }

    #[test]
    fn repeated_tool_failure_second_attempt_escalates() {
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "bash".to_string(),
            count: 5,
            error: "permission denied".to_string(),
        };
        let strategy = select_recovery_strategy(&signal, 1);
        assert!(matches!(strategy, RecoveryStrategy::EscalateToUser { .. }));
    }

    #[test]
    fn conversation_loop_gives_alternative() {
        let signal = FailureSignal::ConversationLoop {
            similarity: 0.9,
            loop_length: 2,
        };
        let strategy = select_recovery_strategy(&signal, 0);
        assert!(matches!(
            strategy,
            RecoveryStrategy::AlternativeApproach { .. }
        ));
    }

    #[test]
    fn context_exhaustion_compacts() {
        let signal = FailureSignal::ContextExhaustion { utilization: 0.95 };
        let strategy = select_recovery_strategy(&signal, 0);
        assert!(matches!(strategy, RecoveryStrategy::CompactAndRetry));
    }

    #[test]
    fn budget_exhaustion_terminates() {
        let signal = FailureSignal::BudgetExhaustion { utilization: 0.95 };
        let strategy = select_recovery_strategy(&signal, 0);
        assert!(matches!(
            strategy,
            RecoveryStrategy::GracefulTermination { .. }
        ));
    }
}
