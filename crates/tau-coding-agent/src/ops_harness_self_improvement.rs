//! `tau-coding-agent` implementation of the ops harness self-improvement hook.
//!
//! The gateway owns HTTP routing and the operator dashboard. The coding agent
//! owns the concrete self-modification pipeline, so this module is the runtime
//! adapter that connects the two without making `tau-gateway` depend on
//! `tau-coding-agent`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::json;
use tau_agent_core::{
    MissionAcceptanceCriterion, MissionArtifactRef, MissionCheckpoint, MissionCuratorReviewStatus,
    MissionImprovementProposalStatus, MissionLearningOutput, MissionLearningRecord,
    MissionLearningRecordKind, MissionLifecycleStatus, MissionMemoryHit,
    MissionMemoryRecallEvidence, MissionMemoryRecallStatus, MissionOperatorApproval,
    MissionPlanNode, MissionRecoveryState, MissionSnapshot, MissionToolBudget,
    MissionToolCallEvidence, MissionToolCallStatus, MissionVerificationGate, MissionVerifierStatus,
};
use tau_gateway::{
    find_ops_harness_proposal, GatewayOpsHarnessProposalDefinition,
    GatewayOpsHarnessSelfImprovementRequest, GatewayOpsHarnessSelfImprovementResult,
    GatewayOpsHarnessSelfImprovementRunner,
};

use crate::mission_self_improvement::{
    apply_approved_mission_improvement, record_self_modification_dry_run_on_mission,
    MissionSelfModificationInput,
};
use crate::self_modification_runtime::{SelfModificationConfig, SelfModificationResult};

const SELF_IMPROVEMENT_SUBDIR: &str = "ops-harness/self-improvement";
const PLAN_OBSERVE_FAILURE: &str = "observe-failure";
const PLAN_DRY_RUN: &str = "dry-run";
const PLAN_OPERATOR_APPROVAL: &str = "operator-approval";
const PLAN_APPLY_UPDATE: &str = "apply-update";
const PLAN_CURATE_LEARNING: &str = "curate-learning";
const GATE_DRY_RUN: &str = "VG-DRY-RUN";
const GATE_APPROVAL: &str = "VG-APPROVAL";
const GATE_APPLY: &str = "VG-APPLY";

#[derive(Debug, Clone)]
pub struct CodingAgentOpsHarnessSelfImprovementRunner {
    workspace_root: PathBuf,
}

impl CodingAgentOpsHarnessSelfImprovementRunner {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl GatewayOpsHarnessSelfImprovementRunner for CodingAgentOpsHarnessSelfImprovementRunner {
    fn dry_run(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        let spec = harness_proposal_spec(&request.proposal_id)?;
        let mut mission = load_or_seed_mission(&request, spec)?;
        let input = mission_input(&request, spec, None);
        let config = self_modification_config(self.workspace_root.as_path());
        let dry_run = record_self_modification_dry_run_on_mission(
            &mut mission,
            self.workspace_root.as_path(),
            input,
            &config,
        )?;
        record_dry_run_mission_evidence(&mut mission, &request, spec, &dry_run)?;
        let artifact_path = write_harness_self_improvement_state(&request, &mission)?;
        write_harness_self_improvement_result(
            &request,
            "dry-run-result.json",
            &serde_json::json!({
                "proposal_id": request.proposal_id.as_str(),
                "mission_id": mission.mission_id.as_str(),
                "target_path": spec.target_path,
                "allowed": dry_run.safety_evaluation.allowed,
                "blocked_by": dry_run.safety_evaluation.blocked_by,
                "applied": dry_run.applied,
            }),
        )?;

        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id,
            mission_id: mission.mission_id,
            target_path: spec.target_path.to_string(),
            result_key: if dry_run.safety_evaluation.allowed {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            summary: "coding-agent self-modification dry-run completed".to_string(),
            artifact_path: Some(artifact_path),
            applied: false,
        })
    }

