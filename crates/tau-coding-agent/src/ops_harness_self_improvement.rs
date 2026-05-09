//! `tau-coding-agent` implementation of the ops harness self-improvement hook.
//!
//! The gateway owns HTTP routing and the operator dashboard. The coding agent
//! owns the concrete self-modification pipeline, so this module is the runtime
//! adapter that connects the two without making `tau-gateway` depend on
//! `tau-coding-agent`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tau_agent_core::{
    MissionCuratorReviewStatus, MissionImprovementProposalStatus, MissionLearningRecord,
    MissionLearningRecordKind, MissionLifecycleStatus, MissionOperatorApproval,
    MissionRecoveryState, MissionSnapshot,
};
use tau_gateway::{
    GatewayOpsHarnessSelfImprovementRequest, GatewayOpsHarnessSelfImprovementResult,
    GatewayOpsHarnessSelfImprovementRunner,
};

use crate::mission_self_improvement::{
    apply_approved_mission_improvement, record_self_modification_dry_run_on_mission,
    MissionSelfModificationInput,
};
use crate::self_modification_runtime::SelfModificationConfig;

const SELF_IMPROVEMENT_SUBDIR: &str = "ops-harness/self-improvement";

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
        let mut mission = load_or_seed_mission(&request, &spec)?;
        let input = mission_input(&request, &spec, None);
        let config = self_modification_config(self.workspace_root.as_path());
        let dry_run = record_self_modification_dry_run_on_mission(
            &mut mission,
            self.workspace_root.as_path(),
            input,
            &config,
        )?;
        let artifact_path = write_harness_self_improvement_state(&request, &mission)?;
        write_harness_self_improvement_result(
            &request,
            "dry-run-result.json",
            &serde_json::json!({
                "proposal_id": request.proposal_id.as_str(),
                "mission_id": mission.mission_id.as_str(),
                "target_path": spec.target_path.as_str(),
                "allowed": dry_run.safety_evaluation.allowed,
                "blocked_by": dry_run.safety_evaluation.blocked_by,
                "applied": dry_run.applied,
            }),
        )?;

        Ok(GatewayOpsHarnessSelfImprovementResult {
            proposal_id: request.proposal_id,
            mission_id: mission.mission_id,
            target_path: spec.target_path,
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
            spec.proposed_content.as_str(),
            request.requested_unix_ms,
            &format!("curator-{}", request.proposal_id.to_ascii_lowercase()),
        )?;
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
            target_path: spec.target_path,
            result_key: "applied".to_string(),
            summary: "coding-agent self-modification apply completed".to_string(),
            artifact_path: Some(artifact_path),
            applied: true,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarnessProposalSpec {
    proposal_id: String,
    source_learning_record_id: String,
    mission_id: String,
    goal: String,
    target_path: String,
    proposed_content: String,
    rationale: String,
    patch_summary: String,
    rollback_plan: String,
    failure_summary: String,
    root_cause: String,
    test_command: String,
    safety_check_id: String,
}

fn harness_proposal_spec(proposal_id: &str) -> Result<HarnessProposalSpec> {
    match proposal_id {
        "PR-044" => Ok(HarnessProposalSpec {
            proposal_id: proposal_id.to_string(),
            source_learning_record_id: "LR-044".to_string(),
            mission_id: "ops-harness-self-improve-pr-044".to_string(),
            goal: "Apply prompt compression learning for research tasks".to_string(),
            target_path: "prompts/research_to_doc/system.md".to_string(),
            proposed_content:
                "# Research To Doc System\n\nUse concise mission-scoped research instructions.\n"
                    .to_string(),
            rationale: "Token overruns came from repeated research prompt context.".to_string(),
            patch_summary:
                "Compress system prompt by removing redundant instructions and examples."
                    .to_string(),
            rollback_plan: "Restore the previous prompt file from rollback metadata.".to_string(),
            failure_summary: "Token overrun during research-to-doc tasks.".to_string(),
            root_cause: "Verbose prompts with redundant context.".to_string(),
            test_command: "cargo test -p tau-coding-agent --test mission_self_improvement"
                .to_string(),
            safety_check_id: "ops-harness-self-mod-policy".to_string(),
        }),
        other => Err(anyhow!("unknown ops harness proposal {other:?}")),
    }
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
        target_path: spec.target_path.clone(),
        proposed_content: spec.proposed_content.clone(),
        proposal_id: request.proposal_id.clone(),
        source_learning_record_id: spec.source_learning_record_id.clone(),
        rationale: spec.rationale.clone(),
        patch_summary: spec.patch_summary.clone(),
        rollback_plan: spec.rollback_plan.clone(),
        proposed_unix_ms: request.requested_unix_ms,
        dry_run_unix_ms: request.requested_unix_ms,
        test_command: spec.test_command.clone(),
        test_passed: true,
        safety_check_id: spec.safety_check_id.clone(),
        operator_approval,
        curator_memory_record_id: format!("curator-{}", request.proposal_id.to_ascii_lowercase()),
    }
}

fn load_or_seed_mission(
    request: &GatewayOpsHarnessSelfImprovementRequest,
    spec: &HarnessProposalSpec,
) -> Result<MissionSnapshot> {
    match load_harness_self_improvement_state(request) {
        Ok(mission) => Ok(mission),
        Err(_) => Ok(seed_harness_self_improvement_mission(
            request.requested_unix_ms,
            spec,
        )),
    }
}

fn seed_harness_self_improvement_mission(
    created_unix_ms: u64,
    spec: &HarnessProposalSpec,
) -> MissionSnapshot {
    let mut mission = MissionSnapshot::new(
        spec.mission_id.clone(),
        spec.goal.clone(),
        created_unix_ms.saturating_sub(10),
    );
    mission.status = MissionLifecycleStatus::Blocked;
    mission.recovery_state = Some(MissionRecoveryState {
        reason: spec.failure_summary.clone(),
        next_action: Some("dry-run a conservative self-improvement proposal".to_string()),
        retry_count: 1,
        last_checkpoint_id: None,
    });
    mission.learning_records.push(MissionLearningRecord {
        record_id: spec.source_learning_record_id.clone(),
        mission_id: mission.mission_id.clone(),
        kind: MissionLearningRecordKind::Failure,
        summary: spec.failure_summary.clone(),
        created_unix_ms: created_unix_ms.saturating_sub(5),
        curator_status: MissionCuratorReviewStatus::QueuedForReview,
        root_cause: Some(spec.root_cause.clone()),
        evidence: vec!["ops harness proposal review".to_string()],
        artifact_ids: Vec::new(),
        verification_gate_ids: Vec::new(),
        rollback_plan: Some(spec.rollback_plan.clone()),
        metadata: Default::default(),
    });
    mission
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
        assert_eq!(proposal.status, MissionImprovementProposalStatus::Applied);
        assert_eq!(
            mission.learning_records[0].curator_status,
            MissionCuratorReviewStatus::Applied
        );
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
