# Spec: Issue #2609 - Expand under-tested crate coverage wave

Status: Implemented

## Problem Statement
Current coverage in several M104-targeted crates is shallow relative to behavior criticality, especially in diagnostics command parsing/report surfaces, training-proxy attribution/error paths, and provider client auth-resolution helpers. This leaves correctness regressions likely in operator-facing and routing-critical code paths.

## Acceptance Criteria

### AC-1 Diagnostics command/audit utility error paths are covered
Given `tau-diagnostics` command and summary utility helpers,
When invalid flags, duplicate flags, and edge percentile/report rendering cases are exercised,
Then deterministic unit/regression tests assert fail-closed usage behavior and expected summary formatting outputs.

### AC-2 Training proxy attribution/forwarding error-path behavior is covered
Given `tau-training-proxy` request parsing and forwarding helpers,
When invalid attribution headers and upstream non-2xx responses are exercised,
Then tests verify correct rejection semantics, status propagation, and attribution logging fields.

### AC-3 Provider client auth helper decisions are covered
Given `tau-provider::client` auth helper logic,
When Azure endpoint detection and credential-secret resolution edge cases are exercised,
Then tests assert deterministic helper outcomes and fail-closed behavior for missing credential payloads.

### AC-4 Scoped verification gates stay green
Given the new test coverage additions,
When scoped lint/test gates run,
Then formatting, clippy, and crate-scoped tests pass with no new warnings.

## Scope

### In Scope
- Add/extend tests in:
  - `crates/tau-diagnostics/src/lib.rs`
  - `crates/tau-training-proxy/src/lib.rs`
  - `crates/tau-provider/src/client.rs`
- Focus on AC-mapped unit/functional/regression error-path coverage only.

### Out of Scope
- Refactoring runtime production logic not required by failing tests.
- New dependencies, protocol changes, or architectural redesign.
- Broad cross-crate integration suite work (covered separately by #2608).

## Conformance Cases
- C-01 (AC-1, unit): `unit_spec_2609_c01_diagnostics_doctor_arg_parser_and_policy_usage_fail_closed`
- C-02 (AC-1, regression): `regression_spec_2609_c02_diagnostics_percentile_and_render_edges`
- C-03 (AC-2, regression): `regression_spec_2609_c03_training_proxy_rejects_invalid_sequence_header`
- C-04 (AC-2, integration): `integration_spec_2609_c04_training_proxy_propagates_upstream_error_status_and_logs`
- C-05 (AC-3, unit): `unit_spec_2609_c05_provider_client_auth_helper_decisions`
- C-06 (AC-4, verify): scoped crate test/lint commands listed in tasks.

## Success Metrics / Observable Signals
- New AC-mapped tests pass in targeted crates.
- Scoped commands are green:
  - `cargo fmt --check`
  - `cargo clippy -p tau-diagnostics -p tau-training-proxy -p tau-provider -- -D warnings`
  - `cargo test -p tau-diagnostics -p tau-training-proxy -p tau-provider`
