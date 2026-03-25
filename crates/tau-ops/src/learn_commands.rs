//! Learn command formatters and types for operator surfaces.
//!
//! Decoupled data types and formatting functions for `/learn-status`,
//! `/learn-clear`, and `/learn-export` commands. The actual data population
//! happens at the call site (TUI/CLI) which has access to runtime stores.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// /learn-status
// ---------------------------------------------------------------------------

/// Aggregate report returned by `/learn-status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearnStatusReport {
    pub total_records: usize,
    pub sessions_tracked: usize,
    pub top_failure_patterns: Vec<FailurePatternSummary>,
    pub tool_success_rates: Vec<ToolRateSummary>,
}

/// A single recurring failure pattern surfaced in a learn-status report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FailurePatternSummary {
    pub tool_name: String,
    pub common_error: String,
    pub occurrence_count: usize,
}

/// Per-tool success rate entry in a learn-status report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolRateSummary {
    pub tool_name: String,
    pub success_rate: f64,
    pub total_executions: usize,
}

/// Format a [`LearnStatusReport`] as a human-readable table.
pub fn format_learn_status(report: &LearnStatusReport) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "learn-status: total_records={} sessions_tracked={}",
        report.total_records, report.sessions_tracked,
    ));

    if report.top_failure_patterns.is_empty() {
        lines.push("learn-status failure-patterns: (none)".to_string());
    } else {
        lines.push("learn-status failure-patterns:".to_string());
        lines.push(format!(
            "  {:<24} {:<8} {}",
            "TOOL", "COUNT", "ERROR"
        ));
        for pattern in &report.top_failure_patterns {
            lines.push(format!(
                "  {:<24} {:<8} {}",
                pattern.tool_name, pattern.occurrence_count, pattern.common_error,
            ));
        }
    }

    if report.tool_success_rates.is_empty() {
        lines.push("learn-status tool-rates: (none)".to_string());
    } else {
        lines.push("learn-status tool-rates:".to_string());
        lines.push(format!(
            "  {:<24} {:<10} {}",
            "TOOL", "RATE", "EXECUTIONS"
        ));
        for rate in &report.tool_success_rates {
            lines.push(format!(
                "  {:<24} {:<10} {}",
                rate.tool_name,
                format!("{:.1}%", rate.success_rate * 100.0),
                rate.total_executions,
            ));
        }
    }

    lines.join("\n")
}

/// Format a [`LearnStatusReport`] as pretty-printed JSON.
pub fn format_learn_status_json(report: &LearnStatusReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// /learn-clear
// ---------------------------------------------------------------------------

/// Result of a `/learn-clear` operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearnClearResult {
    pub records_cleared: usize,
    pub file_path: Option<String>,
}

/// Format a [`LearnClearResult`] as human-readable text.
pub fn format_learn_clear(result: &LearnClearResult) -> String {
    format!("Cleared {} action history records.", result.records_cleared)
}

// ---------------------------------------------------------------------------
// /learn-export
// ---------------------------------------------------------------------------

/// Result of a `/learn-export` operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearnExportResult {
    pub records_exported: usize,
    pub export_path: String,
}

