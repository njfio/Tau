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
}
