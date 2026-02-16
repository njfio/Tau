# Spec #2104

Status: Implemented
Milestone: specs/milestones/m28/index.md
Issue: https://github.com/njfio/Tau/issues/2104

## Problem Statement

Story M28.1 tracks first-wave documentation for split modules with guardrails.
Task/subtask delivery (`#2105/#2106`) is complete via PRs `#2108/#2107`.
This story closes by consolidating AC/conformance evidence.

## Acceptance Criteria

- AC-1: First-wave split modules have documented public helper APIs.
- AC-2: Documentation guard checks prevent regression for scoped files.
- AC-3: Story-level artifacts map ACs to concrete conformance/test evidence.

## Scope

In:

- consume merged outputs from `#2105` and `#2106`
- publish story-level lifecycle artifacts and mapping
- rerun scoped verification commands on latest `master`

Out:

- additional documentation waves beyond M28.1
- runtime behavior changes unrelated to docs baseline

## Conformance Cases

- C-01 (AC-1, functional): scoped files contain expected rustdoc marker phrases.
- C-02 (AC-2, regression): `bash scripts/dev/test-split-module-rustdoc.sh` passes.
- C-03 (AC-2, functional): compile checks pass for touched crates.
- C-04 (AC-3, integration): targeted module tests pass in touched crates.

## Success Metrics

- Story issue `#2104` closes with linked task/subtask evidence.
- `specs/2104/{spec,plan,tasks}.md` lifecycle is complete.
- Epic roll-up `#2103` is unblocked.
