# Spec #2179

Status: Implemented
Milestone: specs/milestones/m37/index.md
Issue: https://github.com/njfio/Tau/issues/2179

## Problem Statement

Wave-10 release-channel/package-manifest split helper modules still expose
public APIs without rustdoc markers, and the split-module rustdoc guard does
not enforce marker presence for this module set.

## Acceptance Criteria

- AC-1: Add `///` rustdoc comments for key public APIs in:
  - `crates/tau-release-channel/src/command_runtime.rs`
  - `crates/tau-release-channel/src/command_runtime/update_state.rs`
  - `crates/tau-skills/src/package_manifest/schema.rs`
  - `crates/tau-skills/src/package_manifest/validation.rs`
- AC-2: Extend `scripts/dev/test-split-module-rustdoc.sh` with wave-10 marker
  assertions for these files.
- AC-3: Scoped compile/test matrix passes for affected crates.

## Scope

In:

- rustdoc additions for wave-10 modules listed above
- guard script assertion expansion for wave-10 markers
- scoped compile/test verification for `tau-release-channel` and `tau-skills`

Out:

- broader documentation rewrites outside scoped files
- non-documentation behavior changes

## Conformance Cases

- C-01 (AC-1, functional): all four wave-10 files contain expected rustdoc
  marker phrases for key public APIs.
- C-02 (AC-2, regression): `bash scripts/dev/test-split-module-rustdoc.sh`
  fails with missing markers and passes after docs are added.
- C-03 (AC-3, functional):
  `cargo check -p tau-release-channel --target-dir target-fast` and
  `cargo check -p tau-skills --target-dir target-fast` pass.
- C-04 (AC-3, integration): targeted release-channel/package-manifest tests pass.

## Success Metrics

- Subtask `#2179` merges with bounded docs + guard updates.
- Wave-10 files no longer appear in zero-doc helper list.
- Conformance suite C-01..C-04 passes.
