//! Self-modification command handlers for proposal tracking, review, and rollback.
//!
//! Provides data structures and formatting functions for the `/self-modify-status`,
//! `/self-modify-review`, and `/self-modify-rollback` operator commands.

use serde::{Deserialize, Serialize};

/// A single self-modification proposal record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelfModificationRecord {
    pub proposal_id: String,
    pub target_path: String,
    /// One of: "skill", "config", "source"
    pub modification_type: String,
    /// One of: "proposed", "approved", "applied", "rejected", "rolled_back"
    pub status: String,
    pub rationale: String,
    pub timestamp: String,
}

/// Aggregated status report for `/self-modify-status`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelfModifyStatusReport {
    pub total_proposals: usize,
    pub applied: usize,
    pub rejected: usize,
    pub pending: usize,
    pub recent: Vec<SelfModificationRecord>,
}

/// A single pending proposal for review via `/self-modify-review`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelfModifyReviewItem {
    pub proposal_id: String,
    pub target_path: String,
    pub diff_preview: String,
    pub rationale: String,
}

/// Result of a `/self-modify-rollback` operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelfModifyRollbackResult {
    pub proposal_id: String,
    pub rolled_back: bool,
    pub revert_commit: Option<String>,
    pub error: Option<String>,
}

/// Format a status report for human-readable display.
pub fn format_self_modify_status(report: &SelfModifyStatusReport) -> String {
    let mut out = String::new();
    out.push_str("Self-Modification Status\n");
    out.push_str(&format!(
        "  Total proposals: {}  Applied: {}  Rejected: {}  Pending: {}\n",
        report.total_proposals, report.applied, report.rejected, report.pending
    ));
    if report.recent.is_empty() {
        out.push_str("  No recent proposals.\n");
    } else {
        out.push_str("\nRecent proposals:\n");
        for r in &report.recent {
            out.push_str(&format!(
                "  [{}] {} ({}) -> {} — {}\n",
                r.proposal_id, r.target_path, r.modification_type, r.status, r.timestamp
            ));
            out.push_str(&format!("    Rationale: {}\n", r.rationale));
        }
    }
    out
}

/// Format review items for human-readable display.
pub fn format_self_modify_review(items: &[SelfModifyReviewItem]) -> String {
    if items.is_empty() {
        return "No pending self-modification proposals to review.\n".to_string();
    }
    let mut out = String::new();
    out.push_str(&format!(
        "Pending Self-Modification Reviews ({})\n\n",
        items.len()
    ));
    for item in items {
        out.push_str(&format!("Proposal: {}\n", item.proposal_id));
        out.push_str(&format!("  Target: {}\n", item.target_path));
        out.push_str(&format!("  Rationale: {}\n", item.rationale));
        out.push_str("  Diff preview:\n");
        for line in item.diff_preview.lines() {
            out.push_str(&format!("    {line}\n"));
        }
        out.push('\n');
    }
    out
}

/// B10: A generated regression test template for self-modification proposals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRegressionTest {
    pub test_name: String,
    pub test_module: String,
    pub failure_pattern: String,
    pub fix_applied: String,
    pub test_code_template: String,
}

/// B10: Generate a regression test template for a self-modification proposal.
///
/// Produces a `GeneratedRegressionTest` containing a Rust test function stub
/// that documents the failure pattern and fix, intended to be inserted into
/// the test suite to prevent regressions.
pub fn generate_regression_test_template(
    proposal_id: &str,
    failure_pattern: &str,
    fix_description: &str,
) -> GeneratedRegressionTest {
    let test_name = format!("regression_{}", proposal_id.replace('-', "_"));
    GeneratedRegressionTest {
        test_name: test_name.clone(),
        test_module: "self_modification_regressions".to_string(),
        failure_pattern: failure_pattern.to_string(),
        fix_applied: fix_description.to_string(),
        test_code_template: format!(
            "#[test]\nfn {}() {{\n    // Regression test for self-modification {}\n    // Failure pattern: {}\n    // Fix: {}\n    todo!(\"Implement regression test\")\n}}\n",
            test_name, proposal_id, failure_pattern, fix_description
        ),
    }
}

