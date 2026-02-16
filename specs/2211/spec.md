# Spec #2211

Status: Implemented
Milestone: specs/milestones/m41/index.md
Issue: https://github.com/njfio/Tau/issues/2211

## Problem Statement

`README.md` currently describes true RL as purely future work tracked in
closed tracker artifacts, which is ambiguous against current repository state.
The training boundary and status language must be updated to remain accurate.

## Acceptance Criteria

- AC-1: Update README training boundary wording so it clearly states:
  - prompt optimization is the current canonical CLI training mode
  - true-RL components/artifacts exist in-repo (e.g., PPO/GAE + benchmark/safety scripts)
  - historical tracker links are identified as closed delivery records
- AC-2: Updated README links/paths referenced by touched text resolve in repo.
- AC-3: Docs quality/format checks used for docs-only updates pass.

## Scope

In:

- targeted updates to `README.md` training roadmap boundary section
- verification of touched references and docs quality gates

Out:

- editing multiple downstream guides unless required for broken links
- behavior/code changes outside documentation

## Conformance Cases

- C-01 (AC-1, functional): `README.md` includes updated, unambiguous training boundary language.
- C-02 (AC-1, regression): README no longer states true RL is only future-tracked under active issue/milestone.
- C-03 (AC-2, conformance): touched README links resolve to existing paths/artifacts.
- C-04 (AC-3, regression): `cargo fmt --check` and docs quality PR checks pass.

## Success Metrics

- Subtask `#2211` merges with README section accuracy corrected.
- Users reading README can correctly distinguish current prompt optimization mode
  from delivered true-RL components and historical roadmap tracking.
