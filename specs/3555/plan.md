# Plan: Issue #3555 - Wire interactive request timeout/retry policy into local runtime + unified TUI

Status: Implemented

## Approach
1. Update local runtime startup wiring in `tau-coding-agent` so
   `LocalRuntimeAgentSettings.request_timeout_ms` derives from
   `cli.request_timeout_ms` (bounded to positive).
2. Extend `tau-tui agent` arguments/help/parser and command builder with:
   - `--request-timeout-ms`
   - `--agent-request-max-retries`
3. Extend `tau-unified.sh tui` with fast-fail interactive defaults and
   pass-through options:
   - `--request-timeout-ms`
   - `--agent-request-max-retries`
   plus env defaults for non-flag usage.
4. Add RED-first tests:
   - local runtime timeout enforcement regression
   - `tau-tui` parser/command contract assertions
   - launcher runner-mode assertions for defaults/overrides
5. Update README/run docs to reflect new controls and expected behavior.

## Affected Modules
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-tui/src/main.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `README.md` (or runbook section covering unified launcher/TUI)

## Risks / Mitigations
- Risk: defaults become too aggressive for slower networks.
  - Mitigation: explicit flags/env overrides and docs with tuning guidance.
- Risk: changing launch command args breaks existing smoke tests.
  - Mitigation: update command contract tests in `tau-tui` and launcher tests.
- Risk: timeout regression test flakiness.
  - Mitigation: use deterministic slow client + short timeout + bounded
    harness timeout.

## Interfaces / Contracts
- New/extended `tau-tui agent` flags:
  - `--request-timeout-ms <ms>`
  - `--agent-request-max-retries <n>`
- New/extended `tau-unified.sh tui` flags/env:
  - `--request-timeout-ms <ms>`
  - `--agent-request-max-retries <n>`
  - `TAU_UNIFIED_TUI_REQUEST_TIMEOUT_MS`
  - `TAU_UNIFIED_TUI_AGENT_REQUEST_MAX_RETRIES`

## ADR
No ADR required. This is runtime policy wiring and launcher UX hardening
without dependency or protocol changes.

## Execution Summary
1. Local runtime timeout wiring fixed:
   - `crates/tau-coding-agent/src/startup_local_runtime.rs` now sets
     `request_timeout_ms` from `cli.request_timeout_ms` instead of defaulting to
     `AgentConfig::default()`.
2. `tau-tui agent` timeout/retry passthrough implemented:
   - Added parser/help support for `--request-timeout-ms` and
     `--agent-request-max-retries`.
   - Launch command builder forwards both flags to `tau-coding-agent`.
3. Unified launcher fast-fail defaults implemented:
   - Added `TAU_UNIFIED_TUI_REQUEST_TIMEOUT_MS` (default `45000`) and
     `TAU_UNIFIED_TUI_AGENT_REQUEST_MAX_RETRIES` (default `0`).
   - Added matching `tui` flags and pass-through to `tau-tui agent`.
4. Documentation updated:
   - `README.md`
   - `docs/guides/operator-deployment-guide.md`

## Verification Notes
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh`
- `bash scripts/run/test-tau-unified.sh`
- `cargo test -p tau-tui -- --nocapture`
- `cargo test -p tau-coding-agent regression_spec_2542_c03_run_local_runtime_prompt_executes_model_call -- --nocapture`
- `cargo test -p tau-coding-agent regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent -- --nocapture`
- `cargo fmt --check`