/// Format a rollback result for human-readable display.
pub fn format_self_modify_rollback(result: &SelfModifyRollbackResult) -> String {
    let mut out = String::new();
    if result.rolled_back {
        out.push_str(&format!(
            "Proposal {} rolled back successfully.\n",
            result.proposal_id
        ));
        if let Some(ref commit) = result.revert_commit {
            out.push_str(&format!("  Revert commit: {commit}\n"));
        }
    } else {
        out.push_str(&format!(
            "Rollback of proposal {} failed.\n",
            result.proposal_id
        ));
        if let Some(ref err) = result.error {
            out.push_str(&format!("  Error: {err}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> SelfModificationRecord {
        SelfModificationRecord {
            proposal_id: "p-001".to_string(),
            target_path: "skills/demo.md".to_string(),
            modification_type: "skill".to_string(),
            status: "applied".to_string(),
            rationale: "Improve demo skill coverage".to_string(),
            timestamp: "2026-03-25T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn unit_self_modification_record_serialization_round_trip() {
        let record = sample_record();
        let json = serde_json::to_string(&record).expect("serialize");
        let deser: SelfModificationRecord =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(record, deser);
    }

    #[test]
    fn unit_format_self_modify_status_with_records() {
        let report = SelfModifyStatusReport {
            total_proposals: 5,
            applied: 3,
            rejected: 1,
            pending: 1,
            recent: vec![sample_record()],
        };
        let rendered = format_self_modify_status(&report);
        assert!(rendered.contains("Total proposals: 5"));
        assert!(rendered.contains("Applied: 3"));
        assert!(rendered.contains("p-001"));
        assert!(rendered.contains("skills/demo.md"));
    }

    #[test]
    fn unit_format_self_modify_status_empty() {
        let report = SelfModifyStatusReport {
            total_proposals: 0,
            applied: 0,
            rejected: 0,
            pending: 0,
            recent: vec![],
        };
        let rendered = format_self_modify_status(&report);
        assert!(rendered.contains("No recent proposals"));
    }

    #[test]
    fn unit_format_self_modify_review_with_items() {
        let items = vec![SelfModifyReviewItem {
            proposal_id: "p-002".to_string(),
            target_path: "config/tau.toml".to_string(),
            diff_preview: "+new_key = true\n-old_key = false".to_string(),
            rationale: "Enable new feature flag".to_string(),
        }];
        let rendered = format_self_modify_review(&items);
        assert!(rendered.contains("Pending Self-Modification Reviews (1)"));
        assert!(rendered.contains("p-002"));
        assert!(rendered.contains("+new_key = true"));
    }

    #[test]
    fn unit_format_self_modify_review_empty() {
        let rendered = format_self_modify_review(&[]);
        assert!(rendered.contains("No pending"));
    }

    #[test]
    fn unit_format_self_modify_rollback_success() {
        let result = SelfModifyRollbackResult {
            proposal_id: "p-003".to_string(),
            rolled_back: true,
            revert_commit: Some("abc123".to_string()),
            error: None,
        };
        let rendered = format_self_modify_rollback(&result);
        assert!(rendered.contains("rolled back successfully"));
        assert!(rendered.contains("abc123"));
    }

    #[test]
    fn unit_format_self_modify_rollback_failure() {
        let result = SelfModifyRollbackResult {
            proposal_id: "p-004".to_string(),
            rolled_back: false,
            revert_commit: None,
            error: Some("proposal not found".to_string()),
        };
        let rendered = format_self_modify_rollback(&result);
        assert!(rendered.contains("failed"));
        assert!(rendered.contains("proposal not found"));
    }

    // B10: Self-improving test infrastructure

    #[test]
    fn generate_regression_test_template_produces_valid_template() {
        let tmpl = generate_regression_test_template(
            "self-mod-123",
            "bash tool fails on empty input",
            "added input validation guard",
        );
        assert_eq!(tmpl.test_name, "regression_self_mod_123");
        assert_eq!(tmpl.test_module, "self_modification_regressions");
        assert_eq!(tmpl.failure_pattern, "bash tool fails on empty input");
        assert_eq!(tmpl.fix_applied, "added input validation guard");
        assert!(tmpl.test_code_template.contains("#[test]"));
        assert!(tmpl.test_code_template.contains("fn regression_self_mod_123()"));
        assert!(tmpl.test_code_template.contains("self-mod-123"));
        assert!(tmpl.test_code_template.contains("bash tool fails on empty input"));
        assert!(tmpl.test_code_template.contains("added input validation guard"));
        assert!(tmpl.test_code_template.contains("todo!"));
    }

    #[test]
    fn generate_regression_test_template_serializes() {
        let tmpl = generate_regression_test_template("p-42", "crash", "fix");
        let json = serde_json::to_string(&tmpl).expect("serialize");
        let deser: GeneratedRegressionTest =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.test_name, tmpl.test_name);
        assert_eq!(deser.test_code_template, tmpl.test_code_template);
    }
}
