//! Mission-owned self-improvement bridge.
//!
//! This module connects `tau-coding-agent` self-modification dry-runs to
//! `tau-agent-core` mission improvement proposals, then provides the
//! approval-gated apply path for conservative skill/config/prompt changes.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tau_agent_core::{
    MissionImprovementDryRun, MissionImprovementEvidenceStatus, MissionImprovementProposal,
    MissionImprovementProposalStatus, MissionImprovementSafetyCheck, MissionImprovementTargetKind,
    MissionImprovementTestEvidence, MissionImprovementTriggerKind, MissionOperatorApproval,
    MissionSnapshot,
};

use crate::self_modification_pipeline::run_dry_run_pipeline;
use crate::self_modification_runtime::{
    classify_modification_target, validate_proposal_id, ModificationTarget, SelfModificationConfig,
    SelfModificationResult,
};

pub const SELF_MOD_ROLLBACKS_SUBDIR: &str = ".tau/self-mod-rollbacks";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionSelfModificationInput {
    pub target_path: String,
    pub proposed_content: String,
    pub proposal_id: String,
    pub source_learning_record_id: String,
    pub rationale: String,
    pub patch_summary: String,
    pub rollback_plan: String,
    pub proposed_unix_ms: u64,
    pub dry_run_unix_ms: u64,
    pub test_command: String,
    pub test_passed: bool,
    pub safety_check_id: String,
    #[serde(default)]
    pub operator_approval: Option<MissionOperatorApproval>,
    pub curator_memory_record_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionSelfModificationApplyResult {
    pub proposal_id: String,
    pub target_path: PathBuf,
    pub rollback_path: PathBuf,
    pub applied_unix_ms: u64,
    pub previous_content_existed: bool,
}

pub fn record_self_modification_dry_run_on_mission(
    mission: &mut MissionSnapshot,
    workspace_root: &Path,
    input: MissionSelfModificationInput,
    config: &SelfModificationConfig,
) -> Result<SelfModificationResult> {
    validate_proposal_id(&input.proposal_id)
        .with_context(|| format!("proposal_id {:?} rejected", input.proposal_id))?;
    let target_kind = mission_target_kind(&input.target_path)?;
    let dry_run = run_dry_run_pipeline(
        workspace_root,
        &input.target_path,
        Some(&input.proposal_id),
        config,
    )?;

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "self_modification_result".to_string(),
        serde_json::to_value(&dry_run).context("serialize self-modification result")?,
    );
    metadata.insert(
        "target_classification".to_string(),
        json!(format!(
            "{:?}",
            classify_modification_target(&input.target_path)
        )),
    );

    mission.record_improvement_proposal_from_failure(MissionImprovementProposal {
        proposal_id: input.proposal_id.clone(),
        mission_id: mission.mission_id.clone(),
        source_learning_record_id: input.source_learning_record_id.clone(),
        trigger_kind: MissionImprovementTriggerKind::BenchmarkFailure,
        target_kind,
        target_path: input.target_path.clone(),
        patch_summary: input.patch_summary.clone(),
        rationale: input.rationale.clone(),
        rollback_plan: input.rollback_plan.clone(),
        proposed_unix_ms: input.proposed_unix_ms,
        status: MissionImprovementProposalStatus::Proposed,
        dry_run: None,
        tests: Vec::new(),
        safety_checks: Vec::new(),
        approval: None,
        applied_unix_ms: None,
        curator_memory_record_id: None,
        metadata,
    })?;

    mission.record_improvement_dry_run(
        &input.proposal_id,
        MissionImprovementDryRun {
            ran_unix_ms: input.dry_run_unix_ms,
            passed: dry_run.safety_evaluation.allowed,
            summary: if dry_run.safety_evaluation.allowed {
                "self-modification dry-run passed policy".to_string()
            } else {
                format!(
                    "self-modification dry-run blocked by {:?}",
                    dry_run.safety_evaluation.blocked_by
                )
            },
            artifact_ids: Vec::new(),
            metadata: BTreeMap::from([
                ("applied".to_string(), json!(dry_run.applied)),
                (
                    "rollback_available".to_string(),
                    json!(dry_run.rollback_available),
                ),
            ]),
        },
    )?;
    mission.record_improvement_test_evidence(
        &input.proposal_id,
        MissionImprovementTestEvidence {
            command: input.test_command.clone(),
            status: if input.test_passed {
                MissionImprovementEvidenceStatus::Passed
            } else {
                MissionImprovementEvidenceStatus::Failed
            },
            ran_unix_ms: input.dry_run_unix_ms.saturating_add(1),
            summary: if input.test_passed {
                "operator-supplied validation command passed".to_string()
            } else {
                "operator-supplied validation command failed".to_string()
            },
            artifact_ids: Vec::new(),
            metadata: BTreeMap::new(),
        },
    )?;
    mission.record_improvement_safety_check(
        &input.proposal_id,
        MissionImprovementSafetyCheck {
            check_id: input.safety_check_id.clone(),
            status: if dry_run.safety_evaluation.allowed {
                MissionImprovementEvidenceStatus::Passed
            } else {
                MissionImprovementEvidenceStatus::Failed
            },
            ran_unix_ms: input.dry_run_unix_ms.saturating_add(2),
            summary: if dry_run.safety_evaluation.allowed {
                "self-modification policy allowed target".to_string()
            } else {
                "self-modification policy blocked target".to_string()
            },
            blocked_by: dry_run.safety_evaluation.blocked_by.clone(),
            metadata: BTreeMap::from([(
                "warnings".to_string(),
                json!(dry_run.safety_evaluation.warnings),
            )]),
        },
    )?;

    if let Some(approval) = input.operator_approval {
        mission.approve_improvement_proposal(&input.proposal_id, approval)?;
    }

    Ok(dry_run)
}

