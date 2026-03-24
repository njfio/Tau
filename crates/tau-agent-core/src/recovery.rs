//! Recovery strategies for detected failure conditions.
//!
//! Maps failure signals to appropriate recovery actions, escalating through
//! progressively stronger interventions. When historical tool insights are
//! available, strategy selection is refined based on failure patterns and
//! tool success rates from past sessions.

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

/// Pre-computed insight about a tool derived from action history.
///
/// Used to decouple recovery strategy selection from a direct dependency
/// on the `ActionHistoryStore`, allowing callers to query history once
/// and pass the relevant insight in.
#[derive(Debug, Clone)]
pub struct HistoricalToolInsight {
    /// Name of the tool this insight describes.
    pub tool_name: String,
    /// Success rate in [0.0, 1.0] over recent history.
    pub success_rate: f64,
    /// Most common error message seen for this tool, if any.
    pub common_error: Option<String>,
    /// Total number of recorded failures for this tool.
    pub failure_count: usize,
}

/// Threshold below which a tool's success rate is considered critically low.
const LOW_SUCCESS_RATE_THRESHOLD: f64 = 0.3;

/// Minimum failure occurrences before a tool is considered chronically failing.
const CHRONIC_FAILURE_THRESHOLD: usize = 3;

/// Select the best recovery strategy for a given failure signal and attempt count.
///
/// This is the backward-compatible entry point that uses no historical data.
pub fn select_recovery_strategy(signal: &FailureSignal, attempt: usize) -> RecoveryStrategy {
    select_recovery_strategy_with_history(signal, attempt, None)
}

