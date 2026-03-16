//! Structured plan data model with DAG representation.
//!
//! Plans are directed acyclic graphs of steps, where each step can depend on
//! others. Steps without dependencies can execute in parallel.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

/// A structured execution plan with dependency tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub created_at_ms: u64,
}

/// A single step in a structured plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub depends_on: Vec<String>,
    pub tools_required: Vec<String>,
    pub estimated_turns: usize,
    pub status: PlanStepStatus,
    #[serde(default)]
    pub condition: Option<PlanCondition>,
}

/// Execution status of a plan step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed { reason: String },
    Skipped { reason: String },
}

/// Conditional execution criteria for a plan step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanCondition {
    IfSucceeded(String),
    IfFailed(String),
    Always,
}

/// Errors arising from plan validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanValidationError {
    /// A step references a dependency that does not exist.
    MissingDependency {
        step_id: String,
        missing_dep: String,
    },
    /// The dependency graph contains a cycle.
    CycleDetected { involved_steps: Vec<String> },
    /// A step requires a tool that is not available.
    UnavailableTool {
        step_id: String,
        tool_name: String,
    },
    /// Duplicate step IDs found.
    DuplicateStepId { step_id: String },
}

impl std::fmt::Display for PlanValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanValidationError::MissingDependency {
                step_id,
                missing_dep,
            } => write!(
                f,
                "step '{}' depends on non-existent step '{}'",
                step_id, missing_dep
            ),
            PlanValidationError::CycleDetected { involved_steps } => {
                write!(f, "dependency cycle among steps: {:?}", involved_steps)
            }
            PlanValidationError::UnavailableTool {
                step_id,
                tool_name,
            } => write!(
                f,
                "step '{}' requires unavailable tool '{}'",
                step_id, tool_name
            ),
            PlanValidationError::DuplicateStepId { step_id } => {
                write!(f, "duplicate step id '{}'", step_id)
            }
        }
    }
}

impl std::error::Error for PlanValidationError {}

impl PlanStep {
    /// Check if this step's condition is satisfied given the plan state.
    pub fn condition_satisfied(&self, plan: &StructuredPlan) -> bool {
        match &self.condition {
            None | Some(PlanCondition::Always) => true,
            Some(PlanCondition::IfSucceeded(dep_id)) => plan
                .steps
                .iter()
                .any(|s| s.id == *dep_id && s.status == PlanStepStatus::Completed),
            Some(PlanCondition::IfFailed(dep_id)) => plan
                .steps
                .iter()
                .any(|s| s.id == *dep_id && matches!(s.status, PlanStepStatus::Failed { .. })),
        }
    }
}

impl StructuredPlan {
    /// Returns steps that are ready to execute (all dependencies satisfied).
    pub fn ready_steps(&self) -> Vec<&PlanStep> {
        self.steps
            .iter()
            .filter(|step| {
                step.status == PlanStepStatus::Pending
                    && step.depends_on.iter().all(|dep_id| {
                        self.steps.iter().any(|s| {
                            s.id == *dep_id
                                && matches!(
                                    s.status,
                                    PlanStepStatus::Completed | PlanStepStatus::Skipped { .. }
                                )
                        })
                    })
                    && step.condition_satisfied(self)
            })
            .collect()
    }