pub fn apply_approved_mission_improvement(
    mission: &mut MissionSnapshot,
    workspace_root: &Path,
    proposal_id: &str,
    proposed_content: &str,
    applied_unix_ms: u64,
    curator_memory_record_id: &str,
) -> Result<MissionSelfModificationApplyResult> {
    validate_proposal_id(proposal_id)
        .with_context(|| format!("proposal_id {:?} rejected", proposal_id))?;

    let proposal = mission
        .improvement_proposals
        .iter()
        .find(|proposal| proposal.proposal_id == proposal_id)
        .cloned()
        .ok_or_else(|| anyhow!("improvement proposal {proposal_id:?} was not found"))?;

    ensure_apply_eligible(&proposal)?;
    let target_path = resolve_workspace_target(workspace_root, &proposal.target_path)?;
    let rollback_root = workspace_root
        .join(SELF_MOD_ROLLBACKS_SUBDIR)
        .join(proposal_id);
    fs::create_dir_all(&rollback_root)
        .with_context(|| format!("create rollback root {}", rollback_root.display()))?;

    let previous_content_existed = target_path.exists();
    let rollback_path = rollback_root.join("previous-content.bin");
    if previous_content_existed {
        let previous =
            fs::read(&target_path).with_context(|| format!("read {}", target_path.display()))?;
        fs::write(&rollback_path, previous)
            .with_context(|| format!("write rollback {}", rollback_path.display()))?;
    }

    let metadata_path = rollback_root.join("metadata.json");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&json!({
            "proposal_id": proposal_id,
            "target_path": proposal.target_path,
            "previous_content_existed": previous_content_existed,
            "applied_unix_ms": applied_unix_ms,
        }))
        .context("serialize rollback metadata")?,
    )
    .with_context(|| format!("write rollback metadata {}", metadata_path.display()))?;

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create target parent {}", parent.display()))?;
    }
    fs::write(&target_path, proposed_content)
        .with_context(|| format!("write target {}", target_path.display()))?;

    mission.apply_approved_improvement(
        proposal_id,
        applied_unix_ms,
        curator_memory_record_id.to_string(),
    )?;
    if let Some(applied) = mission
        .improvement_proposals
        .iter_mut()
        .find(|proposal| proposal.proposal_id == proposal_id)
    {
        applied
            .metadata
            .insert("applied_target_path".to_string(), json!(target_path));
        applied
            .metadata
            .insert("rollback_path".to_string(), json!(rollback_root));
    }

    Ok(MissionSelfModificationApplyResult {
        proposal_id: proposal_id.to_string(),
        target_path,
        rollback_path,
        applied_unix_ms,
        previous_content_existed,
    })
}

fn mission_target_kind(target_path: &str) -> Result<MissionImprovementTargetKind> {
    match classify_modification_target(target_path) {
        ModificationTarget::Skill => Ok(MissionImprovementTargetKind::Skill),
        ModificationTarget::Config => Ok(MissionImprovementTargetKind::Config),
        ModificationTarget::Prompt => Ok(MissionImprovementTargetKind::Prompt),
        ModificationTarget::Source => bail!(
            "target {target_path:?} is outside conservative loop: source-code self-modification"
        ),
        ModificationTarget::Other => bail!(
            "target {target_path:?} is outside conservative loop: unrecognized self-modification target"
        ),
    }
}

fn ensure_apply_eligible(proposal: &MissionImprovementProposal) -> Result<()> {
    if !matches!(
        proposal.target_kind,
        MissionImprovementTargetKind::Skill
            | MissionImprovementTargetKind::Config
            | MissionImprovementTargetKind::Prompt
    ) {
        bail!(
            "proposal {:?} target {:?} is outside conservative loop",
            proposal.proposal_id,
            proposal.target_kind
        );
    }
    if proposal.status == MissionImprovementProposalStatus::Applied {
        bail!(
            "proposal {:?} has already been applied",
            proposal.proposal_id
        );
    }
    if proposal.status != MissionImprovementProposalStatus::Approved {
        bail!(
            "proposal {:?} must be approved before apply; status is {:?}",
            proposal.proposal_id,
            proposal.status
        );
    }
    if proposal.approval.is_none() {
        bail!(
            "proposal {:?} is missing operator approval",
            proposal.proposal_id
        );
    }
    Ok(())
}

fn resolve_workspace_target(workspace_root: &Path, target_path: &str) -> Result<PathBuf> {
    if target_path.trim().is_empty() {
        bail!("target path must not be empty");
    }
    fs::create_dir_all(workspace_root)
        .with_context(|| format!("create workspace root {}", workspace_root.display()))?;
    let workspace = workspace_root
        .canonicalize()
        .with_context(|| format!("canonicalize workspace root {}", workspace_root.display()))?;

    let mut relative = PathBuf::new();
    for component in Path::new(target_path).components() {
        match component {
            Component::Normal(segment) => relative.push(segment),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("target path {target_path:?} is outside workspace")
            }
        }
    }
    if relative.as_os_str().is_empty() {
        bail!("target path must contain at least one normal component");
    }

    let target = workspace.join(relative);
    if !target.starts_with(&workspace) {
        bail!("target path {target_path:?} is outside workspace");
    }
    Ok(target)
}