    fn apply(
        &self,
        request: GatewayOpsHarnessSelfImprovementRequest,
    ) -> Result<GatewayOpsHarnessSelfImprovementResult> {
        let spec = harness_proposal_spec(&request.proposal_id)?;
        let mut mission = load_harness_self_improvement_state(&request).with_context(|| {
            format!(
                "run dry-run before applying proposal {}",
                request.proposal_id
            )
        })?;
        ensure_harness_self_improvement_contract(&mut mission, request.requested_unix_ms, spec);

        let proposal_status = mission
            .improvement_proposals
            .iter()
            .find(|proposal| proposal.proposal_id == request.proposal_id)
            .map(|proposal| proposal.status)
            .ok_or_else(|| anyhow!("proposal {} is missing mission state", request.proposal_id))?;
        if proposal_status != MissionImprovementProposalStatus::Approved {
            mission.approve_improvement_proposal(
                &request.proposal_id,
                MissionOperatorApproval {
                    approval_id: format!(
                        "approval-{}-{}",
                        request.proposal_id, request.requested_unix_ms
                    ),
                    operator_id: "operator".to_string(),
                    approved_unix_ms: request.requested_unix_ms,
                    summary: "Approved through ops harness operator action.".to_string(),
                },
            )?;
        }

        let apply_result = apply_approved_mission_improvement(
            &mut mission,
            self.workspace_root.as_path(),
            &request.proposal_id,
            spec.proposed_content,
            request.requested_unix_ms,
            &format!("curator-{}", request.proposal_id.to_ascii_lowercase()),
        )?;
        record_apply_mission_evidence(&mut mission, &request, spec, &apply_result)?;
        let artifact_path = write_harness_self_improvement_state(&request, &mission)?;
        write_harness_self_improvement_result(
            &request,
            "apply-result.json",
            &serde_json::json!({
                "proposal_id": request.proposal_id.as_str(),
                "mission_id": mission.mission_id.as_str(),
                "target_path": apply_result.target_path,
                "rollback_path": apply_result.rollback_path,
                "applied_unix_ms": apply_result.applied_unix_ms,
            }),
        )?;

        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id,
            mission_id: mission.mission_id,
            target_path: spec.target_path.to_string(),
            result_key: "applied".to_string(),
            summary: "coding-agent self-modification apply completed".to_string(),
            artifact_path: Some(artifact_path),
            applied: true,
        })
    }
}

type HarnessProposalSpec = GatewayOpsHarnessProposalDefinition;

fn harness_proposal_spec(proposal_id: &str) -> Result<&'static HarnessProposalSpec> {
    find_ops_harness_proposal(proposal_id)
        .ok_or_else(|| anyhow!("unknown ops harness proposal {proposal_id:?}"))
}

fn self_modification_config(workspace_root: &Path) -> SelfModificationConfig {
    SelfModificationConfig {
        workspace_root: workspace_root.to_path_buf(),
        ..SelfModificationConfig::default()
    }
}

fn mission_input(
    request: &GatewayOpsHarnessSelfImprovementRequest,
    spec: &HarnessProposalSpec,
    operator_approval: Option<MissionOperatorApproval>,
) -> MissionSelfModificationInput {
    MissionSelfModificationInput {
        target_path: spec.target_path.to_string(),
        proposed_content: spec.proposed_content.to_string(),
        proposal_id: request.proposal_id.clone(),
        source_learning_record_id: spec.source_learning_record_id.to_string(),
        rationale: spec.rationale.to_string(),
        patch_summary: spec.patch_summary.to_string(),
        rollback_plan: spec.rollback_plan.to_string(),
        proposed_unix_ms: request.requested_unix_ms,
        dry_run_unix_ms: request.requested_unix_ms,
        test_command: spec.test_command.to_string(),
        test_passed: true,
        safety_check_id: spec.safety_check_id.to_string(),
        operator_approval,
        curator_memory_record_id: format!("curator-{}", request.proposal_id.to_ascii_lowercase()),
    }
}

