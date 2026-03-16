//! Plan execution engine with parallel step scheduling.
//!
//! Executes structured plans by running ready steps, handling failures, and
//! optionally revising the plan on step failure.

use serde::{Deserialize, Serialize};

use crate::plan::{PlanStepStatus, StructuredPlan};

/// Report generated after plan execution completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionReport {
    pub plan_id: String,
    pub goal: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
    pub is_complete: bool,
    pub is_deadlocked: bool,
}

/// Errors that can occur during plan execution.
#[derive(Debug, Clone)]
pub enum PlanExecutionError {
    /// No ready steps remain but plan is not complete.
    Deadlock { remaining_step_ids: Vec<String> },
    /// Plan validation failed before execution.
    ValidationFailed { errors: Vec<String> },
}

impl std::fmt::Display for PlanExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanExecutionError::Deadlock {
                remaining_step_ids,
            } => write!(
                f,
                "plan deadlocked with remaining steps: {:?}",
                remaining_step_ids
            ),
            PlanExecutionError::ValidationFailed { errors } => {
                write!(f, "plan validation failed: {:?}", errors)
            }
        }
    }
}

impl std::error::Error for PlanExecutionError {}

/// Build an execution report from the current plan state.
pub fn build_execution_report(plan: &StructuredPlan) -> PlanExecutionReport {
    let completed = plan
        .steps
        .iter()
        .filter(|s| s.status == PlanStepStatus::Completed)
        .count();
    let failed = plan
        .steps
        .iter()
        .filter(|s| matches!(s.status, PlanStepStatus::Failed { .. }))
        .count();
    let skipped = plan
        .steps
        .iter()
        .filter(|s| matches!(s.status, PlanStepStatus::Skipped { .. }))
        .count();
    let is_complete = plan.is_complete();
    let is_deadlocked = !is_complete && plan.ready_steps().is_empty();

    PlanExecutionReport {
        plan_id: plan.id.clone(),
        goal: plan.goal.clone(),
        total_steps: plan.steps.len(),
        completed_steps: completed,
        failed_steps: failed,
        skipped_steps: skipped,
        is_complete,
        is_deadlocked,
    }
}

/// Check if the plan is deadlocked (not complete but no steps can proceed).
pub fn check_deadlock(plan: &StructuredPlan) -> Option<PlanExecutionError> {
    if plan.is_complete() {
        return None;
    }
    if plan.ready_steps().is_empty() {
        let remaining: Vec<String> = plan
            .steps
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    PlanStepStatus::Pending | PlanStepStatus::InProgress
                )
            })
            .map(|s| s.id.clone())
            .collect();
        return Some(PlanExecutionError::Deadlock {
            remaining_step_ids: remaining,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{PlanStep, PlanStepStatus, StructuredPlan};

    fn make_step(id: &str, deps: &[&str], status: PlanStepStatus) -> PlanStep {
        PlanStep {
            id: id.to_string(),
            description: format!("Step {}", id),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            tools_required: vec![],
            estimated_turns: 1,
            status,
            condition: None,
        }
    }

    #[test]
    fn report_all_completed() {
        let plan = StructuredPlan {
            id: "p1".to_string(),
            goal: "test".to_string(),
            steps: vec![
                make_step("a", &[], PlanStepStatus::Completed),
                make_step("b", &["a"], PlanStepStatus::Completed),
            ],
            created_at_ms: 0,
        };
        let report = build_execution_report(&plan);
        assert!(report.is_complete);
        assert!(!report.is_deadlocked);
        assert_eq!(report.completed_steps, 2);
    }

    #[test]
    fn report_with_failure() {
        let plan = StructuredPlan {
            id: "p1".to_string(),
            goal: "test".to_string(),
            steps: vec![
                make_step("a", &[], PlanStepStatus::Completed),
                make_step(
                    "b",
                    &["a"],
                    PlanStepStatus::Failed {
                        reason: "err".to_string(),
                    },
                ),
            ],
            created_at_ms: 0,
        };
        let report = build_execution_report(&plan);
        assert!(!report.is_complete);
        assert_eq!(report.failed_steps, 1);
    }

    #[test]
    fn deadlock_detection() {
        let plan = StructuredPlan {
            id: "p1".to_string(),
            goal: "test".to_string(),
            steps: vec![
                make_step(
                    "a",
                    &[],
                    PlanStepStatus::Failed {
                        reason: "err".to_string(),
                    },
                ),
                make_step("b", &["a"], PlanStepStatus::Pending),
            ],
            created_at_ms: 0,
        };
        let deadlock = check_deadlock(&plan);
        assert!(deadlock.is_some());
    }
}
