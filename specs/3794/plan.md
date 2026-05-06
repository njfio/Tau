# Issue 3794 Plan

## Approach

Replace the bare proposal diff fragment with a self-contained styled HTML page.
Keep the patch deterministic and fallback-backed, but make the review page
usable: title, metadata grid, policy chips, diff pre block, and back link.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/ops_config_training_safety.rs`

## Risks

- The route is currently simple and tests only check basic markers; stronger
  HTML assertions could be brittle if formatting changes.

## Mitigations

- Assert durable `data-*` markers and core visible content rather than exact
  full-page HTML.
- Keep the route standalone and deterministic.

## Interfaces

No API, route path, schema, or wire-format changes.
