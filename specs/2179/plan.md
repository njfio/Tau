# Plan #2179

Status: Implemented
Spec: specs/2179/spec.md

## Approach

1. Add RED wave-10 marker assertions to
   `scripts/dev/test-split-module-rustdoc.sh`.
2. Run guard script and capture expected failure.
3. Add concise rustdoc comments to wave-10 modules.
4. Run guard, scoped checks, and targeted tests for both crates.

## Affected Modules

- `specs/milestones/m37/index.md`
- `specs/2179/spec.md`
- `specs/2179/plan.md`
- `specs/2179/tasks.md`
- `scripts/dev/test-split-module-rustdoc.sh`
- `crates/tau-release-channel/src/command_runtime.rs`
- `crates/tau-release-channel/src/command_runtime/update_state.rs`
- `crates/tau-skills/src/package_manifest/schema.rs`
- `crates/tau-skills/src/package_manifest/validation.rs`

## Risks and Mitigations

- Risk: marker assertions become brittle.
  - Mitigation: assert stable phrases tied to API intent names.
- Risk: docs edit accidentally changes behavior.
  - Mitigation: docs-only line additions plus scoped compile/test matrix.

## Interfaces and Contracts

- Guard:
  `bash scripts/dev/test-split-module-rustdoc.sh`
- Compile:
  `cargo check -p tau-release-channel --target-dir target-fast`
  `cargo check -p tau-skills --target-dir target-fast`
- Targeted tests:
  `cargo test -p tau-release-channel functional_execute_release_channel_command_plan_dry_run_writes_update_state --target-dir target-fast`
  `cargo test -p tau-release-channel regression_execute_release_channel_plan_fails_closed_on_malformed_update_state --target-dir target-fast`
  `cargo test -p tau-skills unit_validate_package_manifest_accepts_minimal_semver_shape --target-dir target-fast`
  `cargo test -p tau-skills regression_validate_package_manifest_rejects_invalid_remote_url_or_checksum --target-dir target-fast`

## ADR References

- Not required.
