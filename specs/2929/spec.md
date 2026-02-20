# Spec: Issue #2929 - Refactor gateway_openresponses into maintainable submodules

Status: Reviewed

## Problem Statement
`crates/tau-gateway/src/gateway_openresponses.rs` is a 4,300-line module that continues to accumulate dashboard and memory runtime logic. The oversized-file exemption is temporary and raises maintenance and review risk.

## Scope
In scope:
- Extract dashboard handlers from `gateway_openresponses.rs` into a dedicated submodule.
- Extract memory handlers and memory graph helper logic into a dedicated submodule.
- Keep route wiring and response behavior stable.
- Retire or tighten oversized-file policy exemption once the module is reduced.

Out of scope:
- Functional behavior changes for dashboard/memory APIs.
- New endpoint surfaces.

## Acceptance Criteria
### AC-1 Dashboard runtime handlers are modularized
Given the gateway module,
when dashboard endpoints are implemented,
then dashboard handlers live in `gateway_openresponses/dashboard_runtime.rs` and are wired through the existing router without behavior regressions.

### AC-2 Memory runtime handlers/helpers are modularized
Given the gateway module,
when memory endpoints are implemented,
then memory handlers and memory graph/storage helper logic live in `gateway_openresponses/memory_runtime.rs` and existing behavior remains unchanged.

### AC-3 Oversized-file debt is reduced and policy is updated
Given the refactor,
when line counts and policy artifacts are checked,
then `gateway_openresponses.rs` is materially reduced and `tasks/policies/oversized-file-exemptions.json` no longer carries an outdated exemption for this split work.

### AC-4 Regression and quality gates remain green
Given the modularization,
when targeted regression tests and quality gates run,
then tests pass and no new warnings/regressions are introduced.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | gateway routes | hit dashboard endpoints | endpoint behavior and status codes remain unchanged after module extraction |
| C-02 | AC-2 | Functional | gateway memory endpoints | hit memory CRUD/graph endpoints | endpoint behavior and payload shapes remain unchanged after module extraction |
| C-03 | AC-3 | Conformance | line count + exemption policy | inspect file size/policy artifact | oversized-file debt is reduced and policy artifact matches new state |
| C-04 | AC-4 | Regression | refactored gateway crate | run scoped tests + fmt/clippy | suites stay green |

## Success Metrics / Signals
- `wc -l crates/tau-gateway/src/gateway_openresponses.rs` is below the prior 4300-line exemption ceiling.
- `tasks/policies/oversized-file-exemptions.json` is updated to remove/adjust the stale exemption.
- `cargo test -p tau-gateway gateway_openresponses::tests::` suite remains green.
- `cargo fmt --check` and scoped `cargo clippy ... -D warnings` pass.
