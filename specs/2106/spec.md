# Spec #2106

Status: Implemented
Milestone: specs/milestones/m28/index.md
Issue: https://github.com/njfio/Tau/issues/2106

## Problem Statement

Several newly split helper modules still expose undocumented public APIs, which
reduces maintainability and weakens documentation trajectory goals. We need a
bounded first-wave rustdoc pass with a regression check to prevent accidental
removal of that baseline coverage.

## Acceptance Criteria

- AC-1: Add `///` rustdoc comments for public helper APIs in the scoped first-wave files:
  - `crates/tau-github-issues/src/issue_runtime_helpers.rs`
  - `crates/tau-github-issues/src/issue_command_usage.rs`
  - `crates/tau-ai/src/retry.rs`
  - `crates/tau-runtime/src/slack_helpers_runtime.rs`
- AC-2: Add a task-scoped regression script that asserts required rustdoc
  marker presence for the first-wave files.
- AC-3: Task-scoped validation commands pass with no compile/test regressions
  in affected crates.

## Scope

In:

- rustdoc additions for public functions/constants in first-wave split modules
- add/update scoped regression script under `scripts/dev/`
- run targeted compile/test/doc checks for touched crates

Out:

- repository-wide rustdoc parity target completion
- behavior or interface semantics changes beyond documentation/test guardrails

## Conformance Cases

- C-01 (AC-1, unit): all scoped first-wave files contain rustdoc comments on
  their public helper APIs touched by this subtask.
- C-02 (AC-2, regression): new/updated script fails when required rustdoc
  markers are missing and passes when present.
- C-03 (AC-3, functional): `cargo check -p tau-github-issues --target-dir target-fast`,
  `cargo check -p tau-ai --target-dir target-fast`, and
  `cargo check -p tau-runtime --target-dir target-fast` pass.
- C-04 (AC-3, integration): targeted tests in touched modules pass.

## Success Metrics

- Subtask `#2106` merges with bounded rustdoc additions and guard script.
- First-wave scoped files no longer have zero rustdoc markers.
- Conformance suite C-01..C-04 passes.