    /// Returns true if the plan is complete (all steps completed or skipped).
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| {
            matches!(
                s.status,
                PlanStepStatus::Completed | PlanStepStatus::Skipped { .. }
            )
        })
    }

    /// Returns true if any step has failed.
    pub fn has_failures(&self) -> bool {
        self.steps
            .iter()
            .any(|s| matches!(s.status, PlanStepStatus::Failed { .. }))
    }

    /// Update the status of a step by ID.
    pub fn update_step_status(&mut self, step_id: &str, status: PlanStepStatus) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = status;
        }
    }

    /// Returns groups of steps that can be executed in parallel.
    pub fn parallelizable_groups(&self) -> Vec<Vec<&PlanStep>> {
        let ready = self.ready_steps();
        if ready.is_empty() {
            return Vec::new();
        }
        // All ready steps can run in parallel by definition
        vec![ready]
    }

    /// Validates the plan DAG for cycles, missing dependencies, and tool availability.
    pub fn validate(
        &self,
        available_tools: &[String],
    ) -> Result<(), Vec<PlanValidationError>> {
        let mut errors = Vec::new();
        let tool_set: HashSet<&str> = available_tools.iter().map(|s| s.as_str()).collect();
        let step_ids: HashSet<&str> = self.steps.iter().map(|s| s.id.as_str()).collect();

        // Check for duplicate IDs
        let mut seen = HashSet::new();
        for step in &self.steps {
            if !seen.insert(&step.id) {
                errors.push(PlanValidationError::DuplicateStepId {
                    step_id: step.id.clone(),
                });
            }
        }

        // Check dependencies exist
        for step in &self.steps {
            for dep in &step.depends_on {
                if !step_ids.contains(dep.as_str()) {
                    errors.push(PlanValidationError::MissingDependency {
                        step_id: step.id.clone(),
                        missing_dep: dep.clone(),
                    });
                }
            }
        }

        // Check tools available
        for step in &self.steps {
            for tool in &step.tools_required {
                if !tool_set.contains(tool.as_str()) {
                    errors.push(PlanValidationError::UnavailableTool {
                        step_id: step.id.clone(),
                        tool_name: tool.clone(),
                    });
                }
            }
        }

        // Check for cycles using topological sort (Kahn's algorithm)
        if let Some(cycle_error) = detect_cycle(&self.steps) {
            errors.push(cycle_error);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Estimated total turns for the plan.
    pub fn estimated_total_turns(&self) -> usize {
        self.steps.iter().map(|s| s.estimated_turns).sum()
    }
}

fn detect_cycle(steps: &[PlanStep]) -> Option<PlanValidationError> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for step in steps {
        in_degree.entry(step.id.as_str()).or_insert(0);
        adj.entry(step.id.as_str()).or_default();
        for dep in &step.depends_on {
            adj.entry(dep.as_str()).or_default().push(step.id.as_str());
            *in_degree.entry(step.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut processed = 0usize;
    while let Some(node) = queue.pop_front() {
        processed += 1;
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                if let Some(deg) = in_degree.get_mut(next) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    if processed < steps.len() {
        let involved: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg > 0)
            .map(|(&id, _)| id.to_string())
            .collect();
        Some(PlanValidationError::CycleDetected {
            involved_steps: involved,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(id: &str, deps: &[&str]) -> PlanStep {
        PlanStep {
            id: id.to_string(),
            description: format!("Step {}", id),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            tools_required: vec![],
            estimated_turns: 1,
            status: PlanStepStatus::Pending,
            condition: None,
        }
    }

    #[test]
    fn empty_plan_is_complete() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![],
            created_at_ms: 0,
        };
        assert!(plan.is_complete());
    }

    #[test]
    fn ready_steps_returns_root_nodes() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![make_step("a", &[]), make_step("b", &["a"])],
            created_at_ms: 0,
        };
        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "a");
    }

    #[test]
    fn ready_steps_after_completion() {
        let mut plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![make_step("a", &[]), make_step("b", &["a"])],
            created_at_ms: 0,
        };
        plan.update_step_status("a", PlanStepStatus::Completed);
        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "b");
    }

    #[test]
    fn validate_detects_missing_deps() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![make_step("a", &["nonexistent"])],
            created_at_ms: 0,
        };
        let result = plan.validate(&[]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, PlanValidationError::MissingDependency { .. })));
    }

    #[test]
    fn validate_detects_cycles() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![make_step("a", &["b"]), make_step("b", &["a"])],
            created_at_ms: 0,
        };
        let result = plan.validate(&[]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, PlanValidationError::CycleDetected { .. })));
    }

    #[test]
    fn validate_ok_for_valid_dag() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![
                make_step("a", &[]),
                make_step("b", &["a"]),
                make_step("c", &["a"]),
                make_step("d", &["b", "c"]),
            ],
            created_at_ms: 0,
        };
        assert!(plan.validate(&[]).is_ok());
    }

    #[test]
    fn condition_if_succeeded() {
        let mut plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![
                make_step("a", &[]),
                PlanStep {
                    id: "b".to_string(),
                    description: "conditional step".to_string(),
                    depends_on: vec!["a".to_string()],
                    tools_required: vec![],
                    estimated_turns: 1,
                    status: PlanStepStatus::Pending,
                    condition: Some(PlanCondition::IfSucceeded("a".to_string())),
                },
            ],
            created_at_ms: 0,
        };

        // Before a is completed, b should not be ready
        plan.update_step_status("a", PlanStepStatus::Pending);
        assert!(plan.ready_steps().iter().all(|s| s.id != "b"));

        // After a succeeds, b should be ready
        plan.update_step_status("a", PlanStepStatus::Completed);
        assert!(plan.ready_steps().iter().any(|s| s.id == "b"));
    }

    #[test]
    fn parallelizable_groups_independent_steps() {
        let plan = StructuredPlan {
            id: "test".to_string(),
            goal: "test".to_string(),
            steps: vec![make_step("a", &[]), make_step("b", &[]), make_step("c", &[])],
            created_at_ms: 0,
        };
        let groups = plan.parallelizable_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }
}