/// Select the best recovery strategy, optionally informed by historical tool data.
///
/// When `tool_insight` is provided and matches the failing tool, the strategy
/// may be overridden:
///
/// - If the tool has `failure_count >= 3` (chronic failures), prefer
///   `AlternativeApproach` regardless of attempt count.
/// - If the tool has `success_rate < 0.3` (30%), prefer `RetryWithHint`
///   with the common error pattern included in the hint.
///
/// If no insight is provided or the insight does not trigger an override,
/// the existing rule-based logic applies as a fallback.
pub fn select_recovery_strategy_with_history(
    signal: &FailureSignal,
    attempt: usize,
    tool_insight: Option<&HistoricalToolInsight>,
) -> RecoveryStrategy {
    // Extract the tool name from the signal, if this is a tool-related failure.
    let failing_tool = match signal {
        FailureSignal::RepeatedToolFailure { tool, .. } => Some(tool.as_str()),
        _ => None,
    };

    // If we have historical insight for the failing tool, apply history-informed overrides.
    if let (Some(failing), Some(insight)) = (failing_tool, tool_insight) {
        if insight.tool_name == failing {
            // Chronic failures: this tool keeps failing across sessions — try something else.
            if insight.failure_count >= CHRONIC_FAILURE_THRESHOLD {
                return RecoveryStrategy::AlternativeApproach {
                    new_prompt: format!(
                        "Tool '{}' has failed {} times historically. Use a different tool or approach.",
                        failing, insight.failure_count
                    ),
                };
            }

            // Low success rate: this tool rarely succeeds — retry with the known error pattern.
            if insight.success_rate < LOW_SUCCESS_RATE_THRESHOLD {
                let error_context = insight
                    .common_error
                    .as_deref()
                    .unwrap_or("unknown error pattern");
                return RecoveryStrategy::RetryWithHint {
                    hint: format!(
                        "Tool '{}' has a {:.0}% success rate. Common error: {}. Adjust your approach accordingly.",
                        failing,
                        insight.success_rate * 100.0,
                        error_context
                    ),
                };
            }
        }
    }

    // Fallback: original rule-based strategy selection.
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

    // ── Backward-compatibility tests (no history) ──────────────────────

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

    // ── History-informed tests ──────────────────────────────────────────

    #[test]
    fn no_history_strategy_unchanged() {
        // With None insight, select_recovery_strategy_with_history behaves
        // identically to select_recovery_strategy.
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "bash".to_string(),
            count: 3,
            error: "permission denied".to_string(),
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, None);
        assert!(matches!(strategy, RecoveryStrategy::RetryWithHint { .. }));

        let strategy = select_recovery_strategy_with_history(&signal, 1, None);
        assert!(matches!(strategy, RecoveryStrategy::EscalateToUser { .. }));
    }

    #[test]
    fn low_success_rate_selects_retry_with_hint() {
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "flaky_tool".to_string(),
            count: 2,
            error: "timeout".to_string(),
        };
        let insight = HistoricalToolInsight {
            tool_name: "flaky_tool".to_string(),
            success_rate: 0.2, // 20% — below 30% threshold
            common_error: Some("connection reset by peer".to_string()),
            failure_count: 2, // below chronic threshold
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        match strategy {
            RecoveryStrategy::RetryWithHint { ref hint } => {
                assert!(
                    hint.contains("20%"),
                    "hint should contain success rate percentage, got: {}",
                    hint
                );
                assert!(
                    hint.contains("connection reset by peer"),
                    "hint should contain common error, got: {}",
                    hint
                );
            }
            other => panic!(
                "Expected RetryWithHint for low success rate, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn chronic_failure_selects_alternative_approach() {
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "broken_tool".to_string(),
            count: 5,
            error: "segfault".to_string(),
        };
        let insight = HistoricalToolInsight {
            tool_name: "broken_tool".to_string(),
            success_rate: 0.5, // healthy rate, but chronic failures
            common_error: Some("segfault".to_string()),
            failure_count: 5, // >= 3 chronic threshold
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        match strategy {
            RecoveryStrategy::AlternativeApproach { ref new_prompt } => {
                assert!(
                    new_prompt.contains("broken_tool"),
                    "prompt should mention tool name, got: {}",
                    new_prompt
                );
                assert!(
                    new_prompt.contains("5"),
                    "prompt should mention failure count, got: {}",
                    new_prompt
                );
            }
            other => panic!(
                "Expected AlternativeApproach for chronic failures, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn healthy_tool_stats_fall_through_to_normal_strategy() {
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "healthy_tool".to_string(),
            count: 2,
            error: "transient error".to_string(),
        };
        let insight = HistoricalToolInsight {
            tool_name: "healthy_tool".to_string(),
            success_rate: 0.85, // healthy
            common_error: None,
            failure_count: 1, // below chronic threshold
        };

        // attempt 0 → should fall through to RetryWithHint (normal path)
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        match strategy {
            RecoveryStrategy::RetryWithHint { ref hint } => {
                assert!(
                    hint.contains("transient error"),
                    "should use the signal's error, got: {}",
                    hint
                );
            }
            other => panic!(
                "Expected normal RetryWithHint for healthy tool, got: {:?}",
                other
            ),
        }

        // attempt 1 → should fall through to EscalateToUser (normal path)
        let strategy = select_recovery_strategy_with_history(&signal, 1, Some(&insight));
        assert!(
            matches!(strategy, RecoveryStrategy::EscalateToUser { .. }),
            "Expected EscalateToUser on second attempt for healthy tool"
        );
    }

    #[test]
    fn mismatched_tool_insight_falls_through() {
        // Insight is for a different tool than the one failing.
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "tool_a".to_string(),
            count: 3,
            error: "error".to_string(),
        };
        let insight = HistoricalToolInsight {
            tool_name: "tool_b".to_string(),
            success_rate: 0.1,
            common_error: Some("bad".to_string()),
            failure_count: 10,
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        // Should fall through to the normal path since tool names don't match.
        assert!(matches!(strategy, RecoveryStrategy::RetryWithHint { .. }));
    }

    #[test]
    fn non_tool_signal_ignores_insight() {
        // For non-tool signals like ConversationLoop, insight should be ignored.
        let signal = FailureSignal::ConversationLoop {
            similarity: 0.9,
            loop_length: 2,
        };
        let insight = HistoricalToolInsight {
            tool_name: "bash".to_string(),
            success_rate: 0.1,
            common_error: Some("bad".to_string()),
            failure_count: 10,
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        assert!(matches!(
            strategy,
            RecoveryStrategy::AlternativeApproach { .. }
        ));
    }

    #[test]
    fn chronic_failure_takes_priority_over_low_success_rate() {
        // When both conditions are true, chronic failure (>= 3) should win
        // because it's checked first.
        let signal = FailureSignal::RepeatedToolFailure {
            tool: "bad_tool".to_string(),
            count: 5,
            error: "crash".to_string(),
        };
        let insight = HistoricalToolInsight {
            tool_name: "bad_tool".to_string(),
            success_rate: 0.1, // also below threshold
            common_error: Some("crash".to_string()),
            failure_count: 5, // >= 3
        };
        let strategy = select_recovery_strategy_with_history(&signal, 0, Some(&insight));
        assert!(
            matches!(strategy, RecoveryStrategy::AlternativeApproach { .. }),
            "Chronic failure should take priority over low success rate"
        );
    }
}
