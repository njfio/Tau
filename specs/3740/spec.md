# Spec: Issue #3740 - Restore tau-provider cli_executable clippy hygiene

Status: Implemented

## Problem Statement
The `tau-provider` quality gate currently fails because `crates/tau-provider/src/cli_executable.rs`
contains dead helper symbols that are no longer referenced after recent provider hardening work. Those symbols block `cargo clippy -p tau-provider --all-targets --all-features -- -D warnings`, which in turn
blocks unrelated PRs in the CI quality lane.

## Scope
In scope:
- `crates/tau-provider/src/cli_executable.rs`
- `crates/tau-provider/src/credential_store.rs`
- `specs/3740/spec.md`
- `specs/3740/plan.md`
- `specs/3740/tasks.md`

Out of scope:
- changing provider CLI subprocess behavior
- reworking CLI client environment sanitization implementations
- modifying auth or credential flows

## Acceptance Criteria
### AC-1 Dead-code clippy failure is removed
Given the provider executable helper module is compiled with all targets and features,
when `cargo clippy -p tau-provider --all-targets --all-features -- -D warnings` runs,
then it passes without dead-code failures from `cli_executable.rs`.

### AC-2 Executable discovery behavior remains unchanged
Given the public `is_executable_available` helper,
when the existing absolute-path and PATH-lookup tests run,
then they still pass without behavior regressions.

## Conformance Cases
- C-01 / AC-1 / Functional: `cargo clippy -p tau-provider --all-targets --all-features -- -D warnings`
  passes after removing the dead code.
- C-02 / AC-2 / Regression: `cargo test -p tau-provider is_executable_available -- --test-threads=1`
  passes after the hygiene-only change.

## Success Metrics / Observable Signals
- The `tau-provider` clippy lane is green again.
- `cli_executable.rs` retains only live helper code.
- No provider executable-discovery tests regress.
