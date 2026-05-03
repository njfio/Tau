use std::collections::BTreeMap;

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
}
