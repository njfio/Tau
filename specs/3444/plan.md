# Plan: Issue #3444 - M293 operator integration closure set

Status: Implemented

1. Capture RED for docs contract (`scripts/dev/test-docs-capability-archive.sh`) and preserve failing marker evidence.
2. Fix README contract markers with exact required strings while preserving current capability structure.
3. Implement TUI live shell mode:
   - parse dashboard artifact files (`state.json`, `runtime-events.jsonl`, `control-state.json`, `actions-audit.jsonl`) from configurable state dir,
   - map artifact data to `OperatorShellFrame`,
   - add CLI mode/options and tests.
4. Implement dashboard ops control-action live flow:
   - add `/ops` control action POST handler in gateway ops shell runtime,
   - route shell control forms to handler,
   - invoke existing dashboard action runtime and redirect with existing route controls.
5. Expand readiness verification for RL hardening + live auth validation:
   - add scripted checks with pass/skip reporting semantics,
   - wire into M296 readiness report,
   - update runbook docs.
6. Verify with targeted crates/scripts and capture RED/GREEN/REGRESSION evidence in tasks doc.

## Affected Modules
- `README.md`
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs` (targeted additions)
- `scripts/verify/m296-ga-readiness-gate.sh`
- `docs/guides/m296-ga-readiness-gate.md`
- `specs/milestones/m293/index.md`

## Risks / Mitigations
- Risk: route wiring regressions in gateway ops shell.
  - Mitigation: keep handler isolated to control-action path and add targeted integration test.
- Risk: live-auth checks become non-deterministic in CI.
  - Mitigation: use explicit env-gated execution and deterministic skip reporting.
- Risk: TUI live mode becomes brittle to missing artifact files.
  - Mitigation: add fallback behavior and explicit error messaging with fixture-mode compatibility retained.

## Interfaces / Contracts
- New ops shell POST endpoint for control actions under `/ops` path family (form-submit contract).
- New `tau-tui` shell-live CLI mode/options for state-dir ingestion.
- M296 report step additions for RL hardening and live-auth matrix validation.
