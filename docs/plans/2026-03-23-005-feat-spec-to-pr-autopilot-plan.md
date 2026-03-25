---
title: feat: Add Spec-to-PR Autopilot
type: feat
status: active
date: 2026-03-23
---

# feat: Add Spec-to-PR Autopilot

## Overview

Build an operator-visible automation flow that turns Tau's own repository governance into a product capability: issue intake, spec generation, plan and task creation, conformance/test mapping, status updates, PR drafting, and rollout/rollback notes, all bounded by existing acceptance rules in `AGENTS.md`.

## Problem Statement / Motivation

This repository is explicitly spec-driven, issue-driven, and test-driven. That rigor is valuable, but it creates ceremony:

- issue creation and hierarchy setup
- milestone and label assignment
- `spec.md`, `plan.md`, and `tasks.md`
- conformance case mapping
- verification evidence and PR metadata

Tau already has several adjacent capabilities: GitHub integrations, planning/orchestration, roadmap status tooling, issue templates, and strong written conventions. The missing product layer is an autopilot that performs the flow reliably while preserving human checkpoints where required.

## Proposed Solution

Add a spec-to-PR autopilot workflow that:

1. Starts from a mission or operator request.
2. Creates or links the appropriate GitHub issue and hierarchy.
3. Generates `spec.md`, `plan.md`, and `tasks.md` in repository-conformant form.
4. Maps acceptance criteria to conformance and test surfaces.
5. Tracks lifecycle state transitions (`specifying`, `planning`, `implementing`, `done`).
6. Drafts PR metadata including AC mapping, test matrix, risks, and rollback notes.
7. Enforces human review/acceptance thresholds from `AGENTS.md` instead of bypassing them.

## Implementation Phases

### Phase 1: Governance Model Encoding

- Formalize repository rules from `AGENTS.md` into machine-usable policy.
- Encode issue-type, label, milestone, and acceptance-threshold rules.
- Add validation for missing or inconsistent issue/spec metadata.

### Phase 2: Artifact Generation

- Generate `spec.md`, `plan.md`, and `tasks.md` from mission/issue input.
- Support regeneration/update without destructive rewrites.
- Map ACs to conformance cases and test expectations.

### Phase 3: Tracker and PR Integration

- Integrate with GitHub issue creation and hierarchy maintenance.
- Draft PR bodies with AC/test matrix/risks/rollback sections.
- Link roadmap and issue status sync artifacts where relevant.

### Phase 4: Operator Review Workflow

- Add checkpoints for self-accept vs human-accept rules.
- Surface unresolved questions, missing evidence, or blocked conditions.
- Provide audit logs for generated governance actions.

## Technical Considerations

- The autopilot must treat `AGENTS.md` as policy, not inspiration.
- Generated docs need idempotent updates; repeated runs should refine rather than churn.
- GitHub auth and remote failure modes must be handled as first-class operator states.
- The system should not silently create hierarchy drift or duplicate issues when metadata already exists.

## System-Wide Impact

### Interaction Graph

Mission intake produces governance intent, tracker integration creates or updates issue state, artifact generation writes `spec.md`/`plan.md`/`tasks.md`, implementation uses those artifacts, and PR drafting summarizes resulting evidence back into the governance flow.

### Error & Failure Propagation

Tracker failures, missing milestone/index references, or unresolved acceptance blockers must halt the autopilot in an explicit blocked state rather than producing partial governance artifacts that look complete.

### State Lifecycle Risks

Partially generated specs, duplicate issues, or stale plan/task docs can create governance confusion. The workflow needs clear rerun semantics and conflict detection.

### API Surface Parity

The autopilot should work consistently whether invoked from CLI, TUI mission control, or future dashboard surfaces.

### Integration Test Scenarios

- Create a task from scratch and verify issue, labels, milestone, and local spec artifacts all match repo rules.
- Resume an in-progress issue and verify the autopilot updates docs without duplicating hierarchy or clobbering manual edits.
- Draft a PR summary from a completed mission and verify AC/test matrix population is complete and measurable.

## SpecFlow Notes

### Primary Operator Flows

1. Start with a new idea and let Tau scaffold issue plus local spec artifacts.
2. Resume an issue and regenerate or refine plan/task docs safely.
3. Move from implemented work to PR-ready evidence and closeout summary.
4. Stop at human review checkpoints when required by priority or scope.

### Important Gaps to Resolve in Implementation

- Whether autopilot creates issues before writing specs or stages local artifacts first when remote access is unavailable.
- How manual edits to generated docs are preserved across reruns.
- How the autopilot determines priority and acceptance thresholds when the operator input is ambiguous.

### Default Planning Assumptions

- GitHub is the primary tracker for this repository.
- Human acceptance thresholds in `AGENTS.md` remain binding and are not relaxed.
- Generated governance artifacts are editable by humans and re-runnable by the system.

## Acceptance Criteria

- [ ] The autopilot can generate repository-conformant issue/spec/plan/task artifacts from a single mission or operator request.
- [ ] Issue hierarchy, labels, and milestone handling match the repository contract in `AGENTS.md`.
- [ ] Generated specs include measurable ACs and conformance-oriented structure.
- [ ] Generated PR drafts include AC mapping, test tiers, risks/rollback notes, and artifact links.
- [ ] The workflow halts explicitly when human review/acceptance is required.
- [ ] Reruns update existing artifacts safely without duplicate issue creation or unnecessary churn.
- [ ] Integration tests cover remote tracker failure, rerun behavior, and acceptance-threshold enforcement.

## Success Metrics

- The cost of starting a correctly governed issue decreases substantially.
- Spec and PR quality become more uniform across autonomous and human-led work.
- Operators spend less time on process scaffolding and more time on review-worthy decisions.

## Dependencies & Risks

### Dependencies

- [ ] GitHub issue/runtime integration.
- [ ] Mission model or equivalent work object for end-to-end automation.
- [ ] Policy encoding from `AGENTS.md` and issue template metadata.

### Risks

- Governance automation can become brittle if it overfits current templates.
- Over-automation may create false confidence if blockers are not surfaced early enough.
- Repo-specific conventions may drift unless the autopilot validates against current docs and templates on each run.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- Repository contract: `AGENTS.md`
- Issue templates: `.github/ISSUE_TEMPLATE/epic.md`
- Issue templates: `.github/ISSUE_TEMPLATE/story.md`
- Issue templates: `.github/ISSUE_TEMPLATE/task.md`
- Issue templates: `.github/ISSUE_TEMPLATE/subtask.md`
- GitHub issues runtime crate: `crates/tau-github-issues`
- Roadmap governance index: `docs/guides/roadmap-execution-index.md`
- Roadmap sync automation: `docs/guides/roadmap-status-sync.md`
- Transport guide for issue and control surfaces: `docs/guides/transports.md`