/// Format a [`LearnExportResult`] as human-readable text.
pub fn format_learn_export(result: &LearnExportResult) -> String {
    format!(
        "Exported {} records to {}",
        result.records_exported, result.export_path
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> LearnStatusReport {
        LearnStatusReport {
            total_records: 150,
            sessions_tracked: 12,
            top_failure_patterns: vec![
                FailurePatternSummary {
                    tool_name: "Bash".to_string(),
                    common_error: "command not found".to_string(),
                    occurrence_count: 7,
                },
                FailurePatternSummary {
                    tool_name: "Read".to_string(),
                    common_error: "file not found".to_string(),
                    occurrence_count: 3,
                },
            ],
            tool_success_rates: vec![
                ToolRateSummary {
                    tool_name: "Bash".to_string(),
                    success_rate: 0.85,
                    total_executions: 40,
                },
                ToolRateSummary {
                    tool_name: "Read".to_string(),
                    success_rate: 0.95,
                    total_executions: 60,
                },
            ],
        }
    }

    // -- /learn-status -------------------------------------------------------

    #[test]
    fn unit_format_learn_status_includes_header() {
        let report = sample_report();
        let text = format_learn_status(&report);
        assert!(text.contains("learn-status: total_records=150 sessions_tracked=12"));
    }

    #[test]
    fn unit_format_learn_status_includes_failure_patterns() {
        let report = sample_report();
        let text = format_learn_status(&report);
        assert!(text.contains("Bash"));
        assert!(text.contains("command not found"));
        assert!(text.contains("7"));
        assert!(text.contains("Read"));
        assert!(text.contains("file not found"));
        assert!(text.contains("3"));
    }

    #[test]
    fn unit_format_learn_status_includes_tool_rates() {
        let report = sample_report();
        let text = format_learn_status(&report);
        assert!(text.contains("85.0%"));
        assert!(text.contains("95.0%"));
        assert!(text.contains("40"));
        assert!(text.contains("60"));
    }

    #[test]
    fn unit_format_learn_status_empty_report() {
        let report = LearnStatusReport {
            total_records: 0,
            sessions_tracked: 0,
            top_failure_patterns: vec![],
            tool_success_rates: vec![],
        };
        let text = format_learn_status(&report);
        assert!(text.contains("total_records=0"));
        assert!(text.contains("failure-patterns: (none)"));
        assert!(text.contains("tool-rates: (none)"));
    }

    #[test]
    fn unit_format_learn_status_json_roundtrip() {
        let report = sample_report();
        let json = format_learn_status_json(&report);
        let deserialized: LearnStatusReport =
            serde_json::from_str(&json).expect("roundtrip deserialize");
        assert_eq!(deserialized, report);
    }

    #[test]
    fn unit_format_learn_status_json_contains_fields() {
        let report = sample_report();
        let json = format_learn_status_json(&report);
        assert!(json.contains("\"total_records\": 150"));
        assert!(json.contains("\"sessions_tracked\": 12"));
        assert!(json.contains("\"tool_name\": \"Bash\""));
        assert!(json.contains("\"success_rate\": 0.85"));
    }

    // -- /learn-clear --------------------------------------------------------

    #[test]
    fn unit_format_learn_clear_message() {
        let result = LearnClearResult {
            records_cleared: 42,
            file_path: Some("/tmp/history.jsonl".to_string()),
        };
        assert_eq!(
            format_learn_clear(&result),
            "Cleared 42 action history records."
        );
    }

    #[test]
    fn unit_format_learn_clear_zero_records() {
        let result = LearnClearResult {
            records_cleared: 0,
            file_path: None,
        };
        assert_eq!(
            format_learn_clear(&result),
            "Cleared 0 action history records."
        );
    }

    #[test]
    fn unit_learn_clear_result_json_roundtrip() {
        let result = LearnClearResult {
            records_cleared: 10,
            file_path: Some("/data/history.jsonl".to_string()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: LearnClearResult =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, result);
    }

    // -- /learn-export -------------------------------------------------------

    #[test]
    fn unit_format_learn_export_message() {
        let result = LearnExportResult {
            records_exported: 100,
            export_path: "/tmp/export.jsonl".to_string(),
        };
        assert_eq!(
            format_learn_export(&result),
            "Exported 100 records to /tmp/export.jsonl"
        );
    }

    #[test]
    fn unit_format_learn_export_zero_records() {
        let result = LearnExportResult {
            records_exported: 0,
            export_path: "/dev/null".to_string(),
        };
        assert_eq!(
            format_learn_export(&result),
            "Exported 0 records to /dev/null"
        );
    }

    #[test]
    fn unit_learn_export_result_json_roundtrip() {
        let result = LearnExportResult {
            records_exported: 55,
            export_path: "/out/data.jsonl".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: LearnExportResult =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, result);
    }
}
