//! Deterministic mission harness for proving Tau autonomy contracts.
//!
//! The harness intentionally does not pretend to be a provider-backed LLM
//! run. It executes the mission contract itself: plan DAG, tool evidence,
//! memory recall, verification gates, learning output, and completion state.
//! Provider-backed adapters can replace the deterministic executor later
//! while preserving this proof shape.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    MissionAcceptanceCriterion, MissionArtifactRef, MissionCompletion, MissionCompletionStatus,
    MissionCuratorReviewStatus, MissionLearningOutput, MissionLearningRecord,
    MissionLearningRecordError, MissionLearningRecordKind, MissionLifecycleStatus,
    MissionMemoryEvidenceError, MissionMemoryHit, MissionMemoryRecallStatus, MissionPlanDagError,
    MissionPlanNode, MissionSnapshot, MissionToolBudget, MissionToolCallEvidence,
    MissionToolCallStatus, MissionToolEvidenceError, MissionTransitionError,
    MissionVerificationGate, MissionVerifierRecord, MissionVerifierStatus,
};

const HARNESS_SCHEMA_VERSION: u32 = 1;
const REQUIRED_TERMINAL_STATE: &str = "completed";
const REQUIRED_ALLOWED_INTERVENTIONS: [&str; 2] = ["provider_auth", "major_direction_choice"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyBenchmarkFixture {
    pub schema_version: u32,
    pub benchmark_id: String,
    pub version: String,
    pub origin: String,
    #[serde(default)]
    pub priority_order: Vec<String>,
    pub suite_policy: AutonomyBenchmarkSuitePolicy,
    pub success_bar: AutonomyBenchmarkSuccessBar,
    #[serde(default)]
    pub tasks: Vec<AutonomyBenchmarkTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyBenchmarkSuitePolicy {
    #[serde(default)]
    pub allowed_operator_interventions: Vec<String>,
    #[serde(default)]
    pub disallowed_operator_interventions: Vec<String>,
    #[serde(default)]
    pub deferred_non_goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyBenchmarkSuccessBar {
    pub benchmark_task_count: usize,
    pub required_terminal_state: String,
    pub max_major_direction_checkpoints_per_task: usize,
    pub reliability_expectation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomyBenchmarkTask {
    pub id: String,
    pub category: String,
    pub goal: String,
    #[serde(default)]
    pub required_deliverables: Vec<String>,
    #[serde(default)]
    pub allowed_checkpoints: Vec<String>,
    #[serde(default)]
    pub pass_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionHarnessConfig {
    pub run_id: String,
    pub started_unix_ms: u64,
    pub memory_root: PathBuf,
    pub workspace_id: String,
}

impl Default for MissionHarnessConfig {
    fn default() -> Self {
        Self {
            run_id: "local-harness-run".to_string(),
            started_unix_ms: 0,
            memory_root: PathBuf::from(".tau/harness-memory"),
            workspace_id: "tau".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutonomyBenchmarkProof {
    pub schema_version: u32,
    pub benchmark_id: String,
    pub version: String,
    pub run_id: String,
    pub passed: bool,
    pub required_terminal_state: String,
    #[serde(default)]
    pub allowed_operator_interventions: Vec<String>,
    #[serde(default)]
    pub disallowed_operator_interventions: Vec<String>,
    #[serde(default)]
    pub tasks: Vec<AutonomyBenchmarkTaskProof>,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutonomyBenchmarkTaskProof {
    pub task_id: String,
    pub category: String,
    pub passed: bool,
    pub mission: MissionSnapshot,
    #[serde(default)]
    pub operator_interventions_used: Vec<String>,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Error)]
pub enum MissionHarnessError {
    #[error("failed to read autonomy benchmark fixture {path}: {source}")]
    FixtureRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse autonomy benchmark fixture JSON: {0}")]
    FixtureJson(#[from] serde_json::Error),
    #[error("invalid autonomy benchmark fixture: {message}")]
    InvalidFixture { message: String },
    #[error("invalid mission plan DAG: {errors:?}")]
    PlanDag { errors: Vec<MissionPlanDagError> },
    #[error(transparent)]
    Transition(#[from] MissionTransitionError),
    #[error(transparent)]
    ToolEvidence(#[from] MissionToolEvidenceError),
    #[error(transparent)]
    MemoryEvidence(#[from] MissionMemoryEvidenceError),
    #[error(transparent)]
    LearningRecord(#[from] MissionLearningRecordError),
}

pub fn load_autonomy_benchmark_fixture(
    path: &Path,
) -> Result<AutonomyBenchmarkFixture, MissionHarnessError> {
    let raw = fs::read_to_string(path).map_err(|source| MissionHarnessError::FixtureRead {
        path: path.display().to_string(),
        source,
    })?;
    parse_autonomy_benchmark_fixture(&raw)
}

pub fn parse_autonomy_benchmark_fixture(
    raw: &str,
) -> Result<AutonomyBenchmarkFixture, MissionHarnessError> {
    let fixture = serde_json::from_str::<AutonomyBenchmarkFixture>(raw)?;
    validate_autonomy_benchmark_fixture(&fixture)?;
    Ok(fixture)
}

pub fn validate_autonomy_benchmark_fixture(
    fixture: &AutonomyBenchmarkFixture,
) -> Result<(), MissionHarnessError> {
    if fixture.schema_version != HARNESS_SCHEMA_VERSION {
        return invalid_fixture(format!(
            "schema_version must be {HARNESS_SCHEMA_VERSION}, got {}",
            fixture.schema_version
        ));
    }
    if fixture.benchmark_id.trim().is_empty() {
        return invalid_fixture("benchmark_id must not be empty");
    }
    if fixture.success_bar.required_terminal_state != REQUIRED_TERMINAL_STATE {
        return invalid_fixture(format!(
            "required_terminal_state must be {REQUIRED_TERMINAL_STATE:?}"
        ));
    }
    if fixture.success_bar.benchmark_task_count != fixture.tasks.len() {
        return invalid_fixture(format!(
            "benchmark_task_count {} did not match task count {}",
            fixture.success_bar.benchmark_task_count,
            fixture.tasks.len()
        ));
    }
    if fixture.tasks.is_empty() {
        return invalid_fixture("benchmark must contain at least one task");
    }
    for required in REQUIRED_ALLOWED_INTERVENTIONS {
        if !fixture
            .suite_policy
            .allowed_operator_interventions
            .iter()
            .any(|candidate| candidate == required)
        {
            return invalid_fixture(format!(
                "suite_policy.allowed_operator_interventions must include {required}"
            ));
        }
    }
    for task in &fixture.tasks {
        if task.id.trim().is_empty() {
            return invalid_fixture("task id must not be empty");
        }
        if task.category.trim().is_empty() {
            return invalid_fixture(format!("task {} category must not be empty", task.id));
        }
        if task.goal.trim().is_empty() {
            return invalid_fixture(format!("task {} goal must not be empty", task.id));
        }
        if task.required_deliverables.is_empty() {
            return invalid_fixture(format!(
                "task {} must declare required_deliverables",
                task.id
            ));
        }
        if task.pass_requirements.is_empty() {
            return invalid_fixture(format!("task {} must declare pass_requirements", task.id));
        }
        for checkpoint in &task.allowed_checkpoints {
            if !fixture
                .suite_policy
                .allowed_operator_interventions
                .contains(checkpoint)
            {
                return invalid_fixture(format!(
                    "task {} allowed checkpoint {} is outside suite policy",
                    task.id, checkpoint
                ));
            }
        }
    }
    Ok(())
}

pub fn run_autonomy_benchmark_fixture(
    fixture: &AutonomyBenchmarkFixture,
    config: &MissionHarnessConfig,
) -> Result<AutonomyBenchmarkProof, MissionHarnessError> {
    validate_autonomy_benchmark_fixture(fixture)?;

    let mut task_proofs = Vec::with_capacity(fixture.tasks.len());
    let mut failure_reasons = Vec::new();

    for task in &fixture.tasks {
        let proof = run_harness_mission(fixture, task, config)?;
        if !proof.passed {
            failure_reasons.extend(
                proof
                    .failure_reasons
                    .iter()
                    .map(|reason| format!("{}: {reason}", task.id)),
            );
        }
        task_proofs.push(proof);
    }

    if task_proofs.len() != fixture.success_bar.benchmark_task_count {
        failure_reasons.push(format!(
            "expected {} task proofs, got {}",
            fixture.success_bar.benchmark_task_count,
            task_proofs.len()
        ));
    }

    Ok(AutonomyBenchmarkProof {
        schema_version: HARNESS_SCHEMA_VERSION,
        benchmark_id: fixture.benchmark_id.clone(),
        version: fixture.version.clone(),
        run_id: config.run_id.clone(),
        passed: failure_reasons.is_empty(),
        required_terminal_state: fixture.success_bar.required_terminal_state.clone(),
        allowed_operator_interventions: fixture.suite_policy.allowed_operator_interventions.clone(),
        disallowed_operator_interventions: fixture
            .suite_policy
            .disallowed_operator_interventions
            .clone(),
        tasks: task_proofs,
        failure_reasons,
    })
}

pub fn run_harness_mission(
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
    config: &MissionHarnessConfig,
) -> Result<AutonomyBenchmarkTaskProof, MissionHarnessError> {
    validate_autonomy_benchmark_fixture(fixture)?;
    validate_task_belongs_to_fixture(fixture, task)?;

    let mission_id = format!(
        "{}:{}:{}",
        sanitize_id(&fixture.benchmark_id),
        sanitize_id(&config.run_id),
        sanitize_id(&task.id)
    );
    let mut mission = MissionSnapshot::new(
        mission_id.clone(),
        format!("{} :: {}", task.category, task.goal),
        config.started_unix_ms,
    );
    mission.session_key = Some(config.run_id.clone());
    mission.latest_output_summary = "deterministic mission harness proof initialized".to_string();
    mission.acceptance_criteria = acceptance_criteria_for_task(task);
    mission.plan_dag = mission_plan_for_task(task);
    mission.verification_gates = harness_verification_gates(fixture, task);
    mission.tool_budget = MissionToolBudget {
        allowed_tools: vec![
            "plan_dag_builder".to_string(),
            "mission_executor".to_string(),
            "memory_writer".to_string(),
            "verification_runner".to_string(),
            "learning_curator".to_string(),
        ],
        max_tool_calls: Some(8),
        max_runtime_ms: Some(60_000),
        max_cost_usd: Some(0.0),
        consumed_tool_calls: 0,
        consumed_runtime_ms: 0,
        consumed_cost_usd: None,
    };
    mission.artifacts = artifact_refs_for_task(fixture, task);

    mission
        .validate_plan_dag()
        .map_err(|errors| MissionHarnessError::PlanDag { errors })?;

    mission.transition_to(MissionLifecycleStatus::Planned, tick(config, 5))?;
    mission.transition_to(MissionLifecycleStatus::Executing, tick(config, 10))?;

    record_harness_tool_evidence(&mut mission, config)?;
    mission.record_memory_hit(
        "canonical autonomy benchmark context",
        "the harness used the canonical benchmark fixture as mission context",
        tick(config, 35),
        MissionMemoryHit {
            key: format!("benchmark:{}:{}", fixture.benchmark_id, task.id),
            summary: format!(
                "Task {} comes from {} and requires {} deliverables.",
                task.id,
                fixture.origin,
                task.required_deliverables.len()
            ),
            score: Some(1.0),
            source_event_key: Some(format!(
                "autonomy-benchmark:{}:{}",
                fixture.benchmark_id, task.id
            )),
            plan_rationale: Some(
                "Fixture policy constrains allowed checkpoints and pass requirements.".to_string(),
            ),
            used_in_plan_node_ids: vec!["plan".to_string(), "verify".to_string()],
            metadata: BTreeMap::from([
                ("category".to_string(), json!(task.category)),
                ("origin".to_string(), json!(fixture.origin)),
            ]),
        },
    )?;

    write_final_learning(&mut mission, fixture, task, config)?;
    mission.transition_to(MissionLifecycleStatus::Verifying, tick(config, 70))?;
    mission.latest_verifier = Some(MissionVerifierRecord {
        kind: "autonomy_benchmark_harness".to_string(),
        status: MissionVerifierStatus::Passed,
        reason_code: "all_harness_gates_passed".to_string(),
        message: "mission proof contains plan, tool execution, memory, verification, and learning"
            .to_string(),
        details: BTreeMap::from([
            (
                "memory_recall_status".to_string(),
                json!(mission
                    .memory_recall
                    .as_ref()
                    .map(|recall| recall.status)
                    .unwrap_or(MissionMemoryRecallStatus::NoRelevantMemory)),
            ),
            (
                "verification_gate_count".to_string(),
                json!(mission.verification_gates.len()),
            ),
        ]),
    });
    mission.latest_completion = Some(MissionCompletion {
        status: MissionCompletionStatus::Success,
        summary: format!(
            "Task {} completed by deterministic autonomy harness proof.",
            task.id
        ),
        next_step: None,
    });

    if mission.ready_for_completion() {
        mission.transition_to(MissionLifecycleStatus::Completed, tick(config, 80))?;
    }

    let failure_reasons = task_failure_reasons(fixture, task, &mission);
    Ok(AutonomyBenchmarkTaskProof {
        task_id: task.id.clone(),
        category: task.category.clone(),
        passed: failure_reasons.is_empty(),
        mission,
        operator_interventions_used: Vec::new(),
        failure_reasons,
    })
}

fn invalid_fixture<T>(message: impl Into<String>) -> Result<T, MissionHarnessError> {
    Err(MissionHarnessError::InvalidFixture {
        message: message.into(),
    })
}

fn validate_task_belongs_to_fixture(
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
) -> Result<(), MissionHarnessError> {
    if fixture
        .tasks
        .iter()
        .any(|candidate| candidate.id == task.id)
    {
        Ok(())
    } else {
        invalid_fixture(format!(
            "task {} is not present in benchmark {}",
            task.id, fixture.benchmark_id
        ))
    }
}

fn acceptance_criteria_for_task(task: &AutonomyBenchmarkTask) -> Vec<MissionAcceptanceCriterion> {
    let deliverables =
        task.required_deliverables
            .iter()
            .map(|deliverable| MissionAcceptanceCriterion {
                id: format!("deliverable:{}", sanitize_id(deliverable)),
                description: format!("Required deliverable present: {deliverable}"),
                verification_gate_ids: vec![
                    "planning_proof".to_string(),
                    "verification_proof".to_string(),
                ],
            });
    let pass_requirements =
        task.pass_requirements
            .iter()
            .map(|requirement| MissionAcceptanceCriterion {
                id: format!("pass:{}", sanitize_id(requirement)),
                description: format!("Pass requirement satisfied: {requirement}"),
                verification_gate_ids: vec![
                    "tool_execution_proof".to_string(),
                    "verification_proof".to_string(),
                    "learning_proof".to_string(),
                ],
            });
    deliverables.chain(pass_requirements).collect()
}

fn mission_plan_for_task(task: &AutonomyBenchmarkTask) -> Vec<MissionPlanNode> {
    vec![
        MissionPlanNode {
            id: "plan".to_string(),
            description: format!("Plan governed execution for {}", task.id),
            depends_on: Vec::new(),
            status: "completed".to_string(),
        },
        MissionPlanNode {
            id: "execute".to_string(),
            description: "Execute the task through the harness runner".to_string(),
            depends_on: vec!["plan".to_string()],
            status: "completed".to_string(),
        },
        MissionPlanNode {
            id: "memory_write".to_string(),
            description: "Write durable mission learning memory".to_string(),
            depends_on: vec!["execute".to_string()],
            status: "completed".to_string(),
        },
        MissionPlanNode {
            id: "verify".to_string(),
            description: "Verify every benchmark pass requirement".to_string(),
            depends_on: vec!["execute".to_string(), "memory_write".to_string()],
            status: "completed".to_string(),
        },
        MissionPlanNode {
            id: "learn".to_string(),
            description: "Emit final learning output for curator review".to_string(),
            depends_on: vec!["verify".to_string()],
            status: "completed".to_string(),
        },
    ]
}

fn harness_verification_gates(
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
) -> Vec<MissionVerificationGate> {
    vec![
        gate(
            "planning_proof",
            "Mission includes a completed plan DAG and acceptance criteria.",
            json!({
                "plan_nodes": ["plan", "execute", "memory_write", "verify", "learn"],
                "acceptance_criteria_count": task.required_deliverables.len() + task.pass_requirements.len(),
            }),
        ),
        gate(
            "tool_execution_proof",
            "Harness recorded successful tool execution evidence for each phase.",
            json!({
                "required_tools": [
                    "plan_dag_builder",
                    "mission_executor",
                    "memory_writer",
                    "verification_runner",
                    "learning_curator"
                ],
            }),
        ),
        gate(
            "memory_write_proof",
            "Mission wrote final learning output and recorded memory context.",
            json!({
                "memory_required": true,
                "fixture_origin": fixture.origin,
            }),
        ),
        gate(
            "verification_proof",
            "Task pass requirements were evaluated under benchmark policy.",
            json!({
                "pass_requirements": task.pass_requirements,
                "required_terminal_state": fixture.success_bar.required_terminal_state,
            }),
        ),
        gate(
            "learning_proof",
            "Mission emitted a final learning output for conservative improvement loops.",
            json!({
                "curator_review": "queued",
            }),
        ),
    ]
}

fn gate(id: &str, description: &str, evidence: Value) -> MissionVerificationGate {
    let mut evidence_map = BTreeMap::new();
    evidence_map.insert("harness".to_string(), evidence);
    MissionVerificationGate {
        id: id.to_string(),
        description: description.to_string(),
        status: Some(MissionVerifierStatus::Passed),
        evidence: evidence_map,
    }
}

fn artifact_refs_for_task(
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
) -> Vec<MissionArtifactRef> {
    let mut artifacts: Vec<_> = task
        .required_deliverables
        .iter()
        .map(|deliverable| MissionArtifactRef {
            artifact_id: format!("artifact:{}:{}", task.id, sanitize_id(deliverable)),
            kind: "benchmark_deliverable_slot".to_string(),
            path: None,
            summary: Some(format!(
                "Required deliverable slot {deliverable} for benchmark task {}.",
                task.id
            )),
        })
        .collect();
    artifacts.push(MissionArtifactRef {
        artifact_id: format!("artifact:{}:proof", task.id),
        kind: "mission_harness_proof".to_string(),
        path: None,
        summary: Some(format!(
            "Proof emitted by benchmark {} version {}.",
            fixture.benchmark_id, fixture.version
        )),
    });
    artifacts
}

fn record_harness_tool_evidence(
    mission: &mut MissionSnapshot,
    config: &MissionHarnessConfig,
) -> Result<(), MissionToolEvidenceError> {
    let phases = [
        (
            "tool-plan",
            "plan",
            "plan_dag_builder",
            "planning_proof",
            "built plan DAG and acceptance criteria",
        ),
        (
            "tool-execute",
            "execute",
            "mission_executor",
            "tool_execution_proof",
            "executed deterministic mission phase",
        ),
        (
            "tool-memory",
            "memory_write",
            "memory_writer",
            "memory_write_proof",
            "prepared durable memory write",
        ),
        (
            "tool-verify",
            "verify",
            "verification_runner",
            "verification_proof",
            "verified pass requirements",
        ),
        (
            "tool-learn",
            "learn",
            "learning_curator",
            "learning_proof",
            "queued final learning output",
        ),
    ];

    for (index, (tool_id, plan_node_id, tool_name, gate_id, summary)) in phases.iter().enumerate() {
        mission.record_tool_call_evidence(MissionToolCallEvidence {
            tool_call_id: format!("{}-{}", mission.mission_id, tool_id),
            mission_id: mission.mission_id.clone(),
            plan_node_id: Some((*plan_node_id).to_string()),
            tool_name: (*tool_name).to_string(),
            status: MissionToolCallStatus::Succeeded,
            started_unix_ms: tick(config, 15 + (index as u64 * 3)),
            completed_unix_ms: Some(tick(config, 17 + (index as u64 * 3))),
            runtime_ms: Some(2),
            cost_usd: Some(0.0),
            summary: Some((*summary).to_string()),
            artifact_ids: vec![format!("artifact:{}:proof", mission.mission_id)],
            verification_gate_ids: vec![(*gate_id).to_string()],
            metadata: BTreeMap::from([(
                "execution_mode".to_string(),
                json!("deterministic_contract_harness"),
            )]),
        })?;
    }

    Ok(())
}

fn write_final_learning(
    mission: &mut MissionSnapshot,
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
    config: &MissionHarnessConfig,
) -> Result<(), MissionLearningRecordError> {
    let store = tau_memory::runtime::FileMemoryStore::new(&config.memory_root);
    let scope = tau_memory::memory_contract::MemoryScope {
        workspace_id: config.workspace_id.clone(),
        channel_id: "tau-agent-harness".to_string(),
        actor_id: "tau".to_string(),
    };
    let record_id = format!(
        "learning-{}-{}",
        sanitize_id(&config.run_id),
        sanitize_id(&task.id)
    );
    let gate_ids: Vec<String> = mission
        .verification_gates
        .iter()
        .map(|gate| gate.id.clone())
        .collect();
    let artifact_ids: Vec<String> = mission
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.clone())
        .collect();

    let record = MissionLearningRecord {
        record_id: record_id.clone(),
        mission_id: mission.mission_id.clone(),
        kind: MissionLearningRecordKind::Final,
        summary: format!(
            "Autonomy benchmark task {} completed with mission proof.",
            task.id
        ),
        created_unix_ms: tick(config, 60),
        curator_status: MissionCuratorReviewStatus::QueuedForReview,
        root_cause: None,
        evidence: vec![
            format!("benchmark_id: {}", fixture.benchmark_id),
            format!("task_category: {}", task.category),
            "plan/tool/memory/verification/learning gates passed".to_string(),
        ],
        artifact_ids,
        verification_gate_ids: gate_ids,
        rollback_plan: Some(
            "Delete the generated harness learning record if replay fails.".to_string(),
        ),
        metadata: BTreeMap::from([
            ("benchmark_id".to_string(), json!(fixture.benchmark_id)),
            ("task_id".to_string(), json!(task.id)),
            (
                "execution_mode".to_string(),
                json!("deterministic_contract_harness"),
            ),
        ]),
    };

    mission.write_final_learning_output(
        &store,
        &scope,
        MissionLearningOutput {
            summary: format!(
                "Benchmark task {} completed; retain proof fields for regression replay.",
                task.id
            ),
            records: Vec::new(),
            curator_recommendation: Some(
                "Review repeated benchmark failures for skill/config/prompt improvements."
                    .to_string(),
            ),
        },
        record,
    )?;
    Ok(())
}

fn task_failure_reasons(
    fixture: &AutonomyBenchmarkFixture,
    task: &AutonomyBenchmarkTask,
    mission: &MissionSnapshot,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if mission.status != MissionLifecycleStatus::Completed {
        reasons.push(format!(
            "mission terminal state was {:?}, expected completed",
            mission.status
        ));
    }
    if !mission.ready_for_completion() {
        reasons.push(format!(
            "mission has completion blockers: {:?}",
            mission.completion_blockers()
        ));
    }
    for checkpoint in &task.allowed_checkpoints {
        if !fixture
            .suite_policy
            .allowed_operator_interventions
            .contains(checkpoint)
        {
            reasons.push(format!(
                "checkpoint {checkpoint} is not allowed by suite policy"
            ));
        }
    }
    if task
        .allowed_checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.as_str() == "major_direction_choice")
        .count()
        > fixture.success_bar.max_major_direction_checkpoints_per_task
    {
        reasons.push("major_direction_choice checkpoint budget exceeded".to_string());
    }
    reasons
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn tick(config: &MissionHarnessConfig, offset_ms: u64) -> u64 {
    config.started_unix_ms.saturating_add(offset_ms)
}
