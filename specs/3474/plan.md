# Plan: Issue #3474 - M304 tau-tui shell-live watch mode

Status: Implemented

## Approach
1. Add RED parser/output tests in `crates/tau-tui/src/main.rs` for:
   - accepted `shell-live --watch --iterations --interval-ms` args,
   - rejected `--iterations 0`,
   - deterministic watch marker formatting.
2. Extend `LiveShellArgs` and parser logic to support watch controls with
   validation.
3. Implement watch-mode execution in `run_shell_live`:
   - run N render cycles,
   - print deterministic cycle marker per iteration,
   - preserve existing one-shot behavior when watch is disabled.
4. Update README live TUI command examples with watch-mode invocation.
5. Run scoped tau-tui tests + fmt and update spec/tasks evidence.

## Affected Modules
- `crates/tau-tui/src/main.rs`
- `README.md`
- `specs/milestones/m304/index.md`
- `specs/3474/spec.md`
- `specs/3474/plan.md`
- `specs/3474/tasks.md`

## Risks / Mitigations
- Risk: watch loop behavior introduces non-deterministic output.
  - Mitigation: deterministic marker helper and parser-contract tests.
- Risk: watch flags break existing one-shot shell-live behavior.
  - Mitigation: preserve default args (`watch=false`) and assert unchanged path.
- Risk: long watch loops can stall local scripts.
  - Mitigation: explicit `--iterations` control and default bounded cycle count.

## Interfaces / Contracts
- CLI contract (`shell-live`):
  - `--watch` enables multi-cycle mode.
  - `--iterations <usize>=1+` controls cycle count.
  - `--interval-ms <u64>` controls sleep between cycles.
- Output marker contract:
  - deterministic `watch.cycle=<i>/<n> watch.interval_ms=<ms>` line per cycle.

## ADR
No ADR required (localized CLI workflow enhancement).

## Execution Summary
1. Added RED tests in `crates/tau-tui/src/main.rs` for:
   - watch-mode arg acceptance (`--watch --iterations --interval-ms`),
   - zero-iteration fail-closed validation,
   - help text watch-flag markers.
2. Extended `LiveShellArgs` and parser logic with watch controls and validation:
   - `--watch`,
   - `--iterations >= 1`,
   - `--interval-ms`,
   - fail closed when `--iterations/--interval-ms` are provided without `--watch`.
3. Implemented watch-mode render loop in `run_shell_live` with deterministic
   marker emission:
   - `watch.cycle=<i>/<n> watch.interval_ms=<ms> watch.diff_ops=<k>`.
4. Preserved one-shot `shell-live` behavior when watch mode is not enabled.
5. Updated README live TUI command examples to include watch-mode execution.

## Verification Notes
- RED evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui 3474 -- --nocapture`
  - Failed before implementation with:
    - unknown `--watch`,
    - missing watch flags in help text,
    - missing `--iterations >= 1` validation.
- GREEN evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui 3474 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo run -p tau-tui -- shell-live --state-dir .tau/dashboard --profile local-dev --watch --iterations 1 --interval-ms 0 --no-color` emitted deterministic watch markers.
- Regression/gate evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-tui --tests -- -D warnings` passed.
  - `cargo fmt --check` passed.
