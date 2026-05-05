use tau_agent_core::{
    MissionCuratorReviewStatus, MissionImprovementProposal, MissionImprovementProposalStatus,
    MissionImprovementTargetKind, MissionImprovementTriggerKind, MissionLearningRecord,
    MissionLearningRecordKind, MissionLifecycleStatus, MissionOperatorApproval,
    MissionRecoveryState, MissionSnapshot,
};
use tau_coding_agent::mission_self_improvement::{
    apply_approved_mission_improvement, record_self_modification_dry_run_on_mission,
    MissionSelfModificationInput,
};
use tau_coding_agent::self_modification_runtime::SelfModificationConfig;
use tempfile::TempDir;

fn mission_with_failure_record() -> MissionSnapshot {
    let mut mission = MissionSnapshot::new(
        "mission-self-improve",
        "observe failure and improve a skill",
        1_000,
    );
    mission.status = MissionLifecycleStatus::Blocked;
    mission.recovery_state = Some(MissionRecoveryState {
        reason: "benchmark verifier found missing retry guidance".to_string(),
        next_action: Some("propose a skill improvement".to_string()),
        retry_count: 1,
        last_checkpoint_id: None,
    });
    mission.learning_records.push(MissionLearningRecord {
        record_id: "learning-failure-1".to_string(),
        mission_id: mission.mission_id.clone(),
        kind: MissionLearningRecordKind::Failure,
        summary: "Retry skill missed recovery evidence.".to_string(),
        created_unix_ms: 1_010,
        curator_status: MissionCuratorReviewStatus::QueuedForReview,
        root_cause: Some("missing recovery checklist".to_string()),
        evidence: vec!["benchmark gate failed".to_string()],
        artifact_ids: Vec::new(),
        verification_gate_ids: Vec::new(),
        rollback_plan: Some("remove generated skill patch".to_string()),
        metadata: Default::default(),
    });
    mission
}

fn approved_input() -> MissionSelfModificationInput {
    MissionSelfModificationInput {
        target_path: "skills/autonomy/retry-loop.md".to_string(),
        proposed_content: "# Retry Loop\n\nAlways attach recovery evidence.\n".to_string(),
        proposal_id: "proposal-skill-1".to_string(),
        source_learning_record_id: "learning-failure-1".to_string(),
        rationale: "The benchmark failed because retry recovery evidence was omitted.".to_string(),
        patch_summary: "Add recovery evidence guidance to retry-loop skill.".to_string(),
        rollback_plan: "Restore the prior skill file from rollback metadata.".to_string(),
        proposed_unix_ms: 1_020,
        dry_run_unix_ms: 1_030,
        test_command: "cargo test -p tau-coding-agent --test mission_self_improvement".to_string(),
        test_passed: true,
        safety_check_id: "self-mod-policy".to_string(),
        operator_approval: Some(MissionOperatorApproval {
            approval_id: "approval-1".to_string(),
            operator_id: "operator".to_string(),
            approved_unix_ms: 1_040,
            summary: "Approved safe skill improvement after dry-run.".to_string(),
        }),
        curator_memory_record_id: "curator-memory-1".to_string(),
    }
}

#[test]
fn dry_run_records_mission_improvement_proposal_evidence() {
    let workspace = TempDir::new().expect("tempdir");
    let mut mission = mission_with_failure_record();
    let config = SelfModificationConfig::default();

    let dry_run = record_self_modification_dry_run_on_mission(
        &mut mission,
        workspace.path(),
        approved_input(),
        &config,
    )
    .expect("dry-run should be recorded");

    assert_eq!(dry_run.proposal_id, "proposal-skill-1");
    assert!(!dry_run.applied);
    assert!(dry_run.safety_evaluation.allowed);

    let proposal = mission
        .improvement_proposals
        .iter()
        .find(|proposal| proposal.proposal_id == "proposal-skill-1")
        .expect("proposal recorded");
    assert_eq!(proposal.status, MissionImprovementProposalStatus::Approved);
    assert!(proposal.dry_run.as_ref().is_some_and(|run| run.passed));
    assert_eq!(proposal.tests.len(), 1);
    assert_eq!(proposal.safety_checks.len(), 1);
    assert!(proposal.approval.is_some());
}