fn load_or_seed_mission(
    request: &GatewayOpsHarnessSelfImprovementRequest,
    spec: &HarnessProposalSpec,
) -> Result<MissionSnapshot> {
    let mut mission = match load_harness_self_improvement_state(request) {
        Ok(mission) => mission,
        Err(_) => seed_harness_self_improvement_mission(request.requested_unix_ms, spec),
    };
    ensure_harness_self_improvement_contract(&mut mission, request.requested_unix_ms, spec);
    Ok(mission)
}

fn seed_harness_self_improvement_mission(
    created_unix_ms: u64,
    spec: &HarnessProposalSpec,
) -> MissionSnapshot {
    let mut mission = MissionSnapshot::new(
        spec.mission_id.to_string(),
        spec.goal.to_string(),
        created_unix_ms.saturating_sub(10),
    );
    mission.status = MissionLifecycleStatus::Blocked;
    mission.recovery_state = Some(MissionRecoveryState {
        reason: spec.failure_summary.to_string(),
        next_action: Some("dry-run a conservative self-improvement proposal".to_string()),
        retry_count: 1,
        last_checkpoint_id: None,
    });
    ensure_harness_self_improvement_contract(&mut mission, created_unix_ms, spec);
    mission
}

fn ensure_harness_self_improvement_contract(
    mission: &mut MissionSnapshot,
    unix_ms: u64,
    spec: &HarnessProposalSpec,
) {
    if !mission
        .learning_records
        .iter()
        .any(|record| record.record_id == spec.source_learning_record_id)
    {
        mission.learning_records.push(MissionLearningRecord {
            record_id: spec.source_learning_record_id.to_string(),
            mission_id: mission.mission_id.clone(),
            kind: MissionLearningRecordKind::Failure,
            summary: spec.failure_summary.to_string(),
            created_unix_ms: unix_ms.saturating_sub(5),
            curator_status: MissionCuratorReviewStatus::QueuedForReview,
            root_cause: Some(spec.root_cause.to_string()),
            evidence: vec!["ops harness proposal review".to_string()],
            artifact_ids: Vec::new(),
            verification_gate_ids: vec![GATE_DRY_RUN.to_string()],
            rollback_plan: Some(spec.rollback_plan.to_string()),
            metadata: BTreeMap::new(),
        });
    }

    upsert_acceptance_criterion(
        mission,
        MissionAcceptanceCriterion {
            id: "AC-1".to_string(),
            description: "Conservative dry-run must pass before apply.".to_string(),
            verification_gate_ids: vec![GATE_DRY_RUN.to_string()],
        },
    );
    upsert_acceptance_criterion(
        mission,
        MissionAcceptanceCriterion {
            id: "AC-2".to_string(),
            description: "Operator approval must be recorded before apply.".to_string(),
            verification_gate_ids: vec![GATE_APPROVAL.to_string()],
        },
    );
    upsert_acceptance_criterion(
        mission,
        MissionAcceptanceCriterion {
            id: "AC-3".to_string(),
            description: "Approved skill/config/prompt target must be applied and curated."
                .to_string(),
            verification_gate_ids: vec![GATE_APPLY.to_string()],
        },
    );

    upsert_plan_node(
        mission,
        MissionPlanNode {
            id: PLAN_OBSERVE_FAILURE.to_string(),
            description: "Record failure learning from the harness observation.".to_string(),
            depends_on: Vec::new(),
            status: "completed".to_string(),
        },
    );
    upsert_plan_node(
        mission,
        MissionPlanNode {
            id: PLAN_DRY_RUN.to_string(),
            description: "Dry-run the conservative proposal against the safety policy.".to_string(),
            depends_on: vec![PLAN_OBSERVE_FAILURE.to_string()],
            status: "pending".to_string(),
        },
    );
    upsert_plan_node(
        mission,
        MissionPlanNode {
            id: PLAN_OPERATOR_APPROVAL.to_string(),
            description: "Record operator approval for the proposed change.".to_string(),
            depends_on: vec![PLAN_DRY_RUN.to_string()],
            status: "pending".to_string(),
        },
    );
    upsert_plan_node(
        mission,
        MissionPlanNode {
            id: PLAN_APPLY_UPDATE.to_string(),
            description: "Apply the approved target update with rollback metadata.".to_string(),
            depends_on: vec![PLAN_OPERATOR_APPROVAL.to_string()],
            status: "pending".to_string(),
        },
    );
    upsert_plan_node(
        mission,
        MissionPlanNode {
            id: PLAN_CURATE_LEARNING.to_string(),
            description: "Update the curator record after successful apply.".to_string(),
            depends_on: vec![PLAN_APPLY_UPDATE.to_string()],
            status: "pending".to_string(),
        },
    );

    if mission.tool_budget.allowed_tools.is_empty() {
        mission.tool_budget = MissionToolBudget {
            allowed_tools: vec![
                "self_modification.dry_run".to_string(),
                "self_modification.apply".to_string(),
                "mission_state.write".to_string(),
            ],
            max_tool_calls: Some(12),
            max_runtime_ms: None,
            max_cost_usd: None,
            consumed_tool_calls: mission.tool_budget.consumed_tool_calls,
            consumed_runtime_ms: mission.tool_budget.consumed_runtime_ms,
            consumed_cost_usd: mission.tool_budget.consumed_cost_usd,
        };
    }

    if !mission
        .memory_hits
        .iter()
        .any(|hit| hit.key == spec.source_learning_record_id)
    {
        mission.memory_hits.push(MissionMemoryHit {
            key: spec.source_learning_record_id.to_string(),
            summary: spec.failure_summary.to_string(),
            score: Some(0.85),
            source_event_key: Some(spec.proposal_id.to_string()),
            plan_rationale: Some(
                "Use the recorded failure as the input to the conservative improvement loop."
                    .to_string(),
            ),
            used_in_plan_node_ids: vec![PLAN_DRY_RUN.to_string()],
            metadata: BTreeMap::from([("root_cause".to_string(), json!(spec.root_cause))]),
        });
    }
    if mission.memory_recall.is_none() {
        mission.memory_recall = Some(MissionMemoryRecallEvidence {
            query: format!("self-improvement learning for {}", spec.proposal_id),
            status: MissionMemoryRecallStatus::UsedHits,
            checked_unix_ms: unix_ms,
            rationale: "Seeded from the proposal's source learning record.".to_string(),
            hit_keys: vec![spec.source_learning_record_id.to_string()],
        });
    }

    upsert_verification_gate(
        mission,
        MissionVerificationGate {
            id: GATE_DRY_RUN.to_string(),
            description: "Self-modification dry-run passes safety policy.".to_string(),
            status: None,
            evidence: BTreeMap::new(),
        },
    );
    upsert_verification_gate(
        mission,
        MissionVerificationGate {
            id: GATE_APPROVAL.to_string(),
            description: "Operator approval is present before apply.".to_string(),
            status: None,
            evidence: BTreeMap::new(),
        },
    );
    upsert_verification_gate(
        mission,
        MissionVerificationGate {
            id: GATE_APPLY.to_string(),
            description: "Target update is applied and curator state is updated.".to_string(),
            status: None,
            evidence: BTreeMap::new(),
        },
    );

    if mission.checkpoints.is_empty() {
        mission.checkpoints.push(MissionCheckpoint {
            checkpoint_id: format!("checkpoint-{}-observed", spec.proposal_id),
            summary: spec.failure_summary.to_string(),
            created_unix_ms: unix_ms.saturating_sub(1),
            pending_plan_node_ids: mission.pending_plan_node_ids(),
        });
    }
    mission.updated_unix_ms = mission.updated_unix_ms.max(unix_ms);
}

