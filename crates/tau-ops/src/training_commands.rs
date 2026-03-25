//! Training command formatters and types for operator surfaces.
//!
//! Decoupled data types and formatting functions for `/training-status`.
//! The actual data population happens at the call site (TUI/CLI) which has
//! access to runtime stores and RL configuration.

use serde::{Deserialize, Serialize};

/// Aggregate report returned by `/training-status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingStatusReport {
    pub live_rl_enabled: bool,
    pub total_rollouts: usize,
    pub last_reward_score: Option<f64>,
    pub apo_threshold: usize,
    pub apo_runs_completed: usize,
    pub current_prompt_version: Option<String>,
}

/// Format a [`TrainingStatusReport`] as human-readable output showing
/// progress toward the APO trigger threshold.
pub fn format_training_status(report: &TrainingStatusReport) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "training-status: live_rl={} rollouts={} apo_runs={}/{}",
        if report.live_rl_enabled {
            "enabled"
        } else {
            "disabled"
        },
        report.total_rollouts,
        report.apo_runs_completed,
        report.apo_threshold,
    ));

    lines.push(format!(
        "training-status: last_reward={}",
        report
            .last_reward_score
            .map(|score| format!("{score:.4}"))
            .unwrap_or_else(|| "none".to_string()),
    ));

    lines.push(format!(
        "training-status: prompt_version={}",
        report
            .current_prompt_version
            .as_deref()
            .unwrap_or("none"),
    ));

    let remaining = report.apo_threshold.saturating_sub(report.apo_runs_completed);
    if remaining == 0 && report.apo_threshold > 0 {
        lines.push("training-status: apo_trigger=READY".to_string());
    } else if report.apo_threshold > 0 {
        lines.push(format!(
            "training-status: apo_trigger={} rollouts remaining",
            remaining,
        ));
    } else {
        lines.push("training-status: apo_trigger=disabled (threshold=0)".to_string());
    }

    lines.join("\n")
}

/// Format a [`TrainingStatusReport`] as pretty-printed JSON.
pub fn format_training_status_json(report: &TrainingStatusReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> TrainingStatusReport {
        TrainingStatusReport {
            live_rl_enabled: true,
            total_rollouts: 250,
            last_reward_score: Some(0.8723),
            apo_threshold: 500,
            apo_runs_completed: 120,
            current_prompt_version: Some("v2.3.1".to_string()),
        }
    }

    #[test]
    fn unit_format_training_status_header() {
        let report = sample_report();
        let text = format_training_status(&report);
        assert!(text.contains("training-status: live_rl=enabled rollouts=250 apo_runs=120/500"));
    }

    #[test]
    fn unit_format_training_status_reward() {
        let report = sample_report();
        let text = format_training_status(&report);
        assert!(text.contains("last_reward=0.8723"));
    }

    #[test]
    fn unit_format_training_status_prompt_version() {
        let report = sample_report();
        let text = format_training_status(&report);
        assert!(text.contains("prompt_version=v2.3.1"));
    }

    #[test]
    fn unit_format_training_status_apo_remaining() {
        let report = sample_report();
        let text = format_training_status(&report);
        assert!(text.contains("apo_trigger=380 rollouts remaining"));
    }

    #[test]
    fn unit_format_training_status_apo_ready() {
        let report = TrainingStatusReport {
            live_rl_enabled: true,
            total_rollouts: 500,
            last_reward_score: Some(0.91),
            apo_threshold: 500,
            apo_runs_completed: 500,
            current_prompt_version: Some("v3.0.0".to_string()),
        };
        let text = format_training_status(&report);
        assert!(text.contains("apo_trigger=READY"));
    }

    #[test]
    fn unit_format_training_status_apo_disabled() {
        let report = TrainingStatusReport {
            live_rl_enabled: false,
            total_rollouts: 0,
            last_reward_score: None,
            apo_threshold: 0,
            apo_runs_completed: 0,
            current_prompt_version: None,
        };
        let text = format_training_status(&report);
        assert!(text.contains("live_rl=disabled"));
        assert!(text.contains("last_reward=none"));
        assert!(text.contains("prompt_version=none"));
        assert!(text.contains("apo_trigger=disabled (threshold=0)"));
    }

    #[test]
    fn unit_format_training_status_json_roundtrip() {
        let report = sample_report();
        let json = format_training_status_json(&report);
        let deserialized: TrainingStatusReport =
            serde_json::from_str(&json).expect("roundtrip deserialize");
        assert_eq!(deserialized, report);
    }

    #[test]
    fn unit_format_training_status_json_contains_fields() {
        let report = sample_report();
        let json = format_training_status_json(&report);
        assert!(json.contains("\"live_rl_enabled\": true"));
        assert!(json.contains("\"total_rollouts\": 250"));
        assert!(json.contains("\"apo_threshold\": 500"));
        assert!(json.contains("\"current_prompt_version\": \"v2.3.1\""));
    }

    #[test]
    fn unit_format_training_status_json_none_reward() {
        let report = TrainingStatusReport {
            live_rl_enabled: false,
            total_rollouts: 0,
            last_reward_score: None,
            apo_threshold: 100,
            apo_runs_completed: 0,
            current_prompt_version: None,
        };
        let json = format_training_status_json(&report);
        assert!(json.contains("\"last_reward_score\": null"));
        assert!(json.contains("\"current_prompt_version\": null"));
    }
}
