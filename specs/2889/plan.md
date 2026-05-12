# Plan: Issue #2889 - session reset confirmation and clear-session contracts

## Approach
1. Add additive reset-confirmation form markers to session detail panel in `tau-dashboard-ui`.
2. Add a browser-native confirmation guard scoped to session detail reset/branch submit controls.
3. Add gateway ops reset form parser and POST handler on session detail route path.
4. Reuse existing session file deletion semantics (clear session + lock cleanup), then redirect to deterministic detail URL with theme/sidebar query contracts.
5. Add RED UI/gateway conformance tests for markers, reset redirect, cleared state, non-target isolation, and browser confirmation guard presence.
6. Run regression suites and verification gates before PR.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: route wiring changes could exceed oversized-file threshold in `gateway_openresponses.rs`.
  - Mitigation: keep net line count neutral; compact existing lines where needed.
- Risk: reset flow could accidentally clear non-target sessions.
  - Mitigation: targeted path-based session key resolution and explicit integration tests for isolation.
- Risk: confirmation markers can become decorative without an installed submit guard.
  - Mitigation: assert the scoped session confirmation guard script and `window.confirm` path in UI and gateway output.
- Risk: brittle HTML assertions.
  - Mitigation: assert deterministic IDs/data attributes and key route contracts only.

## Interface / Contract Notes
- Add POST behavior on existing session detail route contract path: `/ops/sessions/{session_key}`.
- Add `#tau-ops-session-confirmation-guard` scoped to `#tau-ops-session-detail-panel`.
- No schema/protocol changes to existing gateway JSON APIs.
- UI contract additions are additive.