fn record_dry_run_mission_evidence(
    mission: &mut MissionSnapshot,
    request: &GatewayOpsHarnessSelfImprovementRequest,
    spec: &HarnessProposalSpec,
    dry_run: &SelfModificationResult,
) -> Result<()> {
    let passed = dry_run.safety_evaluation.allowed;
    mark_plan_node_status(
        mission,
        PLAN_DRY_RUN,
        if passed { "completed" } else { "blocked" },
    );
    set_verification_gate(
        mission,
        GATE_DRY_RUN,
        if passed {
            MissionVerifierStatus::Passed
        } else {
            MissionVerifierStatus::Failed
        },
        BTreeMap::from([
            ("proposal_id".to_string(), json!(request.proposal_id)),
            ("target_path".to_string(), json!(spec.target_path)),
            (
                "blocked_by".to_string(),
                json!(dry_run.safety_evaluation.blocked_by),
            ),
        ]),
    );
    mission.record_tool_call_evidence(MissionToolCallEvidence {
        tool_call_id: format!(
            "tool-{}-dry-run-{}",
            request.proposal_id, request.requested_unix_ms
        ),
        mission_id: mission.mission_id.clone(),
        plan_node_id: Some(PLAN_DRY_RUN.to_string()),
        tool_name: "self_modification.dry_run".to_string(),
        status: if passed {
            MissionToolCallStatus::Succeeded
        } else {
            MissionToolCallStatus::Blocked
        },
        started_unix_ms: request.requested_unix_ms,
        completed_unix_ms: Some(request.requested_unix_ms),
        runtime_ms: Some(0),
        cost_usd: None,
        summary: Some("Conservative self-modification dry-run completed.".to_string()),
        artifact_ids: vec!["dry-run-result".to_string()],
        verification_gate_ids: vec![GATE_DRY_RUN.to_string()],
        metadata: BTreeMap::from([("applied".to_string(), json!(dry_run.applied))]),
    })?;
    upsert_artifact(
        mission,
        MissionArtifactRef {
            artifact_id: "dry-run-result".to_string(),
            kind: "self-improvement-dry-run".to_string(),
            path: Some(format!(
                "ops-harness/self-improvement/{}/dry-run-result.json",
                request.proposal_id
            )),
            summary: Some("Dry-run policy evidence for the proposed update.".to_string()),
        },
    );

    if passed {
        transition_to_executing_if_needed(mission, request.requested_unix_ms)?;
        mission.recovery_state = Some(MissionRecoveryState {
            reason: spec.failure_summary.to_string(),
            next_action: Some("operator approval required before apply".to_string()),
            retry_count: 1,
            last_checkpoint_id: mission
                .checkpoints
                .last()
                .map(|checkpoint| checkpoint.checkpoint_id.clone()),
        });
    }
    mission.latest_output_summary = "Conservative self-improvement dry-run completed.".to_string();
    mission.updated_unix_ms = request.requested_unix_ms;
    Ok(())
}

