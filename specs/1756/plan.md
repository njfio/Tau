# Issue 1756 Plan

Status: Reviewed

## Approach

1. Implement alias normalization in `tau-cli`:
   - map legacy `--train-*` and `--training-proxy-*` flags to canonical
     `--prompt-optimization-*` equivalents
   - return warning strings for each legacy alias encountered
2. Update startup parse flow in `tau-coding-agent` to:
   - normalize daemon subcommand aliases
   - normalize legacy training aliases
   - emit warning lines to stderr before parse
3. Update test helpers to use the same normalization pipeline.
4. Add/update tests:
   - compatibility parsing tests for legacy aliases
   - warning text snapshot-style assertions
   - regression test ensuring unknown flags still fail

## Affected Areas

- `crates/tau-cli/src/` (new legacy alias normalizer module + exports)
- `crates/tau-coding-agent/src/main.rs` (startup parse path)
- `crates/tau-coding-agent/src/tests.rs` (test parse helpers)
- `crates/tau-coding-agent/src/tests/cli_validation.rs`
- `crates/tau-coding-agent/src/tests/misc.rs`

## Contracts

Legacy alias mapping contract:

- `--train-config` -> `--prompt-optimization-config`
- `--train-store-sqlite` -> `--prompt-optimization-store-sqlite`
- `--train-json` -> `--prompt-optimization-json`
- `--training-proxy-server` -> `--prompt-optimization-proxy-server`
- `--training-proxy-bind` -> `--prompt-optimization-proxy-bind`
- `--training-proxy-upstream-url` -> `--prompt-optimization-proxy-upstream-url`
- `--training-proxy-state-dir` -> `--prompt-optimization-proxy-state-dir`
- `--training-proxy-timeout-ms` -> `--prompt-optimization-proxy-timeout-ms`

Warning text contract:

- deterministic format with legacy flag and canonical replacement

## Risks And Mitigations

- Risk: accidental remap of unsupported flags
  - Mitigation: explicit static mapping list and regression tests.
- Risk: warning text churn
  - Mitigation: lock expected warning strings in tests.

## ADR

No dependency/protocol changes. ADR not required.
