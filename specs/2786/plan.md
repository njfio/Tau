# Plan: Issue #2786 - PRD Phase 1B auth bootstrap and protected route shell markers

## Approach
1. Capture RED by adding tests for missing auth bootstrap endpoint and auth/route shell markers.
2. Add auth-aware shell context in `tau-dashboard-ui` with deterministic marker output for login/protected sections.
3. Add gateway `GET /gateway/auth/bootstrap` endpoint and `/ops/login` route, and pass auth/route context into `/ops` render path.
4. Re-run scoped regression tests for `/dashboard` shell and auth session endpoint.
5. Run scoped fmt/clippy/tests and update spec/task status.

## Affected Modules
- `specs/milestones/m130/index.md` (new)
- `specs/2786/spec.md` (new)
- `specs/2786/plan.md` (new)
- `specs/2786/tasks.md` (new)
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: introducing unauthenticated auth metadata endpoint could accidentally leak secrets.
  - Mitigation: return only mode and endpoint metadata, no configured token/password values.
- Risk: route changes may regress existing `/ops` shell markers.
  - Mitigation: preserve existing marker assertions and add route-specific integration tests.
- Risk: auth mode naming mismatch between gateway internals and PRD (`localhost-dev` vs `none`).
  - Mitigation: expose both raw mode and a UI-normalized mode field.

## Interface and Contract Notes
- New endpoint: `GET /gateway/auth/bootstrap`.
- New shell route: `GET /ops/login`.
- `tau-dashboard-ui` gains auth/route context render interface while preserving existing `render_tau_ops_dashboard_shell()` compatibility.