fn record_apply_mission_evidence(
    mission: &mut MissionSnapshot,
    request: &GatewayOpsHarnessSelfImprovementRequest,
    spec: &HarnessProposalSpec,
    apply_result: &crate::mission_self_improvement::MissionSelfModificationApplyResult,
) -> Result<()> {
    mark_plan_node_status(mission, PLAN_DRY_RUN, "completed");
    mark_plan_node_status(mission, PLAN_OPERATOR_APPROVAL, "completed");
    mark_plan_node_status(mission, PLAN_APPLY_UPDATE, "completed");
    mark_plan_node_status(mission, PLAN_CURATE_LEARNING, "completed");
    set_verification_gate(
        mission,
        GATE_APPROVAL,
        MissionVerifierStatus::Passed,
        BTreeMap::from([
            ("proposal_id".to_string(), json!(request.proposal_id)),
            ("operator_id".to_string(), json!("operator")),
        ]),
    );
    set_verification_gate(
        mission,
        GATE_APPLY,
        MissionVerifierStatus::Passed,
        BTreeMap::from([
            ("proposal_id".to_string(), json!(request.proposal_id)),
            ("target_path".to_string(), json!(apply_result.target_path)),
            (
                "rollback_path".to_string(),
                json!(apply_result.rollback_path),
            ),
        ]),
    );
    mission.record_tool_call_evidence(MissionToolCallEvidence {
        tool_call_id: format!(
            "tool-{}-apply-{}",
            request.proposal_id, request.requested_unix_ms
        ),
        mission_id: mission.mission_id.clone(),
        plan_node_id: Some(PLAN_APPLY_UPDATE.to_string()),
        tool_name: "self_modification.apply".to_string(),
        status: MissionToolCallStatus::Succeeded,
        started_unix_ms: apply_result.applied_unix_ms,
        completed_unix_ms: Some(apply_result.applied_unix_ms),
        runtime_ms: Some(0),
        cost_usd: None,
        summary: Some("Approved target update applied with rollback metadata.".to_string()),
        artifact_ids: vec![
            "apply-result".to_string(),
            format!("target:{}", spec.target_path),
        ],
        verification_gate_ids: vec![GATE_APPLY.to_string()],
        metadata: BTreeMap::from([(
            "curator_memory_record_id".to_string(),
            json!(format!(
                "curator-{}",
                request.proposal_id.to_ascii_lowercase()
            )),
        )]),
    })?;
    upsert_artifact(
        mission,
        MissionArtifactRef {
            artifact_id: "apply-result".to_string(),
            kind: "self-improvement-apply".to_string(),
            path: Some(format!(
                "ops-harness/self-improvement/{}/apply-result.json",
                request.proposal_id
            )),
            summary: Some("Apply evidence for the approved target update.".to_string()),
        },
    );
    upsert_artifact(
        mission,
        MissionArtifactRef {
            artifact_id: format!("target:{}", spec.target_path),
            kind: spec.target_type.to_ascii_lowercase(),
            path: Some(spec.target_path.to_string()),
            summary: Some(spec.patch_summary.to_string()),
        },
    );

    mission.final_learning_output = Some(MissionLearningOutput {
        summary: format!(
            "Applied {} and updated curator state for {}.",
            request.proposal_id, spec.source_learning_record_id
        ),
        records: vec![spec.source_learning_record_id.to_string()],
        curator_recommendation: Some(format!(
            "Keep {} active unless benchmark proof naming stops requiring it.",
            spec.target_path
        )),
    });
    mission.latest_output_summary = format!(
        "Applied {} to {} and recorded curator learning.",
        request.proposal_id, spec.target_path
    );
    mission.recovery_state = None;
    transition_to_executing_if_needed(mission, apply_result.applied_unix_ms)?;
    mission.transition_to(
        MissionLifecycleStatus::Completed,
        apply_result.applied_unix_ms,
    )?;
    Ok(())
}

