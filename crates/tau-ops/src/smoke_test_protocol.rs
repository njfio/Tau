//! End-to-end self-improvement smoke test protocol.
//!
//! Defines a structured protocol for verifying that all stages of the
//! self-improvement loop are functioning correctly: from action-history
//! recording through APO prompt optimization and next-session application.

use serde::{Deserialize, Serialize};

/// A complete smoke-test protocol consisting of ordered verification steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfImprovementSmokeTestProtocol {
    pub steps: Vec<SmokeTestStep>,
}

/// A single step in the smoke-test protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeTestStep {
    /// Short human-readable name for this step.
    pub name: String,
    /// What this step verifies.
    pub description: String,
    /// How to confirm the step passed.
    pub verification: String,
    /// Whether this step can be verified automatically (true) or requires
    /// manual inspection (false).
    pub automated: bool,
}

/// Return the default 6-step smoke-test protocol that exercises the full
/// self-improvement loop end-to-end.
pub fn default_smoke_test_protocol() -> SelfImprovementSmokeTestProtocol {
    SelfImprovementSmokeTestProtocol {
        steps: vec![
            SmokeTestStep {
                name: "action_history_records_failures".to_string(),
                description: "Action history records all failures from the current session."
                    .to_string(),
                verification:
                    "Query ActionHistoryStore for recent entries and confirm failure records exist."
                        .to_string(),
                automated: true,
            },
            SmokeTestStep {
                name: "failure_patterns_in_cortex_bulletin".to_string(),
                description: "Failure patterns surface in the Cortex bulletin for the next session."
                    .to_string(),
                verification:
                    "Generate a Cortex bulletin and verify it contains failure-pattern insights."
                        .to_string(),
                automated: true,
            },
            SmokeTestStep {
                name: "recovery_strategies_adapt".to_string(),
                description: "Recovery strategies adapt based on observed failure patterns."
                    .to_string(),
                verification:
                    "Compare recovery strategy weights before and after failure injection; confirm delta > 0."
                        .to_string(),
                automated: true,
            },
            SmokeTestStep {
                name: "rollouts_accumulate_with_rewards".to_string(),
                description: "Rollouts accumulate in the training store with reward scores."
                    .to_string(),
                verification:
                    "Query the training store for rollouts and confirm each has a non-null reward."
                        .to_string(),
                automated: true,
            },
            SmokeTestStep {
                name: "apo_triggers_new_prompt".to_string(),
                description:
                    "APO triggers at threshold and produces an optimized system prompt."
                        .to_string(),
                verification:
                    "Trigger APO with sufficient rollouts and confirm the output prompt differs from the input."
                        .to_string(),
                automated: true,
            },
            SmokeTestStep {
                name: "next_session_uses_improved_prompt".to_string(),
                description: "The next session bootstraps with the APO-optimized prompt."
                    .to_string(),
                verification:
                    "Start a new session and inspect the system prompt to confirm it matches the APO output."
                        .to_string(),
                automated: false,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_protocol_has_six_steps() {
        let protocol = default_smoke_test_protocol();
        assert_eq!(protocol.steps.len(), 6);
    }

    #[test]
    fn all_steps_have_non_empty_fields() {
        let protocol = default_smoke_test_protocol();
        for step in &protocol.steps {
            assert!(!step.name.is_empty(), "step name must not be empty");
            assert!(
                !step.description.is_empty(),
                "step description must not be empty"
            );
            assert!(
                !step.verification.is_empty(),
                "step verification must not be empty"
            );
        }
    }

    #[test]
    fn first_step_is_action_history() {
        let protocol = default_smoke_test_protocol();
        assert_eq!(protocol.steps[0].name, "action_history_records_failures");
    }

    #[test]
    fn last_step_is_next_session() {
        let protocol = default_smoke_test_protocol();
        assert_eq!(
            protocol.steps[5].name,
            "next_session_uses_improved_prompt"
        );
    }

    #[test]
    fn last_step_is_not_automated() {
        let protocol = default_smoke_test_protocol();
        assert!(
            !protocol.steps[5].automated,
            "next-session step requires manual verification"
        );
    }

    #[test]
    fn first_five_steps_are_automated() {
        let protocol = default_smoke_test_protocol();
        for step in &protocol.steps[..5] {
            assert!(
                step.automated,
                "step '{}' should be automated",
                step.name
            );
        }
    }

    #[test]
    fn protocol_round_trips_through_json() {
        let protocol = default_smoke_test_protocol();
        let json = serde_json::to_string(&protocol).expect("serialize");
        let deserialized: SelfImprovementSmokeTestProtocol =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.steps.len(), protocol.steps.len());
        for (a, b) in deserialized.steps.iter().zip(protocol.steps.iter()) {
            assert_eq!(a.name, b.name);
        }
    }
}
