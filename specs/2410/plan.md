# Plan: Issue #2410 - Fly.io deployment baseline for Tau gateway

## Approach
1. Add `tau-deployment` Fly manifest contract validator with test coverage for AC-1..AC-3.
2. Run RED by executing new conformance tests before adding `fly.toml` and docs updates.
3. Add repository-root `fly.toml` with gateway-mode defaults and health checks.
4. Update `docs/guides/deployment-ops.md` with Fly launch/deploy/verify steps.
5. Re-run scoped quality gates and conformance tests (GREEN).

## Affected Modules
- `crates/tau-deployment/src/lib.rs`
- `crates/tau-deployment/src/fly_manifest_contract.rs` (new)
- `fly.toml` (new)
- `docs/guides/deployment-ops.md`

## Risks / Mitigations
- Risk: Fly manifest fields drift from runtime expectations.
  - Mitigation: enforce deterministic required-field checks with conformance tests.
- Risk: Docs and manifest diverge.
  - Mitigation: regression test asserts runbook references Fly workflow and manifest path.

## Interfaces / Contracts
- New deployment helper API:
  - `load_repo_fly_manifest()`
  - `validate_fly_manifest_contract(raw: &str)`

## ADR
- Not required; no architecture/dependency/protocol changes.