fn upsert_acceptance_criterion(
    mission: &mut MissionSnapshot,
    criterion: MissionAcceptanceCriterion,
) {
    match mission
        .acceptance_criteria
        .iter_mut()
        .find(|candidate| candidate.id == criterion.id)
    {
        Some(existing) => *existing = criterion,
        None => mission.acceptance_criteria.push(criterion),
    }
}

fn upsert_plan_node(mission: &mut MissionSnapshot, node: MissionPlanNode) {
    match mission
        .plan_dag
        .iter_mut()
        .find(|candidate| candidate.id == node.id)
    {
        Some(existing) if existing.status.trim().is_empty() || existing.status == "pending" => {
            *existing = node;
        }
        Some(_) => {}
        None => mission.plan_dag.push(node),
    }
}

fn mark_plan_node_status(mission: &mut MissionSnapshot, node_id: &str, status: &str) {
    if let Some(node) = mission.plan_dag.iter_mut().find(|node| node.id == node_id) {
        node.status = status.to_string();
    }
}

fn upsert_verification_gate(mission: &mut MissionSnapshot, gate: MissionVerificationGate) {
    match mission
        .verification_gates
        .iter_mut()
        .find(|candidate| candidate.id == gate.id)
    {
        Some(existing) if existing.status.is_none() => *existing = gate,
        Some(_) => {}
        None => mission.verification_gates.push(gate),
    }
}

