# Plan: Issue #3478 - M305 command-center last-action detail rows

Status: Implemented

## Approach
1. Add RED tests:
   - `tau-dashboard-ui` functional/regression assertions for rendered Last Action
     detail row elements and fallback values.
   - gateway integration assertion for Last Action readable row markers.
2. Implement UI rendering in `tau-dashboard-ui`:
   - add detail `<p>` rows under `tau-ops-control-last-action`,
   - bind existing command-center metadata values to row content.
3. Preserve existing `data-*` marker attributes for compatibility.
4. Run scoped tests + quality checks and update spec evidence.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/milestones/m305/index.md`
- `specs/3478/spec.md`
- `specs/3478/plan.md`
- `specs/3478/tasks.md`

## Risks / Mitigations
- Risk: HTML contract change breaks existing marker selectors.
  - Mitigation: retain current IDs/data-attributes and add new rows additive.
- Risk: empty values slip into UI for no-action path.
  - Mitigation: explicit fallback assertions in regression tests.
- Risk: gateway integration diverges from dashboard-ui render expectations.
  - Mitigation: mirror row assertions in gateway integration test.

## Interfaces / Contracts
- Last Action section contract (additive):
  - container: `id="tau-ops-control-last-action"`
  - detail rows:
    - `id="tau-ops-last-action-request-id"`
    - `id="tau-ops-last-action-name"`
    - `id="tau-ops-last-action-actor"`
    - `id="tau-ops-last-action-timestamp"`

## ADR
No ADR required (UI rendering + tests only).

## Execution Summary
1. Added RED tests for Last Action readable detail rows in:
   - `crates/tau-dashboard-ui/src/tests.rs` (functional + regression),
   - `crates/tau-gateway/src/gateway_openresponses/tests.rs` (integration).
2. Implemented additive UI rendering in `tau-dashboard-ui`:
   - `tau-ops-last-action-request-id`,
   - `tau-ops-last-action-name`,
   - `tau-ops-last-action-actor`,
   - `tau-ops-last-action-timestamp`.
3. Preserved existing `tau-ops-control-last-action` `data-*` markers for
   compatibility while adding operator-readable text rows.
4. Verified fallback contract uses deterministic existing defaults (`none`/`0`).
5. Ran scoped regression/gates for dashboard-ui + gateway surfaces.

## Verification Notes
- RED evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` failed for missing readable row elements.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` failed for missing readable row markers in `/ops`.
- GREEN evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` passed.
- Regression/gate evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-dashboard-ui -p tau-gateway --tests --no-deps -- -D warnings` passed.
  - `cargo fmt --check` passed.
