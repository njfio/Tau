use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const MISSION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionLifecycleStatus {
    Draft,
    Planned,
    Executing,
    Checkpointed,
    Blocked,
    Verifying,
    Completed,
    Failed,
    Archived,
}

impl MissionLifecycleStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        if self == next {
            return true;
        }

        match self {
            Self::Draft => matches!(next, Self::Planned | Self::Failed | Self::Archived),
            Self::Planned => {
                matches!(
                    next,
                    Self::Executing | Self::Blocked | Self::Failed | Self::Archived
                )
            }
            Self::Executing => {
                matches!(
                    next,
                    Self::Checkpointed
                        | Self::Blocked
                        | Self::Verifying
                        | Self::Completed
                        | Self::Failed
                )
            }
            Self::Checkpointed => {
                matches!(
                    next,
                    Self::Executing
                        | Self::Verifying
                        | Self::Completed
                        | Self::Blocked
                        | Self::Failed
                        | Self::Archived
                )
            }
            Self::Blocked => matches!(next, Self::Executing | Self::Failed | Self::Archived),
            Self::Verifying => {
                matches!(
                    next,
                    Self::Completed | Self::Executing | Self::Blocked | Self::Failed
                )
            }
            Self::Completed | Self::Failed => matches!(next, Self::Archived),
            Self::Archived => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionVerifierStatus {
    Passed,
    Continue,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionCompletionStatus {
    Success,
    Partial,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionSnapshot {
    pub schema_version: u32,
    pub mission_id: String,
    #[serde(default)]
    pub session_key: Option<String>,
    #[serde(default)]
    pub response_id: Option<String>,
    pub goal: String,
    #[serde(default)]
    pub latest_output_summary: String,
    pub status: MissionLifecycleStatus,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
    #[serde(default)]
    pub acceptance_criteria: Vec<MissionAcceptanceCriterion>,
    #[serde(default)]
    pub plan_dag: Vec<MissionPlanNode>,
    #[serde(default)]
    pub tool_budget: MissionToolBudget,
    #[serde(default)]
    pub memory_hits: Vec<MissionMemoryHit>,
    #[serde(default)]
    pub verification_gates: Vec<MissionVerificationGate>,
    #[serde(default)]
    pub checkpoints: Vec<MissionCheckpoint>,
    #[serde(default)]
    pub recovery_state: Option<MissionRecoveryState>,
    #[serde(default)]
    pub artifacts: Vec<MissionArtifactRef>,
    #[serde(default)]
    pub final_learning_output: Option<MissionLearningOutput>,
    #[serde(default)]
    pub iteration_count: usize,
    #[serde(default)]
    pub latest_verifier: Option<MissionVerifierRecord>,
    #[serde(default)]
    pub latest_completion: Option<MissionCompletion>,
}

impl MissionSnapshot {
    pub fn new(
        mission_id: impl Into<String>,
        goal: impl Into<String>,
        created_unix_ms: u64,
    ) -> Self {
        Self {
            schema_version: MISSION_SCHEMA_VERSION,
            mission_id: mission_id.into(),
            session_key: None,
            response_id: None,
            goal: goal.into(),
            latest_output_summary: String::new(),
            status: MissionLifecycleStatus::Draft,
            created_unix_ms,
            updated_unix_ms: created_unix_ms,
            acceptance_criteria: Vec::new(),
            plan_dag: Vec::new(),
            tool_budget: MissionToolBudget::default(),
            memory_hits: Vec::new(),
            verification_gates: Vec::new(),
            checkpoints: Vec::new(),
            recovery_state: None,
            artifacts: Vec::new(),
            final_learning_output: None,
            iteration_count: 0,
            latest_verifier: None,
            latest_completion: None,
        }
    }

    pub fn transition_to(
        &mut self,
        next: MissionLifecycleStatus,
        updated_unix_ms: u64,
    ) -> Result<(), MissionTransitionError> {
        if !self.status.can_transition_to(next) {
            return Err(MissionTransitionError {
                from: self.status,
                to: next,
            });
        }
        self.status = next;
        self.updated_unix_ms = updated_unix_ms;
        Ok(())
    }

    pub fn validate_plan_dag(&self) -> Result<(), Vec<MissionPlanDagError>> {
        let mut errors = Vec::new();
        let mut node_ids = BTreeSet::new();
        let mut duplicate_ids = BTreeSet::new();

        for node in &self.plan_dag {
            if !node_ids.insert(node.id.as_str()) {
                duplicate_ids.insert(node.id.clone());
            }
        }

        for node_id in duplicate_ids {
            errors.push(MissionPlanDagError::DuplicateNodeId { node_id });
        }

        for node in &self.plan_dag {
            for dependency_id in &node.depends_on {
                if !node_ids.contains(dependency_id.as_str()) {
                    errors.push(MissionPlanDagError::MissingDependency {
                        node_id: node.id.clone(),
                        missing_dependency: dependency_id.clone(),
                    });
                }
            }
        }

        if errors.is_empty() && self.plan_dag_contains_cycle() {
            errors.push(MissionPlanDagError::CycleDetected);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn ready_plan_node_ids(&self) -> Vec<String> {
        self.plan_dag
            .iter()
            .filter(|node| {
                mission_plan_status_is_pending(&node.status)
                    && node.depends_on.iter().all(|dependency_id| {
                        self.plan_dag
                            .iter()
                            .find(|candidate| candidate.id == *dependency_id)
                            .is_some_and(|dependency| {
                                mission_plan_status_satisfies_dependency(&dependency.status)
                            })
                    })
            })
            .map(|node| node.id.clone())
            .collect()
    }

    pub fn pending_plan_node_ids(&self) -> Vec<String> {
        self.plan_dag
            .iter()
            .filter(|node| !mission_plan_status_is_terminal_success(&node.status))
            .map(|node| node.id.clone())
            .collect()
    }

    pub fn record_checkpoint(
        &mut self,
        checkpoint_id: impl Into<String>,
        summary: impl Into<String>,
        created_unix_ms: u64,
    ) -> Result<(), MissionTransitionError> {
        let pending_plan_node_ids = self.pending_plan_node_ids();
        self.transition_to(MissionLifecycleStatus::Checkpointed, created_unix_ms)?;
        self.checkpoints.push(MissionCheckpoint {
            checkpoint_id: checkpoint_id.into(),
            summary: summary.into(),
            created_unix_ms,
            pending_plan_node_ids,
        });
        Ok(())
    }

    pub fn block_for_recovery(
        &mut self,
        reason: impl Into<String>,
        next_action: Option<impl Into<String>>,
        updated_unix_ms: u64,
    ) -> Result<(), MissionTransitionError> {
        let retry_count = self
            .recovery_state
            .as_ref()
            .map(|state| state.retry_count.saturating_add(1))
            .unwrap_or_default();
        let last_checkpoint_id = self
            .checkpoints
            .last()
            .map(|checkpoint| checkpoint.checkpoint_id.clone());

        self.transition_to(MissionLifecycleStatus::Blocked, updated_unix_ms)?;
        self.recovery_state = Some(MissionRecoveryState {
            reason: reason.into(),
            next_action: next_action.map(Into::into),
            retry_count,
            last_checkpoint_id,
        });
        Ok(())
    }

    pub fn completion_blockers(&self) -> Vec<MissionCompletionBlocker> {
        let mut blockers = Vec::new();

        if self.plan_dag.is_empty() {
            blockers.push(MissionCompletionBlocker::MissingPlanDag);
        }

        for node in &self.plan_dag {
            if !mission_plan_status_is_terminal_success(&node.status) {
                blockers.push(MissionCompletionBlocker::PlanNodeIncomplete {
                    node_id: node.id.clone(),
                    status: node.status.clone(),
                });
            }
        }

        if self.verification_gates.is_empty() {
            blockers.push(MissionCompletionBlocker::MissingVerificationGates);
        }

        for gate in &self.verification_gates {
            if gate.status != Some(MissionVerifierStatus::Passed) {
                blockers.push(MissionCompletionBlocker::VerificationGateNotPassed {
                    gate_id: gate.id.clone(),
                    status: gate.status,
                });
            }
        }

        if self.final_learning_output.is_none() {
            blockers.push(MissionCompletionBlocker::MissingFinalLearningOutput);
        }

        blockers
    }

    pub fn ready_for_completion(&self) -> bool {
        self.completion_blockers().is_empty()
    }

    fn plan_dag_contains_cycle(&self) -> bool {
        let mut in_degree = BTreeMap::<&str, usize>::new();
        let mut dependents = BTreeMap::<&str, Vec<&str>>::new();

        for node in &self.plan_dag {
            in_degree.entry(node.id.as_str()).or_insert(0);
            dependents.entry(node.id.as_str()).or_default();
            for dependency_id in &node.depends_on {
                dependents
                    .entry(dependency_id.as_str())
                    .or_default()
                    .push(node.id.as_str());
                *in_degree.entry(node.id.as_str()).or_insert(0) += 1;
            }
        }

        let mut ready = in_degree
            .iter()
            .filter_map(|(node_id, degree)| (*degree == 0).then_some(*node_id))
            .collect::<VecDeque<_>>();
        let mut processed = 0usize;

        while let Some(node_id) = ready.pop_front() {
            processed += 1;
            if let Some(next_nodes) = dependents.get(node_id) {
                for next_node_id in next_nodes {
                    if let Some(degree) = in_degree.get_mut(next_node_id) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            ready.push_back(next_node_id);
                        }
                    }
                }
            }
        }

        processed < self.plan_dag.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionAcceptanceCriterion {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub verification_gate_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionPlanNode {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub status: String,
}

fn mission_plan_status_is_pending(status: &str) -> bool {
    let status = normalized_mission_plan_status(status);
    status.is_empty() || status == "pending"
}

fn mission_plan_status_satisfies_dependency(status: &str) -> bool {
    matches!(
        normalized_mission_plan_status(status).as_str(),
        "completed" | "skipped"
    )
}

fn mission_plan_status_is_terminal_success(status: &str) -> bool {
    mission_plan_status_satisfies_dependency(status)
}

fn normalized_mission_plan_status(status: &str) -> String {
    status.trim().to_ascii_lowercase()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MissionToolBudget {
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub max_tool_calls: Option<usize>,
    #[serde(default)]
    pub max_runtime_ms: Option<u64>,
    #[serde(default)]
    pub max_cost_usd: Option<f64>,
    #[serde(default)]
    pub consumed_tool_calls: usize,
    #[serde(default)]
    pub consumed_runtime_ms: u64,
    #[serde(default)]
    pub consumed_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionMemoryHit {
    pub key: String,
    pub summary: String,
    #[serde(default)]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionVerificationGate {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub status: Option<MissionVerifierStatus>,
    #[serde(default)]
    pub evidence: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionCheckpoint {
    pub checkpoint_id: String,
    pub summary: String,
    pub created_unix_ms: u64,
    #[serde(default)]
    pub pending_plan_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionRecoveryState {
    pub reason: String,
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub retry_count: usize,
    #[serde(default)]
    pub last_checkpoint_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionArtifactRef {
    pub artifact_id: String,
    pub kind: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionLearningOutput {
    pub summary: String,
    #[serde(default)]
    pub records: Vec<String>,
    #[serde(default)]
    pub curator_recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionVerifierRecord {
    pub kind: String,
    pub status: MissionVerifierStatus,
    pub reason_code: String,
    pub message: String,
    #[serde(default)]
    pub details: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionCompletion {
    pub status: MissionCompletionStatus,
    pub summary: String,
    #[serde(default)]
    pub next_step: Option<String>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MissionPlanDagError {
    #[error("duplicate mission plan node id {node_id}")]
    DuplicateNodeId { node_id: String },
    #[error("mission plan node {node_id} depends on missing node {missing_dependency}")]
    MissingDependency {
        node_id: String,
        missing_dependency: String,
    },
    #[error("mission plan DAG contains a cycle")]
    CycleDetected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionCompletionBlocker {
    MissingPlanDag,
    PlanNodeIncomplete {
        node_id: String,
        status: String,
    },
    MissingVerificationGates,
    VerificationGateNotPassed {
        gate_id: String,
        status: Option<MissionVerifierStatus>,
    },
    MissingFinalLearningOutput,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("invalid mission transition from {from:?} to {to:?}")]
pub struct MissionTransitionError {
    pub from: MissionLifecycleStatus,
    pub to: MissionLifecycleStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_completed_does_not_transition_back_to_executing() {
        let mut mission = MissionSnapshot::new("mission-alpha", "fix the bug", 100);
        mission
            .transition_to(MissionLifecycleStatus::Planned, 105)
            .expect("plan mission");
        mission
            .transition_to(MissionLifecycleStatus::Executing, 108)
            .expect("execute mission");
        mission
            .transition_to(MissionLifecycleStatus::Completed, 110)
            .expect("complete mission");

        let error = mission
            .transition_to(MissionLifecycleStatus::Executing, 120)
            .expect_err("completed mission should not become executing again");

        assert_eq!(mission.status, MissionLifecycleStatus::Completed);
        assert_eq!(
            error,
            MissionTransitionError {
                from: MissionLifecycleStatus::Completed,
                to: MissionLifecycleStatus::Executing,
            }
        );
    }

    #[test]
    fn mission_snapshot_defaults_include_harness_proof_fields() {
        let mission = MissionSnapshot::new("mission-beta", "write a doc", 200);

        assert_eq!(mission.schema_version, MISSION_SCHEMA_VERSION);
        assert_eq!(mission.mission_id, "mission-beta");
        assert_eq!(mission.goal, "write a doc");
        assert_eq!(mission.status, MissionLifecycleStatus::Draft);
        assert!(mission.acceptance_criteria.is_empty());
        assert!(mission.plan_dag.is_empty());
        assert!(mission.tool_budget.allowed_tools.is_empty());
        assert!(mission.memory_hits.is_empty());
        assert!(mission.verification_gates.is_empty());
        assert!(mission.checkpoints.is_empty());
        assert!(mission.artifacts.is_empty());
        assert!(mission.final_learning_output.is_none());
    }

    fn plan_node(id: &str, depends_on: &[&str], status: &str) -> MissionPlanNode {
        MissionPlanNode {
            id: id.to_string(),
            description: format!("step {id}"),
            depends_on: depends_on.iter().map(|dep| dep.to_string()).collect(),
            status: status.to_string(),
        }
    }

    #[test]
    fn mission_plan_ready_nodes_require_completed_or_skipped_dependencies() {
        let mut mission = MissionSnapshot::new("mission-gamma", "ship a utility", 300);
        mission.plan_dag = vec![
            plan_node("spec", &[], "completed"),
            plan_node("skip_docs", &[], "skipped"),
            plan_node("impl", &["spec"], "pending"),
            plan_node("announce", &["skip_docs"], "pending"),
            plan_node("verify", &["impl"], "pending"),
            plan_node("blocked", &[], "blocked"),
        ];

        mission.validate_plan_dag().expect("valid mission plan DAG");

        assert_eq!(
            mission.ready_plan_node_ids(),
            vec!["impl".to_string(), "announce".to_string()]
        );
    }

    #[test]
    fn mission_plan_validation_reports_missing_dependencies_and_cycles() {
        let mut missing = MissionSnapshot::new("mission-delta", "validate dependencies", 400);
        missing.plan_dag = vec![plan_node("verify", &["missing"], "pending")];

        assert_eq!(
            missing.validate_plan_dag().expect_err("missing dep"),
            vec![MissionPlanDagError::MissingDependency {
                node_id: "verify".to_string(),
                missing_dependency: "missing".to_string(),
            }]
        );

        let mut cyclic = MissionSnapshot::new("mission-epsilon", "validate cycles", 410);
        cyclic.plan_dag = vec![
            plan_node("a", &["b"], "pending"),
            plan_node("b", &["a"], "pending"),
        ];

        assert_eq!(
            cyclic.validate_plan_dag().expect_err("cycle"),
            vec![MissionPlanDagError::CycleDetected]
        );
    }

    #[test]
    fn mission_checkpoint_records_pending_nodes_and_recovery_anchor() {
        let mut mission = MissionSnapshot::new("mission-zeta", "resume safely", 500);
        mission
            .transition_to(MissionLifecycleStatus::Planned, 505)
            .expect("planned");
        mission
            .transition_to(MissionLifecycleStatus::Executing, 510)
            .expect("executing");
        mission.plan_dag = vec![
            plan_node("spec", &[], "completed"),
            plan_node("impl", &["spec"], "in_progress"),
            plan_node("verify", &["impl"], "pending"),
            plan_node("docs", &[], "skipped"),
        ];

        mission
            .record_checkpoint("cp-1", "implementation underway", 520)
            .expect("checkpoint");

        assert_eq!(mission.status, MissionLifecycleStatus::Checkpointed);
        assert_eq!(mission.checkpoints.len(), 1);
        assert_eq!(
            mission.checkpoints[0].pending_plan_node_ids,
            vec!["impl".to_string(), "verify".to_string()]
        );

        mission
            .block_for_recovery("missing credential", Some("request operator approval"), 530)
            .expect("blocked");

        assert_eq!(mission.status, MissionLifecycleStatus::Blocked);
        assert_eq!(
            mission.recovery_state,
            Some(MissionRecoveryState {
                reason: "missing credential".to_string(),
                next_action: Some("request operator approval".to_string()),
                retry_count: 0,
                last_checkpoint_id: Some("cp-1".to_string()),
            })
        );
    }

    #[test]
    fn mission_completion_readiness_requires_plan_verification_and_learning() {
        let mut mission = MissionSnapshot::new("mission-eta", "complete with proof", 600);
        mission.plan_dag = vec![
            plan_node("impl", &[], "completed"),
            plan_node("verify", &["impl"], "pending"),
        ];
        mission.verification_gates.push(MissionVerificationGate {
            id: "gate-1".to_string(),
            description: "tests passed".to_string(),
            status: Some(MissionVerifierStatus::Continue),
            evidence: BTreeMap::new(),
        });

        assert_eq!(
            mission.completion_blockers(),
            vec![
                MissionCompletionBlocker::PlanNodeIncomplete {
                    node_id: "verify".to_string(),
                    status: "pending".to_string(),
                },
                MissionCompletionBlocker::VerificationGateNotPassed {
                    gate_id: "gate-1".to_string(),
                    status: Some(MissionVerifierStatus::Continue),
                },
                MissionCompletionBlocker::MissingFinalLearningOutput,
            ]
        );
        assert!(!mission.ready_for_completion());

        mission.plan_dag[1].status = "completed".to_string();
        mission.verification_gates[0].status = Some(MissionVerifierStatus::Passed);
        mission.final_learning_output = Some(MissionLearningOutput {
            summary: "learned to keep proof attached".to_string(),
            records: vec!["learning-record-1".to_string()],
            curator_recommendation: Some("retain skill prompt".to_string()),
        });

        assert!(mission.completion_blockers().is_empty());
        assert!(mission.ready_for_completion());
    }
}