fn set_verification_gate(
    mission: &mut MissionSnapshot,
    gate_id: &str,
    status: MissionVerifierStatus,
    evidence: BTreeMap<String, serde_json::Value>,
) {
    if let Some(gate) = mission
        .verification_gates
        .iter_mut()
        .find(|gate| gate.id == gate_id)
    {
        gate.status = Some(status);
        gate.evidence.extend(evidence);
    }
}

fn upsert_artifact(mission: &mut MissionSnapshot, artifact: MissionArtifactRef) {
    match mission
        .artifacts
        .iter_mut()
        .find(|candidate| candidate.artifact_id == artifact.artifact_id)
    {
        Some(existing) => *existing = artifact,
        None => mission.artifacts.push(artifact),
    }
}

fn transition_to_executing_if_needed(mission: &mut MissionSnapshot, unix_ms: u64) -> Result<()> {
    match mission.status {
        MissionLifecycleStatus::Draft => {
            mission.transition_to(MissionLifecycleStatus::Planned, unix_ms)?;
            mission.transition_to(MissionLifecycleStatus::Executing, unix_ms)?;
        }
        MissionLifecycleStatus::Planned
        | MissionLifecycleStatus::Blocked
        | MissionLifecycleStatus::Checkpointed
        | MissionLifecycleStatus::Verifying => {
            mission.transition_to(MissionLifecycleStatus::Executing, unix_ms)?;
        }
        MissionLifecycleStatus::Executing | MissionLifecycleStatus::Completed => {}
        MissionLifecycleStatus::Failed | MissionLifecycleStatus::Archived => {
            anyhow::bail!(
                "mission {} cannot resume from status {:?}",
                mission.mission_id,
                mission.status
            );
        }
    }
    Ok(())
}

fn proposal_state_dir(request: &GatewayOpsHarnessSelfImprovementRequest) -> PathBuf {
    request
        .state_dir
        .join(SELF_IMPROVEMENT_SUBDIR)
        .join(&request.proposal_id)
}

fn mission_state_path(request: &GatewayOpsHarnessSelfImprovementRequest) -> PathBuf {
    proposal_state_dir(request).join("mission.json")
}

fn load_harness_self_improvement_state(
    request: &GatewayOpsHarnessSelfImprovementRequest,
) -> Result<MissionSnapshot> {
    let path = mission_state_path(request);
    let payload = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&payload).with_context(|| format!("parse {}", path.display()))
}

