# Spec: Issue #2410 - Fly.io deployment baseline for Tau gateway

Status: Accepted

## Problem Statement
Tau currently has Docker and release automation artifacts but no first-party `fly.toml` deployment manifest. Operators must hand-roll Fly.io configuration, which causes drift and inconsistent gateway startup behavior.

## Acceptance Criteria

### AC-1 Fly manifest exists with gateway startup contract
Given repository root configuration,
When loading `fly.toml`,
Then the manifest exists and includes required gateway startup contract entries (`Dockerfile` build, gateway transport mode, OpenResponses server flag, and bind address for Fly internal port routing).

### AC-2 Fly service exposure and health checks are defined
Given repository root `fly.toml`,
When validating service settings,
Then internal/exposed ports and HTTP health-check path are configured for gateway operations.

### AC-3 Deployment guide contains repeatable Fly workflow
Given deployment operations docs,
When operators follow the runbook,
Then launch/deploy/health verification commands for Fly.io are documented with explicit `fly.toml` usage.

## Scope

### In Scope
- Add `fly.toml` at repository root for gateway mode.
- Add deployment conformance validation logic/tests in `tau-deployment`.
- Update deployment runbook docs for Fly.io workflow.

### Out of Scope
- CI-managed Fly deployment rollout.
- Runtime gateway protocol changes.
- Fly secrets provisioning automation.

## Conformance Cases
- C-01 (AC-1, functional): `spec_c01_load_repo_fly_manifest_contains_gateway_startup_contract`
- C-02 (AC-2, integration): `spec_c02_validate_fly_manifest_contract_requires_service_and_health_checks`
- C-03 (AC-3, regression): `regression_spec_c03_deployment_runbook_documents_fly_workflow`

## Success Metrics / Observable Signals
- `fly.toml` is present and validated by conformance tests.
- `cargo test -p tau-deployment fly_manifest_contract` passes.
- `cargo fmt --check` and `cargo clippy -p tau-deployment -- -D warnings` pass.
