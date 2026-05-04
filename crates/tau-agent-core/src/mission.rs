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
    pub tool_evidence: Vec<MissionToolCallEvidence>,
    #[serde(default)]
    pub memory_hits: Vec<MissionMemoryHit>,
    #[serde(default)]
    pub memory_recall: Option<MissionMemoryRecallEvidence>,
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
    pub learning_records: Vec<MissionLearningRecord>,
    #[serde(default)]
    pub improvement_proposals: Vec<MissionImprovementProposal>,
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
            tool_evidence: Vec::new(),
            memory_hits: Vec::new(),
            memory_recall: None,
            verification_gates: Vec::new(),
            checkpoints: Vec::new(),
            recovery_state: None,
            artifacts: Vec::new(),
            final_learning_output: None,
            learning_records: Vec::new(),
            improvement_proposals: Vec::new(),
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

        if self.memory_recall.is_none() {
            blockers.push(MissionCompletionBlocker::MissingMemoryRecallEvidence);
        }

        if self.tool_budget.consumed_tool_calls > self.tool_evidence.len() {
            blockers.push(MissionCompletionBlocker::MissingToolEvidence {
                consumed_tool_calls: self.tool_budget.consumed_tool_calls,
                recorded_tool_calls: self.tool_evidence.len(),
            });
        }

        blockers
    }

    pub fn ready_for_completion(&self) -> bool {
        self.completion_blockers().is_empty()
    }

    pub fn ensure_tool_budget_available(
        &self,
        tool_name: &str,
        runtime_ms: Option<u64>,
        cost_usd: Option<f64>,
    ) -> Result<(), MissionToolBudgetError> {
        if !self.tool_budget.allowed_tools.is_empty()
            && !self
                .tool_budget
                .allowed_tools
                .iter()
                .any(|allowed_tool| allowed_tool == tool_name)
        {
            return Err(MissionToolBudgetError::ToolNotAllowed {
                tool_name: tool_name.to_string(),
            });
        }

        if let Some(max_tool_calls) = self.tool_budget.max_tool_calls {
            if self.tool_budget.consumed_tool_calls >= max_tool_calls {
                return Err(MissionToolBudgetError::ToolCallBudgetExhausted { max_tool_calls });
            }
        }

        if let Some(max_runtime_ms) = self.tool_budget.max_runtime_ms {
            let next_runtime = self
                .tool_budget
                .consumed_runtime_ms
                .saturating_add(runtime_ms.unwrap_or_default());
            if next_runtime > max_runtime_ms {
                return Err(MissionToolBudgetError::RuntimeBudgetExhausted { max_runtime_ms });
            }
        }

        if let Some(max_cost_usd) = self.tool_budget.max_cost_usd {
            let next_cost = self.tool_budget.consumed_cost_usd.unwrap_or_default()
                + cost_usd.unwrap_or_default();
            if next_cost > max_cost_usd {
                return Err(MissionToolBudgetError::CostBudgetExhausted { max_cost_usd });
            }
        }

        Ok(())
    }

    pub fn record_tool_call_evidence(
        &mut self,
        evidence: MissionToolCallEvidence,
    ) -> Result<(), MissionToolEvidenceError> {
        if evidence.mission_id != self.mission_id {
            return Err(MissionToolEvidenceError::MissionIdMismatch {
                expected: self.mission_id.clone(),
                actual: evidence.mission_id,
            });
        }

        self.ensure_tool_budget_available(
            &evidence.tool_name,
            evidence.runtime_ms,
            evidence.cost_usd,
        )
        .map_err(MissionToolEvidenceError::Budget)?;

        self.tool_budget.consumed_tool_calls =
            self.tool_budget.consumed_tool_calls.saturating_add(1);
        self.tool_budget.consumed_runtime_ms = self
            .tool_budget
            .consumed_runtime_ms
            .saturating_add(evidence.runtime_ms.unwrap_or_default());
        if let Some(cost_usd) = evidence.cost_usd {
            self.tool_budget.consumed_cost_usd =
                Some(self.tool_budget.consumed_cost_usd.unwrap_or_default() + cost_usd);
        }
        self.tool_evidence.push(evidence);
        Ok(())
    }

    pub fn tool_evidence_for_verification_gate(
        &self,
        gate_id: &str,
    ) -> Vec<&MissionToolCallEvidence> {
        self.tool_evidence
            .iter()
            .filter(|evidence| {
                evidence
                    .verification_gate_ids
                    .iter()
                    .any(|candidate_gate_id| candidate_gate_id == gate_id)
            })
            .collect()
    }

    pub fn record_memory_hit(
        &mut self,
        query: impl Into<String>,
        rationale: impl Into<String>,
        checked_unix_ms: u64,
        hit: MissionMemoryHit,
    ) -> Result<(), MissionMemoryEvidenceError> {
        if hit.key.trim().is_empty() {
            return Err(MissionMemoryEvidenceError::EmptyMemoryHitKey);
        }

        if self
            .memory_recall
            .as_ref()
            .is_some_and(|recall| recall.status == MissionMemoryRecallStatus::NoRelevantMemory)
        {
            return Err(MissionMemoryEvidenceError::MemoryHitConflictsWithNoMemoryResult);
        }

        self.memory_hits.push(hit);
        self.memory_recall = Some(MissionMemoryRecallEvidence {
            query: query.into(),
            status: MissionMemoryRecallStatus::UsedHits,
            checked_unix_ms,
            rationale: rationale.into(),
            hit_keys: self
                .memory_hits
                .iter()
                .map(|memory_hit| memory_hit.key.clone())
                .collect(),
        });
        Ok(())
    }

    pub fn record_no_memory_result(
        &mut self,
        query: impl Into<String>,
        rationale: impl Into<String>,
        checked_unix_ms: u64,
    ) -> Result<(), MissionMemoryEvidenceError> {
        if !self.memory_hits.is_empty() {
            return Err(
                MissionMemoryEvidenceError::NoMemoryResultConflictsWithHits {
                    recorded_hits: self.memory_hits.len(),
                },
            );
        }

        self.memory_recall = Some(MissionMemoryRecallEvidence {
            query: query.into(),
            status: MissionMemoryRecallStatus::NoRelevantMemory,
            checked_unix_ms,
            rationale: rationale.into(),
            hit_keys: Vec::new(),
        });
        Ok(())
    }

    pub fn write_final_learning_output(
        &mut self,
        store: &tau_memory::runtime::FileMemoryStore,
        scope: &tau_memory::memory_contract::MemoryScope,
        mut output: MissionLearningOutput,
        record: MissionLearningRecord,
    ) -> Result<tau_memory::runtime::MemoryWriteResult, MissionLearningRecordError> {
        self.validate_learning_record(&record, MissionLearningRecordKind::Final)?;
        if !output
            .records
            .iter()
            .any(|record_id| record_id == &record.record_id)
        {
            output.records.push(record.record_id.clone());
        }

        let result = write_learning_record_to_memory(store, scope, &record)?;
        upsert_learning_record(&mut self.learning_records, record);
        self.final_learning_output = Some(output);
        Ok(result)
    }

    pub fn write_failure_learning_record(
        &mut self,
        store: &tau_memory::runtime::FileMemoryStore,
        scope: &tau_memory::memory_contract::MemoryScope,
        record: MissionLearningRecord,
    ) -> Result<tau_memory::runtime::MemoryWriteResult, MissionLearningRecordError> {
        if self.recovery_state.is_none() {
            return Err(MissionLearningRecordError::MissingRecoveryState);
        }

        self.validate_learning_record(&record, MissionLearningRecordKind::Failure)?;
        let result = write_learning_record_to_memory(store, scope, &record)?;
        upsert_learning_record(&mut self.learning_records, record);
        Ok(result)
    }

    pub fn record_improvement_proposal_from_failure(
        &mut self,
        mut proposal: MissionImprovementProposal,
    ) -> Result<(), MissionImprovementProposalError> {
        self.validate_improvement_proposal(&proposal)?;

        if !self.learning_records.iter().any(|record| {
            record.record_id == proposal.source_learning_record_id
                && record.kind == MissionLearningRecordKind::Failure
        }) {
            return Err(
                MissionImprovementProposalError::MissingFailureLearningRecord {
                    record_id: proposal.source_learning_record_id,
                },
            );
        }

        proposal.status = MissionImprovementProposalStatus::Proposed;
        upsert_improvement_proposal(&mut self.improvement_proposals, proposal);
        Ok(())
    }

    pub fn record_improvement_dry_run(
        &mut self,
        proposal_id: &str,
        dry_run: MissionImprovementDryRun,
    ) -> Result<(), MissionImprovementProposalError> {
        let proposal = self.improvement_proposal_mut(proposal_id)?;
        proposal.dry_run = Some(dry_run);
        proposal.status = MissionImprovementProposalStatus::DryRunRecorded;
        Ok(())
    }

    pub fn record_improvement_test_evidence(
        &mut self,
        proposal_id: &str,
        evidence: MissionImprovementTestEvidence,
    ) -> Result<(), MissionImprovementProposalError> {
        let proposal = self.improvement_proposal_mut(proposal_id)?;
        proposal.tests.push(evidence);
        Ok(())
    }

    pub fn record_improvement_safety_check(
        &mut self,
        proposal_id: &str,
        check: MissionImprovementSafetyCheck,
    ) -> Result<(), MissionImprovementProposalError> {
        let proposal = self.improvement_proposal_mut(proposal_id)?;
        proposal.safety_checks.push(check);
        Ok(())
    }

    pub fn approve_improvement_proposal(
        &mut self,
        proposal_id: &str,
        approval: MissionOperatorApproval,
    ) -> Result<(), MissionImprovementProposalError> {
        validate_improvement_approval_eligibility(self.improvement_proposal(proposal_id)?)?;
        let proposal = self.improvement_proposal_mut(proposal_id)?;
        proposal.approval = Some(approval);
        proposal.status = MissionImprovementProposalStatus::Approved;
        Ok(())
    }

    pub fn apply_approved_improvement(
        &mut self,
        proposal_id: &str,
        applied_unix_ms: u64,
        curator_memory_record_id: impl Into<String>,
    ) -> Result<(), MissionImprovementProposalError> {
        let curator_memory_record_id = curator_memory_record_id.into();
        if curator_memory_record_id.trim().is_empty() {
            return Err(MissionImprovementProposalError::EmptyCuratorMemoryRecordId);
        }

        let proposal_index = self.improvement_proposal_index(proposal_id)?;
        if self.improvement_proposals[proposal_index]
            .approval
            .is_none()
        {
            return Err(MissionImprovementProposalError::MissingOperatorApproval {
                proposal_id: proposal_id.to_string(),
            });
        }
        if self.improvement_proposals[proposal_index].status
            == MissionImprovementProposalStatus::Applied
        {
            return Err(MissionImprovementProposalError::AlreadyApplied {
                proposal_id: proposal_id.to_string(),
            });
        }

        let source_learning_record_id = self.improvement_proposals[proposal_index]
            .source_learning_record_id
            .clone();
        let proposal_id = self.improvement_proposals[proposal_index]
            .proposal_id
            .clone();
        self.improvement_proposals[proposal_index].status =
            MissionImprovementProposalStatus::Applied;
        self.improvement_proposals[proposal_index].applied_unix_ms = Some(applied_unix_ms);
        self.improvement_proposals[proposal_index].curator_memory_record_id =
            Some(curator_memory_record_id.clone());

        let learning_record = self
            .learning_records
            .iter_mut()
            .find(|record| record.record_id == source_learning_record_id)
            .ok_or_else(
                || MissionImprovementProposalError::MissingFailureLearningRecord {
                    record_id: source_learning_record_id.clone(),
                },
            )?;
        learning_record.curator_status = MissionCuratorReviewStatus::Applied;
        learning_record.metadata.insert(
            "accepted_improvement_proposal_id".to_string(),
            Value::String(proposal_id),
        );
        learning_record.metadata.insert(
            "accepted_improvement_curator_memory_record_id".to_string(),
            Value::String(curator_memory_record_id),
        );

        Ok(())
    }

    fn validate_learning_record(
        &self,
        record: &MissionLearningRecord,
        expected_kind: MissionLearningRecordKind,
    ) -> Result<(), MissionLearningRecordError> {
        if record.record_id.trim().is_empty() {
            return Err(MissionLearningRecordError::EmptyRecordId);
        }
        if record.mission_id != self.mission_id {
            return Err(MissionLearningRecordError::MissionIdMismatch {
                expected: self.mission_id.clone(),
                actual: record.mission_id.clone(),
            });
        }
        if record.kind != expected_kind {
            return Err(MissionLearningRecordError::UnexpectedKind {
                expected: expected_kind,
                actual: record.kind,
            });
        }
        Ok(())
    }

    fn validate_improvement_proposal(
        &self,
        proposal: &MissionImprovementProposal,
    ) -> Result<(), MissionImprovementProposalError> {
        if proposal.proposal_id.trim().is_empty() {
            return Err(MissionImprovementProposalError::EmptyProposalId);
        }
        if proposal.mission_id != self.mission_id {
            return Err(MissionImprovementProposalError::MissionIdMismatch {
                expected: self.mission_id.clone(),
                actual: proposal.mission_id.clone(),
            });
        }
        if !proposal.target_kind.is_allowed_in_conservative_loop() {
            return Err(MissionImprovementProposalError::UnsafeTargetKind {
                target_kind: proposal.target_kind,
            });
        }

        let safety_proposal = tau_safety::SelfModificationProposal {
            target_path: proposal.target_path.clone(),
            diff: proposal.patch_summary.clone(),
            rationale: proposal.rationale.clone(),
            trigger_source: proposal.trigger_kind.as_str().to_string(),
        };
        let safety_eval = tau_safety::evaluate_self_modification(
            &safety_proposal,
            &tau_safety::default_self_modification_rules(),
        );
        if !safety_eval.allowed {
            return Err(MissionImprovementProposalError::SafetyBlocked {
                proposal_id: proposal.proposal_id.clone(),
                blocked_by: safety_eval.blocked_by,
            });
        }

        Ok(())
    }

    fn improvement_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<&MissionImprovementProposal, MissionImprovementProposalError> {
        self.improvement_proposals
            .iter()
            .find(|proposal| proposal.proposal_id == proposal_id)
            .ok_or_else(|| MissionImprovementProposalError::ProposalNotFound {
                proposal_id: proposal_id.to_string(),
            })
    }

    fn improvement_proposal_mut(
        &mut self,
        proposal_id: &str,
    ) -> Result<&mut MissionImprovementProposal, MissionImprovementProposalError> {
        self.improvement_proposals
            .iter_mut()
            .find(|proposal| proposal.proposal_id == proposal_id)
            .ok_or_else(|| MissionImprovementProposalError::ProposalNotFound {
                proposal_id: proposal_id.to_string(),
            })
    }

    fn improvement_proposal_index(
        &self,
        proposal_id: &str,
    ) -> Result<usize, MissionImprovementProposalError> {
        self.improvement_proposals
            .iter()
            .position(|proposal| proposal.proposal_id == proposal_id)
            .ok_or_else(|| MissionImprovementProposalError::ProposalNotFound {
                proposal_id: proposal_id.to_string(),
            })
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionToolCallStatus {
    Started,
    Succeeded,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionToolCallEvidence {
    pub tool_call_id: String,
    pub mission_id: String,
    #[serde(default)]
    pub plan_node_id: Option<String>,
    pub tool_name: String,
    pub status: MissionToolCallStatus,
    pub started_unix_ms: u64,
    #[serde(default)]
    pub completed_unix_ms: Option<u64>,
    #[serde(default)]
    pub runtime_ms: Option<u64>,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub verification_gate_ids: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionMemoryHit {
    pub key: String,
    pub summary: String,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub source_event_key: Option<String>,
    #[serde(default)]
    pub plan_rationale: Option<String>,
    #[serde(default)]
    pub used_in_plan_node_ids: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionMemoryRecallStatus {
    UsedHits,
    NoRelevantMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionMemoryRecallEvidence {
    pub query: String,
    pub status: MissionMemoryRecallStatus,
    pub checked_unix_ms: u64,
    pub rationale: String,
    #[serde(default)]
    pub hit_keys: Vec<String>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionLearningRecordKind {
    Final,
    Failure,
}

impl MissionLearningRecordKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Final => "final",
            Self::Failure => "failure",
        }
    }

    fn memory_type(self) -> tau_memory::runtime::MemoryType {
        match self {
            Self::Final => tau_memory::runtime::MemoryType::Decision,
            Self::Failure => tau_memory::runtime::MemoryType::Observation,
        }
    }

    fn importance(self) -> f32 {
        match self {
            Self::Final => 0.85,
            Self::Failure => 0.75,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionCuratorReviewStatus {
    QueuedForReview,
    Reviewed,
    Applied,
    Rejected,
}

impl MissionCuratorReviewStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::QueuedForReview => "queued_for_review",
            Self::Reviewed => "reviewed",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionLearningRecord {
    pub record_id: String,
    pub mission_id: String,
    pub kind: MissionLearningRecordKind,
    pub summary: String,
    pub created_unix_ms: u64,
    pub curator_status: MissionCuratorReviewStatus,
    #[serde(default)]
    pub root_cause: Option<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub verification_gate_ids: Vec<String>,
    #[serde(default)]
    pub rollback_plan: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionImprovementTriggerKind {
    BenchmarkFailure,
    VerifierFailure,
    OperatorCorrection,
    RecoveryFallback,
}

impl MissionImprovementTriggerKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::BenchmarkFailure => "benchmark_failure",
            Self::VerifierFailure => "verifier_failure",
            Self::OperatorCorrection => "operator_correction",
            Self::RecoveryFallback => "recovery_fallback",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionImprovementTargetKind {
    Skill,
    Config,
    Prompt,
    SourceCode,
    SafetyPolicy,
}

impl MissionImprovementTargetKind {
    fn is_allowed_in_conservative_loop(self) -> bool {
        matches!(self, Self::Skill | Self::Config | Self::Prompt)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionImprovementProposalStatus {
    Proposed,
    DryRunRecorded,
    Approved,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionImprovementEvidenceStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionImprovementDryRun {
    pub ran_unix_ms: u64,
    pub passed: bool,
    pub summary: String,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionImprovementTestEvidence {
    pub command: String,
    pub status: MissionImprovementEvidenceStatus,
    pub ran_unix_ms: u64,
    pub summary: String,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionImprovementSafetyCheck {
    pub check_id: String,
    pub status: MissionImprovementEvidenceStatus,
    pub ran_unix_ms: u64,
    pub summary: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionOperatorApproval {
    pub approval_id: String,
    pub operator_id: String,
    pub approved_unix_ms: u64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissionImprovementProposal {
    pub proposal_id: String,
    pub mission_id: String,
    pub source_learning_record_id: String,
    pub trigger_kind: MissionImprovementTriggerKind,
    pub target_kind: MissionImprovementTargetKind,
    pub target_path: String,
    pub patch_summary: String,
    pub rationale: String,
    pub rollback_plan: String,
    pub proposed_unix_ms: u64,
    pub status: MissionImprovementProposalStatus,
    #[serde(default)]
    pub dry_run: Option<MissionImprovementDryRun>,
    #[serde(default)]
    pub tests: Vec<MissionImprovementTestEvidence>,
    #[serde(default)]
    pub safety_checks: Vec<MissionImprovementSafetyCheck>,
    #[serde(default)]
    pub approval: Option<MissionOperatorApproval>,
    #[serde(default)]
    pub applied_unix_ms: Option<u64>,
    #[serde(default)]
    pub curator_memory_record_id: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
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

fn write_learning_record_to_memory(
    store: &tau_memory::runtime::FileMemoryStore,
    scope: &tau_memory::memory_contract::MemoryScope,
    record: &MissionLearningRecord,
) -> Result<tau_memory::runtime::MemoryWriteResult, MissionLearningRecordError> {
    let entry = tau_memory::memory_contract::MemoryEntry {
        memory_id: record.record_id.clone(),
        summary: record.summary.clone(),
        tags: learning_record_tags(record),
        facts: learning_record_facts(record),
        source_event_key: format!(
            "mission:{}:learning:{}",
            record.mission_id, record.record_id
        ),
        recency_weight_bps: 1_000,
        confidence_bps: 9_000,
    };

    store
        .write_entry_with_metadata(
            scope,
            entry,
            Some(record.kind.memory_type()),
            Some(record.kind.importance()),
        )
        .map_err(|error| MissionLearningRecordError::MemoryWrite {
            message: error.to_string(),
        })
}

fn learning_record_tags(record: &MissionLearningRecord) -> Vec<String> {
    let mut tags = vec![
        "mission_learning".to_string(),
        format!("mission:{}", record.mission_id),
        format!("kind:{}", record.kind.as_str()),
        format!("curator:{}", record.curator_status.as_str()),
    ];
    tags.extend(
        record
            .artifact_ids
            .iter()
            .map(|artifact_id| format!("artifact:{artifact_id}")),
    );
    tags.extend(
        record
            .verification_gate_ids
            .iter()
            .map(|gate_id| format!("verification_gate:{gate_id}")),
    );
    tags
}

fn learning_record_facts(record: &MissionLearningRecord) -> Vec<String> {
    let mut facts = vec![
        format!("mission_id: {}", record.mission_id),
        format!("kind: {}", record.kind.as_str()),
        format!("curator_status: {}", record.curator_status.as_str()),
    ];
    if let Some(root_cause) = &record.root_cause {
        facts.push(format!("root_cause: {root_cause}"));
    }
    if let Some(rollback_plan) = &record.rollback_plan {
        facts.push(format!("rollback_plan: {rollback_plan}"));
    }
    facts.extend(
        record
            .evidence
            .iter()
            .map(|evidence| format!("evidence: {evidence}")),
    );
    facts
}

fn upsert_learning_record(records: &mut Vec<MissionLearningRecord>, record: MissionLearningRecord) {
    if let Some(existing) = records
        .iter_mut()
        .find(|existing| existing.record_id == record.record_id)
    {
        *existing = record;
    } else {
        records.push(record);
    }
}

fn upsert_improvement_proposal(
    proposals: &mut Vec<MissionImprovementProposal>,
    proposal: MissionImprovementProposal,
) {
    if let Some(existing) = proposals
        .iter_mut()
        .find(|existing| existing.proposal_id == proposal.proposal_id)
    {
        *existing = proposal;
    } else {
        proposals.push(proposal);
    }
}

fn validate_improvement_approval_eligibility(
    proposal: &MissionImprovementProposal,
) -> Result<(), MissionImprovementProposalError> {
    match &proposal.dry_run {
        Some(dry_run) if dry_run.passed => {}
        Some(_) => {
            return Err(MissionImprovementProposalError::DryRunFailed {
                proposal_id: proposal.proposal_id.clone(),
            });
        }
        None => {
            return Err(MissionImprovementProposalError::MissingDryRun {
                proposal_id: proposal.proposal_id.clone(),
            });
        }
    }

    if !proposal
        .tests
        .iter()
        .any(|test| test.status == MissionImprovementEvidenceStatus::Passed)
    {
        return Err(
            MissionImprovementProposalError::MissingPassingTestEvidence {
                proposal_id: proposal.proposal_id.clone(),
            },
        );
    }
    if proposal
        .tests
        .iter()
        .any(|test| test.status == MissionImprovementEvidenceStatus::Failed)
    {
        return Err(MissionImprovementProposalError::FailingTestEvidence {
            proposal_id: proposal.proposal_id.clone(),
        });
    }

    if !proposal
        .safety_checks
        .iter()
        .any(|check| check.status == MissionImprovementEvidenceStatus::Passed)
    {
        return Err(
            MissionImprovementProposalError::MissingPassingSafetyEvidence {
                proposal_id: proposal.proposal_id.clone(),
            },
        );
    }
    if proposal
        .safety_checks
        .iter()
        .any(|check| check.status == MissionImprovementEvidenceStatus::Failed)
    {
        return Err(MissionImprovementProposalError::FailingSafetyEvidence {
            proposal_id: proposal.proposal_id.clone(),
        });
    }

    Ok(())
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
    MissingMemoryRecallEvidence,
    MissingToolEvidence {
        consumed_tool_calls: usize,
        recorded_tool_calls: usize,
    },
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum MissionToolBudgetError {
    #[error("tool {tool_name} is not allowed by mission tool budget")]
    ToolNotAllowed { tool_name: String },
    #[error("mission tool call budget exhausted at {max_tool_calls} calls")]
    ToolCallBudgetExhausted { max_tool_calls: usize },
    #[error("mission runtime budget exhausted at {max_runtime_ms}ms")]
    RuntimeBudgetExhausted { max_runtime_ms: u64 },
    #[error("mission cost budget exhausted at ${max_cost_usd}")]
    CostBudgetExhausted { max_cost_usd: f64 },
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum MissionToolEvidenceError {
    #[error("tool evidence mission id mismatch: expected {expected}, got {actual}")]
    MissionIdMismatch { expected: String, actual: String },
    #[error(transparent)]
    Budget(#[from] MissionToolBudgetError),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MissionMemoryEvidenceError {
    #[error("mission memory hit key must not be empty")]
    EmptyMemoryHitKey,
    #[error("cannot record memory hit after an explicit no-memory result")]
    MemoryHitConflictsWithNoMemoryResult,
    #[error("cannot record explicit no-memory result with {recorded_hits} memory hits")]
    NoMemoryResultConflictsWithHits { recorded_hits: usize },
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MissionLearningRecordError {
    #[error("mission learning record id must not be empty")]
    EmptyRecordId,
    #[error("learning record mission id mismatch: expected {expected}, got {actual}")]
    MissionIdMismatch { expected: String, actual: String },
    #[error("learning record kind mismatch: expected {expected:?}, got {actual:?}")]
    UnexpectedKind {
        expected: MissionLearningRecordKind,
        actual: MissionLearningRecordKind,
    },
    #[error("failure learning record requires recovery state")]
    MissingRecoveryState,
    #[error("failed to write learning record to memory: {message}")]
    MemoryWrite { message: String },
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MissionImprovementProposalError {
    #[error("mission improvement proposal id must not be empty")]
    EmptyProposalId,
    #[error("improvement proposal mission id mismatch: expected {expected}, got {actual}")]
    MissionIdMismatch { expected: String, actual: String },
    #[error("improvement proposal target kind {target_kind:?} is outside the conservative loop")]
    UnsafeTargetKind {
        target_kind: MissionImprovementTargetKind,
    },
    #[error("improvement proposal {proposal_id} was blocked by safety rules: {blocked_by:?}")]
    SafetyBlocked {
        proposal_id: String,
        blocked_by: Vec<String>,
    },
    #[error("failure learning record {record_id} was not found")]
    MissingFailureLearningRecord { record_id: String },
    #[error("improvement proposal {proposal_id} was not found")]
    ProposalNotFound { proposal_id: String },
    #[error("improvement proposal {proposal_id} is missing dry-run evidence")]
    MissingDryRun { proposal_id: String },
    #[error("improvement proposal {proposal_id} dry run failed")]
    DryRunFailed { proposal_id: String },
    #[error("improvement proposal {proposal_id} is missing passing test evidence")]
    MissingPassingTestEvidence { proposal_id: String },
    #[error("improvement proposal {proposal_id} has failing test evidence")]
    FailingTestEvidence { proposal_id: String },
    #[error("improvement proposal {proposal_id} is missing passing safety evidence")]
    MissingPassingSafetyEvidence { proposal_id: String },
    #[error("improvement proposal {proposal_id} has failing safety evidence")]
    FailingSafetyEvidence { proposal_id: String },
    #[error("improvement proposal {proposal_id} is missing operator approval")]
    MissingOperatorApproval { proposal_id: String },
    #[error("improvement proposal {proposal_id} was already applied")]
    AlreadyApplied { proposal_id: String },
    #[error("curator memory record id must not be empty")]
    EmptyCuratorMemoryRecordId,
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
        assert!(mission.tool_evidence.is_empty());
        assert!(mission.memory_hits.is_empty());
        assert!(mission.memory_recall.is_none());
        assert!(mission.verification_gates.is_empty());
        assert!(mission.checkpoints.is_empty());
        assert!(mission.artifacts.is_empty());
        assert!(mission.final_learning_output.is_none());
        assert!(mission.learning_records.is_empty());
        assert!(mission.improvement_proposals.is_empty());
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
                MissionCompletionBlocker::MissingMemoryRecallEvidence,
            ]
        );
        assert!(!mission.ready_for_completion());

        mission.plan_dag[1].status = "completed".to_string();
        mission.verification_gates[0].status = Some(MissionVerifierStatus::Passed);
        mission
            .record_no_memory_result("complete with proof", "no relevant prior memory", 610)
            .expect("memory proof");
        mission.final_learning_output = Some(MissionLearningOutput {
            summary: "learned to keep proof attached".to_string(),
            records: vec!["learning-record-1".to_string()],
            curator_recommendation: Some("retain skill prompt".to_string()),
        });

        assert!(mission.completion_blockers().is_empty());
        assert!(mission.ready_for_completion());
    }

    fn tool_evidence(
        mission_id: &str,
        plan_node_id: Option<&str>,
        tool_call_id: &str,
        tool_name: &str,
    ) -> MissionToolCallEvidence {
        MissionToolCallEvidence {
            tool_call_id: tool_call_id.to_string(),
            mission_id: mission_id.to_string(),
            plan_node_id: plan_node_id.map(str::to_string),
            tool_name: tool_name.to_string(),
            status: MissionToolCallStatus::Succeeded,
            started_unix_ms: 700,
            completed_unix_ms: Some(725),
            runtime_ms: Some(25),
            cost_usd: Some(0.01),
            summary: Some("tool completed".to_string()),
            artifact_ids: vec!["artifact-1".to_string()],
            verification_gate_ids: vec!["gate-1".to_string()],
            metadata: BTreeMap::from([("stdout_excerpt".to_string(), Value::String("ok".into()))]),
        }
    }

    #[test]
    fn mission_tool_evidence_preserves_attribution_and_links() {
        let mut mission = MissionSnapshot::new("mission-theta", "run tests", 700);
        mission.tool_budget.allowed_tools = vec!["bash".to_string()];
        mission.tool_budget.max_tool_calls = Some(2);
        let evidence = tool_evidence(&mission.mission_id, Some("verify"), "tool-1", "bash");

        mission
            .record_tool_call_evidence(evidence.clone())
            .expect("record tool evidence");

        assert_eq!(mission.tool_budget.consumed_tool_calls, 1);
        assert_eq!(mission.tool_budget.consumed_runtime_ms, 25);
        assert_eq!(mission.tool_budget.consumed_cost_usd, Some(0.01));
        assert_eq!(mission.tool_evidence, vec![evidence]);
        assert_eq!(
            mission.tool_evidence_for_verification_gate("gate-1"),
            vec![&mission.tool_evidence[0]]
        );
    }

    #[test]
    fn mission_tool_budget_blocks_disallowed_and_exhausted_calls() {
        let mut exhausted = MissionSnapshot::new("mission-iota", "respect budget", 800);
        exhausted.tool_budget.allowed_tools = vec!["bash".to_string()];
        exhausted.tool_budget.max_tool_calls = Some(1);
        exhausted
            .record_tool_call_evidence(tool_evidence(&exhausted.mission_id, None, "tool-1", "bash"))
            .expect("first call is inside budget");

        let before = exhausted.tool_evidence.clone();
        let error = exhausted
            .record_tool_call_evidence(tool_evidence(&exhausted.mission_id, None, "tool-2", "bash"))
            .expect_err("second call exceeds max calls");

        assert_eq!(
            error,
            MissionToolEvidenceError::Budget(MissionToolBudgetError::ToolCallBudgetExhausted {
                max_tool_calls: 1,
            })
        );
        assert_eq!(exhausted.tool_evidence, before);
        assert_eq!(exhausted.tool_budget.consumed_tool_calls, 1);

        let mut disallowed = MissionSnapshot::new("mission-kappa", "respect tool allowlist", 810);
        disallowed.tool_budget.allowed_tools = vec!["bash".to_string()];

        let error = disallowed
            .record_tool_call_evidence(tool_evidence(
                &disallowed.mission_id,
                None,
                "tool-1",
                "python",
            ))
            .expect_err("python is not allowed");

        assert_eq!(
            error,
            MissionToolEvidenceError::Budget(MissionToolBudgetError::ToolNotAllowed {
                tool_name: "python".to_string(),
            })
        );
        assert!(disallowed.tool_evidence.is_empty());
        assert_eq!(disallowed.tool_budget.consumed_tool_calls, 0);

        let mut runtime = MissionSnapshot::new("mission-runtime", "respect runtime", 820);
        runtime.tool_budget.max_runtime_ms = Some(20);
        let error = runtime
            .record_tool_call_evidence(tool_evidence(&runtime.mission_id, None, "tool-1", "bash"))
            .expect_err("runtime exceeds budget");
        assert_eq!(
            error,
            MissionToolEvidenceError::Budget(MissionToolBudgetError::RuntimeBudgetExhausted {
                max_runtime_ms: 20,
            })
        );
        assert!(runtime.tool_evidence.is_empty());

        let mut cost = MissionSnapshot::new("mission-cost", "respect cost", 830);
        cost.tool_budget.max_cost_usd = Some(0.005);
        let error = cost
            .record_tool_call_evidence(tool_evidence(&cost.mission_id, None, "tool-1", "bash"))
            .expect_err("cost exceeds budget");
        assert_eq!(
            error,
            MissionToolEvidenceError::Budget(MissionToolBudgetError::CostBudgetExhausted {
                max_cost_usd: 0.005,
            })
        );
        assert!(cost.tool_evidence.is_empty());
    }

    #[test]
    fn mission_completion_blocks_consumed_tools_without_ledger_evidence() {
        let mut mission = MissionSnapshot::new("mission-lambda", "complete with tool proof", 900);
        mission.plan_dag = vec![plan_node("verify", &[], "completed")];
        mission.verification_gates.push(MissionVerificationGate {
            id: "gate-1".to_string(),
            description: "tests passed".to_string(),
            status: Some(MissionVerifierStatus::Passed),
            evidence: BTreeMap::new(),
        });
        mission.final_learning_output = Some(MissionLearningOutput {
            summary: "tool trace proved".to_string(),
            records: vec!["learning-record-1".to_string()],
            curator_recommendation: None,
        });
        mission
            .record_no_memory_result("complete with tool proof", "no relevant prior memory", 910)
            .expect("memory proof");
        mission.tool_budget.consumed_tool_calls = 1;

        assert_eq!(
            mission.completion_blockers(),
            vec![MissionCompletionBlocker::MissingToolEvidence {
                consumed_tool_calls: 1,
                recorded_tool_calls: 0,
            }]
        );

        mission.tool_evidence.push(tool_evidence(
            &mission.mission_id,
            Some("verify"),
            "tool-1",
            "bash",
        ));

        assert!(mission.completion_blockers().is_empty());
        assert!(mission.ready_for_completion());
    }

    #[test]
    fn mission_memory_recall_preserves_hits_or_explicit_no_memory_result() {
        let mut mission = MissionSnapshot::new("mission-mu", "plan with memory proof", 1_000);
        let hit = MissionMemoryHit {
            key: "memory-1".to_string(),
            summary: "prior verifier recovery pattern".to_string(),
            score: Some(0.91),
            source_event_key: Some("source-memory-1".to_string()),
            plan_rationale: Some("use the recovery pattern for verify node".to_string()),
            used_in_plan_node_ids: vec!["verify".to_string()],
            metadata: BTreeMap::from([("rank".to_string(), Value::from(1))]),
        };

        mission
            .record_memory_hit(
                "verifier recovery",
                "memory shaped the verify plan node",
                1_010,
                hit.clone(),
            )
            .expect("record memory hit");

        assert_eq!(mission.memory_hits, vec![hit]);
        assert_eq!(
            mission.memory_recall,
            Some(MissionMemoryRecallEvidence {
                query: "verifier recovery".to_string(),
                status: MissionMemoryRecallStatus::UsedHits,
                checked_unix_ms: 1_010,
                rationale: "memory shaped the verify plan node".to_string(),
                hit_keys: vec!["memory-1".to_string()],
            })
        );

        let error = mission
            .record_no_memory_result(
                "verifier recovery",
                "search returned no relevant memories",
                1_020,
            )
            .expect_err("no-memory proof cannot conflict with recorded hits");

        assert_eq!(
            error,
            MissionMemoryEvidenceError::NoMemoryResultConflictsWithHits { recorded_hits: 1 }
        );

        let mut no_memory = MissionSnapshot::new("mission-nu", "plan without memory hits", 1_100);
        no_memory
            .record_no_memory_result(
                "new task shape",
                "search returned no relevant memories",
                1_110,
            )
            .expect("record no-memory proof");

        assert_eq!(
            no_memory.memory_recall,
            Some(MissionMemoryRecallEvidence {
                query: "new task shape".to_string(),
                status: MissionMemoryRecallStatus::NoRelevantMemory,
                checked_unix_ms: 1_110,
                rationale: "search returned no relevant memories".to_string(),
                hit_keys: Vec::new(),
            })
        );
    }

    #[test]
    fn mission_final_learning_output_writes_tau_memory_record_and_unblocks_completion() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = tau_memory::runtime::FileMemoryStore::new(temp.path());
        let scope = tau_memory::memory_contract::MemoryScope {
            workspace_id: "workspace".to_string(),
            channel_id: "mission".to_string(),
            actor_id: "tau".to_string(),
        };
        let mut mission =
            MissionSnapshot::new("mission-xi", "complete with learning memory", 1_200);
        mission.plan_dag = vec![plan_node("verify", &[], "completed")];
        mission.verification_gates.push(MissionVerificationGate {
            id: "gate-1".to_string(),
            description: "tests passed".to_string(),
            status: Some(MissionVerifierStatus::Passed),
            evidence: BTreeMap::new(),
        });
        mission
            .record_no_memory_result(
                "complete with learning memory",
                "no relevant prior memory",
                1_210,
            )
            .expect("memory proof");

        assert_eq!(
            mission.completion_blockers(),
            vec![MissionCompletionBlocker::MissingFinalLearningOutput]
        );

        let record = MissionLearningRecord {
            record_id: "learning-final-1".to_string(),
            mission_id: mission.mission_id.clone(),
            kind: MissionLearningRecordKind::Final,
            summary: "Keep mission proof fields attached before completion".to_string(),
            created_unix_ms: 1_220,
            curator_status: MissionCuratorReviewStatus::QueuedForReview,
            root_cause: None,
            evidence: vec!["gate-1 passed".to_string()],
            artifact_ids: vec!["artifact-1".to_string()],
            verification_gate_ids: vec!["gate-1".to_string()],
            rollback_plan: Some("remove the queued learning record".to_string()),
            metadata: BTreeMap::new(),
        };

        let write = mission
            .write_final_learning_output(
                &store,
                &scope,
                MissionLearningOutput {
                    summary: "mission finished with proof".to_string(),
                    records: Vec::new(),
                    curator_recommendation: Some("review for skill prompt update".to_string()),
                },
                record.clone(),
            )
            .expect("write final learning");

        assert!(write.created);
        assert_eq!(mission.learning_records, vec![record]);
        assert_eq!(
            mission
                .final_learning_output
                .as_ref()
                .map(|output| &output.records),
            Some(&vec!["learning-final-1".to_string()])
        );

        let saved = store
            .read_entry("learning-final-1", None)
            .expect("read final learning")
            .expect("final learning stored");
        assert_eq!(
            saved.entry.summary,
            "Keep mission proof fields attached before completion"
        );
        assert!(saved.entry.tags.contains(&"mission_learning".to_string()));
        assert!(saved
            .entry
            .tags
            .contains(&"curator:queued_for_review".to_string()));
        assert_eq!(saved.memory_type, tau_memory::runtime::MemoryType::Decision);
        assert!(mission.ready_for_completion());
    }

    #[test]
    fn mission_failure_learning_record_requires_recovery_and_writes_curator_queue() {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = tau_memory::runtime::FileMemoryStore::new(temp.path());
        let scope = tau_memory::memory_contract::MemoryScope {
            workspace_id: "workspace".to_string(),
            channel_id: "mission".to_string(),
            actor_id: "tau".to_string(),
        };
        let mut mission = MissionSnapshot::new("mission-omicron", "learn from failure", 1_300);
        let record = MissionLearningRecord {
            record_id: "learning-failure-1".to_string(),
            mission_id: mission.mission_id.clone(),
            kind: MissionLearningRecordKind::Failure,
            summary: "Verifier blocked because the harness lacked memory proof".to_string(),
            created_unix_ms: 1_320,
            curator_status: MissionCuratorReviewStatus::QueuedForReview,
            root_cause: Some("missing memory proof gate".to_string()),
            evidence: vec!["completion blocker: missing memory proof".to_string()],
            artifact_ids: Vec::new(),
            verification_gate_ids: Vec::new(),
            rollback_plan: Some("drop the learning if benchmark replay disagrees".to_string()),
            metadata: BTreeMap::new(),
        };

        let error = mission
            .write_failure_learning_record(&store, &scope, record.clone())
            .expect_err("failure learning requires recovery state");

        assert_eq!(error, MissionLearningRecordError::MissingRecoveryState);

        mission
            .transition_to(MissionLifecycleStatus::Planned, 1_305)
            .expect("planned");
        mission
            .transition_to(MissionLifecycleStatus::Executing, 1_310)
            .expect("executing");
        mission
            .block_for_recovery("verifier blocked", Some("queue failure learning"), 1_315)
            .expect("blocked");

        mission
            .write_failure_learning_record(&store, &scope, record.clone())
            .expect("write failure learning");

        assert_eq!(mission.learning_records, vec![record]);
        let saved = store
            .read_entry("learning-failure-1", None)
            .expect("read failure learning")
            .expect("failure learning stored");
        assert!(saved.entry.tags.contains(&"kind:failure".to_string()));
        assert!(saved
            .entry
            .facts
            .contains(&"root_cause: missing memory proof gate".to_string()));
        assert_eq!(
            saved.memory_type,
            tau_memory::runtime::MemoryType::Observation
        );
    }

    fn failure_learning_record(mission_id: &str, record_id: &str) -> MissionLearningRecord {
        MissionLearningRecord {
            record_id: record_id.to_string(),
            mission_id: mission_id.to_string(),
            kind: MissionLearningRecordKind::Failure,
            summary: "Benchmark failure exposed a missing recovery prompt".to_string(),
            created_unix_ms: 1_600,
            curator_status: MissionCuratorReviewStatus::QueuedForReview,
            root_cause: Some("missing recovery prompt".to_string()),
            evidence: vec!["benchmark repo_bugfix failed verification".to_string()],
            artifact_ids: Vec::new(),
            verification_gate_ids: Vec::new(),
            rollback_plan: Some("remove the prompt change".to_string()),
            metadata: BTreeMap::new(),
        }
    }

    fn skill_improvement_proposal(
        mission_id: &str,
        proposal_id: &str,
        source_learning_record_id: &str,
    ) -> MissionImprovementProposal {
        MissionImprovementProposal {
            proposal_id: proposal_id.to_string(),
            mission_id: mission_id.to_string(),
            source_learning_record_id: source_learning_record_id.to_string(),
            trigger_kind: MissionImprovementTriggerKind::BenchmarkFailure,
            target_kind: MissionImprovementTargetKind::Skill,
            target_path: "skills/recovery/SKILL.md".to_string(),
            patch_summary: "Add a recovery prompt checklist".to_string(),
            rationale: "The benchmark failed because recovery skipped memory proof".to_string(),
            rollback_plan: "Remove the checklist section".to_string(),
            proposed_unix_ms: 1_610,
            status: MissionImprovementProposalStatus::Proposed,
            dry_run: None,
            tests: Vec::new(),
            safety_checks: Vec::new(),
            approval: None,
            applied_unix_ms: None,
            curator_memory_record_id: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn mission_improvement_proposal_is_limited_to_safe_skill_config_prompt_targets() {
        let mut mission = MissionSnapshot::new("mission-pi", "improve conservatively", 1_600);
        mission.learning_records.push(failure_learning_record(
            &mission.mission_id,
            "learning-failure-2",
        ));

        let proposal =
            skill_improvement_proposal(&mission.mission_id, "improvement-1", "learning-failure-2");

        mission
            .record_improvement_proposal_from_failure(proposal.clone())
            .expect("safe skill proposal should be recorded");

        assert_eq!(mission.improvement_proposals, vec![proposal]);

        let mut source_code = skill_improvement_proposal(
            &mission.mission_id,
            "improvement-source",
            "learning-failure-2",
        );
        source_code.target_kind = MissionImprovementTargetKind::SourceCode;
        source_code.target_path = "crates/tau-agent-core/src/lib.rs".to_string();

        let error = mission
            .record_improvement_proposal_from_failure(source_code)
            .expect_err("source-code proposal should be rejected");

        assert_eq!(
            error,
            MissionImprovementProposalError::UnsafeTargetKind {
                target_kind: MissionImprovementTargetKind::SourceCode,
            }
        );

        let mut blocked_path = skill_improvement_proposal(
            &mission.mission_id,
            "improvement-blocked-path",
            "learning-failure-2",
        );
        blocked_path.target_path = "crates/tau-agent-core/src/lib.rs".to_string();

        let error = mission
            .record_improvement_proposal_from_failure(blocked_path)
            .expect_err("source path should be blocked by safety rules");

        assert_eq!(
            error,
            MissionImprovementProposalError::SafetyBlocked {
                proposal_id: "improvement-blocked-path".to_string(),
                blocked_by: vec!["self_mod_block_source_default".to_string()],
            }
        );
    }

    #[test]
    fn mission_improvement_approval_requires_dry_run_tests_safety_and_operator() {
        let mut mission = MissionSnapshot::new("mission-rho", "approve conservatively", 1_700);
        mission.learning_records.push(failure_learning_record(
            &mission.mission_id,
            "learning-failure-3",
        ));
        mission
            .record_improvement_proposal_from_failure(skill_improvement_proposal(
                &mission.mission_id,
                "improvement-2",
                "learning-failure-3",
            ))
            .expect("proposal");

        let approval = MissionOperatorApproval {
            approval_id: "approval-1".to_string(),
            operator_id: "operator".to_string(),
            approved_unix_ms: 1_740,
            summary: "Approved after dry-run evidence".to_string(),
        };

        let error = mission
            .approve_improvement_proposal("improvement-2", approval.clone())
            .expect_err("approval needs dry-run proof");
        assert_eq!(
            error,
            MissionImprovementProposalError::MissingDryRun {
                proposal_id: "improvement-2".to_string(),
            }
        );

        mission
            .record_improvement_dry_run(
                "improvement-2",
                MissionImprovementDryRun {
                    ran_unix_ms: 1_710,
                    passed: true,
                    summary: "patch preview applied cleanly".to_string(),
                    artifact_ids: vec!["dry-run-artifact".to_string()],
                    metadata: BTreeMap::new(),
                },
            )
            .expect("dry-run");

        let error = mission
            .approve_improvement_proposal("improvement-2", approval.clone())
            .expect_err("approval needs test proof");
        assert_eq!(
            error,
            MissionImprovementProposalError::MissingPassingTestEvidence {
                proposal_id: "improvement-2".to_string(),
            }
        );

        mission
            .record_improvement_test_evidence(
                "improvement-2",
                MissionImprovementTestEvidence {
                    command: "cargo test -p tau-agent-core mission --lib".to_string(),
                    status: MissionImprovementEvidenceStatus::Passed,
                    ran_unix_ms: 1_720,
                    summary: "mission tests passed".to_string(),
                    artifact_ids: Vec::new(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("test evidence");

        let error = mission
            .approve_improvement_proposal("improvement-2", approval.clone())
            .expect_err("approval needs safety proof");
        assert_eq!(
            error,
            MissionImprovementProposalError::MissingPassingSafetyEvidence {
                proposal_id: "improvement-2".to_string(),
            }
        );

        mission
            .record_improvement_safety_check(
                "improvement-2",
                MissionImprovementSafetyCheck {
                    check_id: "self-mod-policy".to_string(),
                    status: MissionImprovementEvidenceStatus::Passed,
                    ran_unix_ms: 1_730,
                    summary: "target stayed within skills".to_string(),
                    blocked_by: Vec::new(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("safety evidence");

        mission
            .approve_improvement_proposal("improvement-2", approval.clone())
            .expect("approval eligible");

        assert_eq!(
            mission.improvement_proposals[0].approval,
            Some(approval.clone())
        );
        assert_eq!(
            mission.improvement_proposals[0].status,
            MissionImprovementProposalStatus::Approved
        );

        let mut unapproved =
            MissionSnapshot::new("mission-sigma", "apply requires approval", 1_800);
        unapproved.learning_records.push(failure_learning_record(
            &unapproved.mission_id,
            "learning-failure-4",
        ));
        unapproved
            .record_improvement_proposal_from_failure(skill_improvement_proposal(
                &unapproved.mission_id,
                "improvement-3",
                "learning-failure-4",
            ))
            .expect("proposal");

        let error = unapproved
            .apply_approved_improvement("improvement-3", 1_850, "curator-memory-1")
            .expect_err("apply requires approval");
        assert_eq!(
            error,
            MissionImprovementProposalError::MissingOperatorApproval {
                proposal_id: "improvement-3".to_string(),
            }
        );
    }

    #[test]
    fn mission_improvement_apply_updates_curator_metadata() {
        let mut mission = MissionSnapshot::new("mission-tau", "apply accepted improvement", 1_900);
        mission.learning_records.push(failure_learning_record(
            &mission.mission_id,
            "learning-failure-5",
        ));
        mission
            .record_improvement_proposal_from_failure(skill_improvement_proposal(
                &mission.mission_id,
                "improvement-4",
                "learning-failure-5",
            ))
            .expect("proposal");
        mission
            .record_improvement_dry_run(
                "improvement-4",
                MissionImprovementDryRun {
                    ran_unix_ms: 1_910,
                    passed: true,
                    summary: "preview succeeded".to_string(),
                    artifact_ids: Vec::new(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("dry-run");
        mission
            .record_improvement_test_evidence(
                "improvement-4",
                MissionImprovementTestEvidence {
                    command: "cargo test -p tau-agent-core mission --lib".to_string(),
                    status: MissionImprovementEvidenceStatus::Passed,
                    ran_unix_ms: 1_920,
                    summary: "tests passed".to_string(),
                    artifact_ids: Vec::new(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("test evidence");
        mission
            .record_improvement_safety_check(
                "improvement-4",
                MissionImprovementSafetyCheck {
                    check_id: "self-mod-policy".to_string(),
                    status: MissionImprovementEvidenceStatus::Passed,
                    ran_unix_ms: 1_930,
                    summary: "safety passed".to_string(),
                    blocked_by: Vec::new(),
                    metadata: BTreeMap::new(),
                },
            )
            .expect("safety evidence");
        mission
            .approve_improvement_proposal(
                "improvement-4",
                MissionOperatorApproval {
                    approval_id: "approval-4".to_string(),
                    operator_id: "operator".to_string(),
                    approved_unix_ms: 1_940,
                    summary: "approved".to_string(),
                },
            )
            .expect("approve");

        mission
            .apply_approved_improvement("improvement-4", 1_950, "curator-memory-4")
            .expect("apply");

        assert_eq!(
            mission.improvement_proposals[0].status,
            MissionImprovementProposalStatus::Applied
        );
        assert_eq!(
            mission.improvement_proposals[0].applied_unix_ms,
            Some(1_950)
        );
        assert_eq!(
            mission.improvement_proposals[0].curator_memory_record_id,
            Some("curator-memory-4".to_string())
        );
        assert_eq!(
            mission.learning_records[0].curator_status,
            MissionCuratorReviewStatus::Applied
        );
        assert_eq!(
            mission.learning_records[0]
                .metadata
                .get("accepted_improvement_proposal_id"),
            Some(&Value::String("improvement-4".to_string()))
        );
        assert_eq!(
            mission.learning_records[0]
                .metadata
                .get("accepted_improvement_curator_memory_record_id"),
            Some(&Value::String("curator-memory-4".to_string()))
        );
    }
}
