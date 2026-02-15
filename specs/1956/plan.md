# Issue 1956 Plan

Status: Reviewed

## Approach

1. Extend `RunnerConfig` with:
   - `transient_error_backoff_initial: Duration`
   - `transient_error_backoff_max: Duration`
2. Add `RunnerConfig::validate()` enforcing positive durations and max>=initial.
3. Add helper `compute_poll_retry_delay(failure_count, initial, max)` for bounded
   exponential backoff.
4. Update `TrainingRunner::new` to validate config and panic with deterministic
   message on invalid values (constructor remains non-fallible API).
5. Update poll branch in `run`:
   - on `process_once` success -> reset `poll_failure_count`
   - on error -> increment count, sleep computed delay, continue loop.
6. Add tests for C-01..C-04 including integration recovery path.

## Affected Areas

- `crates/tau-training-runner/src/lib.rs`
- `specs/1956/spec.md`
- `specs/1956/plan.md`
- `specs/1956/tasks.md`

## Risks And Mitigations

- Risk: backoff could hide persistent failures.
  - Mitigation: cap delay and keep loop active; retain existing tracing for visibility.
- Risk: constructor panic policy could surprise callers.
  - Mitigation: deterministic panic text and explicit validation method for pre-check.

## ADR

No dependency/protocol changes; ADR not required.
