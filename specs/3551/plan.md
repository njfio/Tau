# Plan: Issue #3551 - Unified TUI runtime bootstrap + codex login command compatibility

Status: Implemented

## Approach
1. Extend `tau-unified.sh tui` options with bootstrap control:
   - default bootstrap enabled in normal runs,
   - default disabled when script is under test runner mode,
   - explicit `--bootstrap-runtime` / `--no-bootstrap-runtime`.
2. Reuse existing `cmd_up` flow to start runtime before TUI launch when needed.
3. Add best-effort wait for initial dashboard artifacts and emit clear launcher
   diagnostics.
4. Update OpenAI auth launch spec from `--login` to `login` in provider auth
   runtime helpers and update assertions.
5. Run launcher script tests + provider/auth focused tests.

## Affected Modules
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `crates/tau-provider/src/auth_commands_runtime/shared_runtime_core.rs`
- `crates/tau-provider/src/auth_commands_runtime/openai_backend.rs`
- `crates/tau-provider/src/auth_commands_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/auth_and_provider/provider_launch_cli_and_fallback.rs`

## Risks / Mitigations
- Risk: bootstrap changes alter test runner behavior.
  - Mitigation: disable bootstrap by default when `TAU_UNIFIED_RUNNER` is set;
    add explicit test coverage.
- Risk: changing codex launch args breaks existing tests.
  - Mitigation: update codex launch assertions and run targeted suites.

## Interfaces / Contracts
- New launcher flags:
  - `--bootstrap-runtime`
  - `--no-bootstrap-runtime`
- OpenAI auth launch command contract:
  - from `codex --login`
  - to `codex login`

## ADR
No ADR required (CLI/launcher behavior compatibility adjustment; no architecture
or dependency change).

## Execution Summary
1. Implemented runtime bootstrap helpers in launcher:
   - `wait_for_dashboard_artifacts`
   - `bootstrap_runtime_for_tui`
2. Added TUI bootstrap controls and runtime bootstrap options:
   - `--bootstrap-runtime`
   - `--no-bootstrap-runtime`
   - `--bind`, `--auth-mode`, `--auth-token`, `--auth-password` for bootstrap path.
3. Updated OpenAI auth launch command from `codex --login` to `codex login`:
   - launch spec args
   - user-facing action/reauth guidance strings
4. Updated launcher and auth tests for new behavior.

## Verification Notes
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh`
- `bash scripts/run/test-tau-unified.sh`
- `cargo test -p tau-provider`
- `cargo test -p tau-coding-agent functional_execute_auth_command_login_openai_launch_executes_codex_login_command`
- `cargo fmt --check`