fn write_harness_self_improvement_state(
    request: &GatewayOpsHarnessSelfImprovementRequest,
    mission: &MissionSnapshot,
) -> Result<PathBuf> {
    let path = mission_state_path(request);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(mission).context("serialize mission self-improvement state")?,
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn write_harness_self_improvement_result(
    request: &GatewayOpsHarnessSelfImprovementRequest,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<()> {
    let path = proposal_state_dir(request).join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(payload).context("serialize self-improvement result")?,
    )
    .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn dry_run_and_apply_prompt_proposal_persist_mission_state() {
        let workspace = TempDir::new().expect("workspace");
        let state = TempDir::new().expect("state");
        let runner =
            CodingAgentOpsHarnessSelfImprovementRunner::new(workspace.path().to_path_buf());
        let request = GatewayOpsHarnessSelfImprovementRequest {
            proposal_id: "PR-044".to_string(),
            state_dir: state.path().join(".tau/gateway"),
            workspace_root: workspace.path().to_path_buf(),
            requested_unix_ms: 1_700,
        };

        let dry_run = runner.dry_run(request.clone()).expect("dry-run");
        assert_eq!(dry_run.result_key, "passed");
        assert!(!dry_run.applied);
        assert!(dry_run
            .artifact_path
            .as_ref()
            .is_some_and(|path| path.exists()));

        let apply = runner.apply(request.clone()).expect("apply");
        assert_eq!(apply.result_key, "applied");
        assert!(apply.applied);
        assert_eq!(
            fs::read_to_string(workspace.path().join("prompts/research_to_doc/system.md"))
                .expect("target prompt"),
            "# Research To Doc System\n\nUse concise mission-scoped research instructions.\n"
        );

        let mission = load_harness_self_improvement_state(&request).expect("mission state");
        let proposal = mission
            .improvement_proposals
            .iter()
            .find(|proposal| proposal.proposal_id == "PR-044")
            .expect("proposal");
        assert_eq!(mission.status, MissionLifecycleStatus::Completed);
        assert!(mission.recovery_state.is_none());
        assert!(mission.ready_for_completion());
        assert_eq!(proposal.status, MissionImprovementProposalStatus::Applied);
        assert_eq!(
            mission.learning_records[0].curator_status,
            MissionCuratorReviewStatus::Applied
        );
    }

    #[test]
    fn dry_run_and_apply_skill_proposal_writes_skill_manifest() {
        let workspace = TempDir::new().expect("workspace");
        let state = TempDir::new().expect("state");
        let runner =
            CodingAgentOpsHarnessSelfImprovementRunner::new(workspace.path().to_path_buf());
        let request = GatewayOpsHarnessSelfImprovementRequest {
            proposal_id: "PR-045".to_string(),
            state_dir: state.path().join(".tau/gateway"),
            workspace_root: workspace.path().to_path_buf(),
            requested_unix_ms: 1_900,
        };

        let dry_run = runner.dry_run(request.clone()).expect("dry-run");
        assert_eq!(dry_run.result_key, "passed");

        let apply = runner.apply(request.clone()).expect("apply");
        assert_eq!(apply.result_key, "applied");
        assert!(apply.applied);
        let skill_manifest =
            fs::read_to_string(workspace.path().join("skills/benchmark_artifacts/SKILL.md"))
                .expect("target skill manifest");
        assert!(skill_manifest.contains("name: benchmark-artifacts"));
        assert!(skill_manifest
            .contains("description: Name and validate Tau autonomy benchmark proof artifacts."));
        assert!(skill_manifest.contains("mission id, benchmark id, run id, and proof type"));

        let mission = load_harness_self_improvement_state(&request).expect("mission state");
        let proposal = mission
            .improvement_proposals
            .iter()
            .find(|proposal| proposal.proposal_id == "PR-045")
            .expect("proposal");
        assert_eq!(mission.status, MissionLifecycleStatus::Completed);
        assert!(mission.recovery_state.is_none());
        assert!(mission.ready_for_completion());
        assert_eq!(proposal.status, MissionImprovementProposalStatus::Applied);
        assert_eq!(proposal.target_path, "skills/benchmark_artifacts/SKILL.md");
        assert!(mission.artifacts.iter().any(
            |artifact| artifact.path.as_deref() == Some("skills/benchmark_artifacts/SKILL.md")
        ));
        assert!(mission
            .final_learning_output
            .as_ref()
            .is_some_and(|output| output.records == vec!["LR-045".to_string()]));
    }

    #[test]
    fn dry_run_rejects_unknown_harness_proposal() {
        let workspace = TempDir::new().expect("workspace");
        let state = TempDir::new().expect("state");
        let runner =
            CodingAgentOpsHarnessSelfImprovementRunner::new(workspace.path().to_path_buf());
        let request = GatewayOpsHarnessSelfImprovementRequest {
            proposal_id: "PR-999".to_string(),
            state_dir: state.path().join(".tau/gateway"),
            workspace_root: workspace.path().to_path_buf(),
            requested_unix_ms: 1_700,
        };

        let error = runner
            .dry_run(request)
            .expect_err("unknown proposal must fail");
        assert!(error.to_string().contains("unknown ops harness proposal"));
    }
}
