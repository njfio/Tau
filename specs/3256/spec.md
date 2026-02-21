# Spec: Issue #3256 - move openresponses preflight helpers to dedicated module

Status: Implemented

## Problem Statement
`gateway_openresponses.rs` still owns preflight helper functions used by openresponses request handling (auth/rate-limit checks, body-size validation, JSON parse, policy gate construction, timestamp conversion). Keeping these helpers in root increases module density and obscures runtime boundaries.

## Scope
In scope:
- Move preflight helper functions from root into `gateway_openresponses/request_preflight.rs`.
- Preserve helper behavior and downstream request handling semantics.
- Ratchet and enforce root-module size/ownership guard.

Out of scope:
- Endpoint/path changes.
- Policy limit semantics changes.
- Session/auth model changes.

## Acceptance Criteria
### AC-1 preflight behavior remains stable
Given existing preflight conformance/regression tests,
when tests run,
then over-budget requests are blocked, provider dispatch is skipped on blocked requests, and successful response schema remains unchanged.

### AC-2 root module ownership boundaries improve
Given refactored module layout,
when root guard runs,
then root line count is under tightened threshold and moved preflight helper function definitions are no longer declared in root.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Integration/Conformance | over-budget preflight fixture | `integration_spec_c01_openresponses_preflight_blocks_over_budget_request` | preflight blocks request with expected contract |
| C-02 | AC-1 | Integration/Conformance | provider-skip preflight fixture | `integration_spec_c02_openresponses_preflight_skips_provider_dispatch` | provider dispatch remains skipped on blocked request |
| C-03 | AC-1 | Regression/Conformance | success-schema preflight fixture | `regression_spec_c03_openresponses_preflight_preserves_success_schema` | successful response schema unchanged |
| C-04 | AC-2 | Functional/Regression | repo checkout | `scripts/dev/test-gateway-openresponses-size.sh` | tightened threshold + ownership checks pass |

## Success Metrics / Observable Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway integration_spec_c01_openresponses_preflight_blocks_over_budget_request`
- `cargo test -p tau-gateway integration_spec_c02_openresponses_preflight_skips_provider_dispatch`
- `cargo test -p tau-gateway regression_spec_c03_openresponses_preflight_preserves_success_schema`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
