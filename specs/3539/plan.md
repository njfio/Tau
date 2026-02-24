# Plan: Issue #3539 - M320 TUI-driven interactive agent flow

Status: Implemented

## Approach
1. Add RED-first tests for new `tau-tui` agent-mode parsing and launcher `tui`
   default routing behavior.
2. Implement `tau-tui agent` command mode that:
   - loads/render operator shell context from dashboard artifacts, and
   - delegates to `tau-coding-agent` interactive runtime with inherited TTY IO.
3. Update launcher `cmd_tui`:
   - default path: `tau-tui agent`,
   - explicit read-only path: `--live-shell` routed to `tau-tui shell-live`.
4. Extend launcher contract tests to assert both default and live-shell routes.
5. Update README + operator guide with new mode semantics and usage examples.
6. Run scoped verification commands and complete AC evidence.

## Affected Modules
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `README.md`
- `docs/guides/operator-deployment-guide.md`
- `specs/milestones/m320/index.md`
- `specs/3539/spec.md`
- `specs/3539/plan.md`
- `specs/3539/tasks.md`

## Risks / Mitigations
- Risk: interactive mode could block CI tests if it launches full agent runtime.
  - Mitigation: add deterministic command-construction tests and test runner
    hook path to avoid full runtime in automated checks.
- Risk: launcher option semantics could break existing workflows.
  - Mitigation: preserve live shell via explicit override flag and document
    migration clearly.
- Risk: argument sprawl across launcher and TUI.
  - Mitigation: keep agent-mode arguments minimal and fail closed on unknowns.

## Interfaces / Contracts
- New TUI mode:
  - `cargo run -p tau-tui -- agent [options]`
- Unified launcher:
  - `scripts/run/tau-unified.sh tui` -> interactive agent mode (default)
  - `scripts/run/tau-unified.sh tui --live-shell [watch options]` -> read-only
    dashboard mode
- Deterministic runner hook behavior remains available through:
  - `TAU_UNIFIED_RUNNER`, `TAU_UNIFIED_RUNNER_LOG`, `TAU_UNIFIED_RUNNER_PID`

## ADR
No ADR required (CLI/TUI orchestration and launcher routing behavior within
existing architecture).

## Execution Summary
1. Added RED tests for new contracts:
   - `tau-tui` agent-mode dry-run binary conformance case.
   - launcher contract expectations for default `tui` agent route plus explicit
     `--live-shell` route.
2. Implemented `tau-tui agent`:
   - parser support and help text wiring,
   - deterministic interactive runtime command builder for `tau-coding-agent`,
   - operator shell context render + inherited-IO process handoff,
   - `--dry-run` contract for deterministic testability.
3. Updated `scripts/run/tau-unified.sh`:
   - default `tui` mode now routes to `tau-tui agent`,
   - explicit `--live-shell` preserves read-only watch mode,
   - fail-closed check for watch-only flags without `--live-shell`.
4. Updated README and operator deployment guide with integrated behavior.

## Verification Notes
- RED evidence:
  - `cargo test -p tau-tui --test tui_demo_smoke conformance_tui_agent_mode_dry_run_emits_interactive_launch_contract`
    failed with `unknown argument: agent`.
  - `bash scripts/run/test-tau-unified.sh` failed with missing
    `tau-unified: launching tui (agent)` marker.
- GREEN evidence:
  - `cargo test -p tau-tui` passed.
  - `bash scripts/run/test-tau-unified.sh` passed.
- REGRESSION evidence:
  - `cargo fmt --check` passed.
  - `cargo clippy -p tau-tui -- -D warnings` passed.
  - `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
