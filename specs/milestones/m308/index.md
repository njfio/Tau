# M308 - Dashboard live mutation depth verification wave

Status: Active

## Context
M308 adds a deterministic dashboard live mutation verification gate so
operators can validate end-to-end dashboard control/status/stream contracts
with one auditable command and report artifact.

Primary sources:
- `docs/guides/dashboard-ops.md`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `scripts/demo/dashboard.sh`

## Issue Hierarchy
- Epic: #3488
- Story: #3489
- Task: #3490

## Scope
- Add deterministic M308 dashboard verification script and report output.
- Add contract test that enforces report schema and required-step inventory.
- Map dashboard live mutation coverage to executable selectors.
- Update README verification links.

## Exit Criteria
- `specs/3490/spec.md` is `Implemented` with AC-to-test evidence.
- M308 script passes and emits a deterministic report artifact.
- Contract test fails closed on missing required steps.
- README references M308 verification entrypoint.
