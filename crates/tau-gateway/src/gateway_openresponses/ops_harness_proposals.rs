//! Canonical proposal definitions for the ops harness review/apply lane.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GatewayOpsHarnessProposalDefinition {
    pub proposal_id: &'static str,
    pub source_learning_record_id: &'static str,
    pub queue_label: &'static str,
    pub status_key: &'static str,
    pub title: &'static str,
    pub mission_id: &'static str,
    pub goal: &'static str,
    pub target_type: &'static str,
    pub target_path: &'static str,
    pub proposed_content: &'static str,
    pub rationale: &'static str,
    pub patch_summary: &'static str,
    pub rollback_plan: &'static str,
    pub failure_summary: &'static str,
    pub root_cause: &'static str,
    pub test_command: &'static str,
    pub test_evidence_href: &'static str,
    pub test_evidence_label: &'static str,
    pub safety_check_id: &'static str,
    pub dry_run_result_label: &'static str,
    pub dry_run_result_key: &'static str,
    pub safety_check_label: &'static str,
    pub safety_check_key: &'static str,
    pub diff_removed_lines: &'static [&'static str],
    pub diff_added_lines: &'static [&'static str],
}

const PR_044_REMOVED: &[&str] = &[
    "- Repeat all research goals, source constraints, and output formatting examples in every prompt.",
    "- Include the full research task preamble before each document synthesis step.",
];

const PR_044_ADDED: &[&str] = &[
    "+ Use concise mission-scoped research instructions.",
    "+ Carry source constraints in mission state instead of repeating them in every prompt.",
];

const PR_045_REMOVED: &[&str] =
    &["- Write benchmark artifacts with ad hoc filenames chosen by each task runner."];

const PR_045_ADDED: &[&str] = &[
    "+ Name benchmark artifacts with mission id, benchmark id, run id, and proof type.",
    "+ Keep artifact naming rules in the benchmark artifact skill manifest.",
];

const OPS_HARNESS_PROPOSALS: &[GatewayOpsHarnessProposalDefinition] = &[
    GatewayOpsHarnessProposalDefinition {
        proposal_id: "PR-044",
        source_learning_record_id: "LR-044",
        queue_label: "Prompt compression for research tasks",
        status_key: "proposal",
        title: "Prompt compression for research tasks",
        mission_id: "ops-harness-self-improve-pr-044",
        goal: "Apply prompt compression learning for research tasks",
        target_type: "Prompt",
        target_path: "prompts/research_to_doc/system.md",
        proposed_content: "# Research To Doc System\n\nUse concise mission-scoped research instructions.\n",
        rationale: "Token overruns came from repeated research prompt context.",
        patch_summary: "Compress system prompt by removing redundant instructions and examples.",
        rollback_plan: "Restore the previous prompt file from rollback metadata.",
        failure_summary: "Token overrun during research-to-doc tasks.",
        root_cause: "Verbose prompts with redundant context.",
        test_command: "cargo test -p tau-coding-agent --test mission_self_improvement",
        test_evidence_href: "/evidence/pr-044-dryrun.json",
        test_evidence_label: "evidence/pr-044-dryrun.json",
        safety_check_id: "ops-harness-self-mod-policy",
        dry_run_result_label: "Tests passed (18/18)",
        dry_run_result_key: "passed",
        safety_check_label: "Passed",
        safety_check_key: "passed",
        diff_removed_lines: PR_044_REMOVED,
        diff_added_lines: PR_044_ADDED,
    },
    GatewayOpsHarnessProposalDefinition {
        proposal_id: "PR-045",
        source_learning_record_id: "LR-045",
        queue_label: "Skill patch for benchmark artifact naming",
        status_key: "proposal",
        title: "Skill patch for benchmark artifact naming",
        mission_id: "ops-harness-self-improve-pr-045",
        goal: "Standardize benchmark artifact names through a skill update",
        target_type: "Skill",
        target_path: "skills/benchmark_artifacts/SKILL.md",
        proposed_content: r#"---
name: benchmark-artifacts
description: Name and validate Tau autonomy benchmark proof artifacts.
---

When writing benchmark proof artifacts:

1. Use deterministic names that include mission id, benchmark id, run id, and proof type.
2. Prefer paths shaped as `artifacts/bench/<benchmark-id>/<mission-id>/<run-id>/<proof-type>.json`.
3. Keep `latest.json` as an index or pointer only; do not make it the sole proof artifact.
4. Include mission id, benchmark id, run id, proof type, generated timestamp, verification gates, and artifact source in the artifact payload when available.
5. When a benchmark task emits multiple files, keep the same prefix and vary only the proof type suffix.
6. Do not claim benchmark proof unless the named artifact exists and can be read.
"#,
        rationale: "Benchmark proof review was slowed by inconsistent artifact names.",
        patch_summary: "Add a skill rule for deterministic benchmark artifact naming.",
        rollback_plan: "Remove the skill rule and keep existing artifact names.",
        failure_summary: "Benchmark artifacts were hard to correlate with missions.",
        root_cause: "Artifact naming was left to each task runner.",
        test_command: "cargo test -p tau-coding-agent harness_bin_runs_canonical_m334_benchmark_and_writes_proof",
        test_evidence_href: "/evidence/pr-045-dryrun.json",
        test_evidence_label: "evidence/pr-045-dryrun.json",
        safety_check_id: "ops-harness-skill-artifact-policy",
        dry_run_result_label: "Tests passed (1/1)",
        dry_run_result_key: "passed",
        safety_check_label: "Passed",
        safety_check_key: "passed",
        diff_removed_lines: PR_045_REMOVED,
        diff_added_lines: PR_045_ADDED,
    },
];

pub fn list_ops_harness_proposals() -> &'static [GatewayOpsHarnessProposalDefinition] {
    OPS_HARNESS_PROPOSALS
}

pub fn find_ops_harness_proposal(
    proposal_id: &str,
) -> Option<&'static GatewayOpsHarnessProposalDefinition> {
    OPS_HARNESS_PROPOSALS
        .iter()
        .find(|proposal| proposal.proposal_id == proposal_id)
}
