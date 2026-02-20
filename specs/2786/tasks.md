# Tasks: Issue #2786 - PRD Phase 1B auth bootstrap and protected route shell markers

## Ordered Tasks
1. [x] T1 (RED): add failing tests for auth bootstrap endpoint and auth-aware `/ops`/`/ops/login` shell markers.
2. [x] T2 (GREEN): implement `tau-dashboard-ui` auth/route context render contracts and unit coverage.
3. [x] T3 (GREEN): implement gateway auth bootstrap endpoint and route wiring for `/ops/login` + context-aware `/ops`.
4. [x] T4 (REGRESSION): run existing `/dashboard` and `/gateway/auth/session` regression tests.
5. [x] T5 (VERIFY): run scoped fmt/clippy/tests and set spec status to `Implemented`.

## Tier Mapping
- Unit: `tau-dashboard-ui` auth marker tests.
- Property: N/A (no parser/invariant expansion in this slice).
- Contract/DbC: N/A (no new DbC macro surfaces).
- Snapshot: N/A.
- Functional: auth bootstrap JSON contract tests.
- Conformance: C-01..C-05 mapped in crate/gateway tests.
- Integration: `/ops` and `/ops/login` endpoint tests.
- Fuzz: N/A (no untrusted parser added).
- Mutation: N/A (scaffolding contract slice; no critical algorithm path).
- Regression: existing `/dashboard` and auth session tests.
- Performance: N/A (no hot-path runtime changes).