#[test]
fn approved_safe_skill_improvement_applies_file_and_curator_metadata() {
    let workspace = TempDir::new().expect("tempdir");
    let mut mission = mission_with_failure_record();
    let input = approved_input();
    let config = SelfModificationConfig::default();

    record_self_modification_dry_run_on_mission(
        &mut mission,
        workspace.path(),
        input.clone(),
        &config,
    )
    .expect("dry-run should be recorded");

    let applied = apply_approved_mission_improvement(
        &mut mission,
        workspace.path(),
        &input.proposal_id,
        &input.proposed_content,
        1_050,
        &input.curator_memory_record_id,
    )
    .expect("approved improvement should apply");

    let target = workspace.path().join(&input.target_path);
    assert_eq!(
        std::fs::read_to_string(&target).expect("target written"),
        input.proposed_content
    );
    assert!(applied.rollback_path.starts_with(workspace.path()));
    assert!(applied.target_path.ends_with(&input.target_path));
    assert!(applied.target_path.starts_with(
        workspace
            .path()
            .canonicalize()
            .expect("canonical workspace")
    ));

    let proposal = mission
        .improvement_proposals
        .iter()
        .find(|proposal| proposal.proposal_id == input.proposal_id)
        .expect("proposal recorded");
    assert_eq!(proposal.status, MissionImprovementProposalStatus::Applied);
    assert_eq!(
        proposal.curator_memory_record_id.as_deref(),
        Some(input.curator_memory_record_id.as_str())
    );
    assert_eq!(
        mission.learning_records[0].curator_status,
        MissionCuratorReviewStatus::Applied
    );
}

#[test]
fn apply_rejects_source_or_outside_workspace_target() {
    let workspace = TempDir::new().expect("tempdir");
    let config = SelfModificationConfig::default();

    let mut source_input = approved_input();
    source_input.target_path = "crates/tau-coding-agent/src/lib.rs".to_string();
    source_input.proposal_id = "proposal-source-1".to_string();
    source_input.operator_approval = None;
    let mut mission = mission_with_failure_record();
    let error = record_self_modification_dry_run_on_mission(
        &mut mission,
        workspace.path(),
        source_input,
        &config,
    )
    .expect_err("source targets must be blocked");
    assert!(
        error.to_string().contains("outside conservative loop")
            || error.to_string().contains("auto_apply_source_disabled"),
        "unexpected error: {error:#}"
    );

    let mut mission = mission_with_failure_record();
    let escape_input = approved_input();
    mission
        .improvement_proposals
        .push(MissionImprovementProposal {
            proposal_id: "proposal-escape-1".to_string(),
            mission_id: mission.mission_id.clone(),
            source_learning_record_id: "learning-failure-1".to_string(),
            trigger_kind: MissionImprovementTriggerKind::BenchmarkFailure,
            target_kind: MissionImprovementTargetKind::Skill,
            target_path: "../escape.md".to_string(),
            patch_summary: "Unsafe escaped target should never apply.".to_string(),
            rationale: "Regression fixture for apply containment.".to_string(),
            rollback_plan: "No mutation should occur.".to_string(),
            proposed_unix_ms: 1_020,
            status: MissionImprovementProposalStatus::Approved,
            dry_run: None,
            tests: Vec::new(),
            safety_checks: Vec::new(),
            approval: Some(MissionOperatorApproval {
                approval_id: "approval-escape".to_string(),
                operator_id: "operator".to_string(),
                approved_unix_ms: 1_040,
                summary: "Synthetic approval for containment regression.".to_string(),
            }),
            applied_unix_ms: None,
            curator_memory_record_id: None,
            metadata: Default::default(),
        });
    let error = apply_approved_mission_improvement(
        &mut mission,
        workspace.path(),
        "proposal-escape-1",
        &escape_input.proposed_content,
        1_050,
        &escape_input.curator_memory_record_id,
    )
    .expect_err("outside workspace target must be rejected");
    assert!(
        error.to_string().contains("outside workspace"),
        "unexpected error: {error:#}"
    );
}
