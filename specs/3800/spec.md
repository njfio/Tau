# Spec 3800: Deterministic Full Validation Target

Status: Implemented
GitHub Issue: #3763
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

The branch release-grade gate can time out while compiling
`tau-agent-core` doctests in the shared default `target/` directory. The same
doctest suite passes in an isolated target directory, so the blocker is stale
or contended validation target state rather than failing doctest behavior.

## Scope

In scope:
- Preserve `tau-agent-core` doctest coverage.
- Make full/release validation avoid the shared default target directory unless
  the caller explicitly supplies `CARGO_TARGET_DIR`.
- Record exact validation commands and outcomes for the branch.

Out of scope:
- Disabling doctests.
- Changing workspace dependencies or TLS providers.
- Reworking production code paths.

## Acceptance Criteria

AC-1: Given the original default-target doctest reproducer is run, when it
times out, then the branch records the timeout as a shared-target validation
failure.

AC-2: Given `scripts/dev/fast-validate.sh --full` runs without an explicit
`CARGO_TARGET_DIR`, when it starts validation, then it uses a deterministic
isolated target directory and disables incremental compilation for that run.

AC-3: Given a caller explicitly sets `CARGO_TARGET_DIR`, when
`scripts/dev/fast-validate.sh --full` runs, then the script preserves the
caller-provided target directory.

AC-4: Given the branch validation path is rerun, when `tau-agent-core` doctests
execute through the isolated target path, then all doctests pass without timing
out.

## Conformance Cases

C-01 maps to AC-1: validation evidence records the exact default-target
`cargo test -p tau-agent-core --doc` timeout and its exit status.

C-02 maps to AC-2 and AC-3: script tests assert full validation exports an
isolated target only when the caller did not already provide one.

C-03 maps to AC-4: focused doctest validation runs with an isolated target and
passes all `tau-agent-core` doctests.

## Success Signals

- `scripts/dev/test-fast-validate.sh`
- `CARGO_TARGET_DIR=/tmp/rust-pi-agent-core-docfix-target /opt/homebrew/bin/timeout 300s env RUST_MIN_STACK=16777216 cargo test -p tau-agent-core --doc --no-fail-fast -- --nocapture`
- `/opt/homebrew/bin/timeout 1800s env RUST_MIN_STACK=16777216 scripts/dev/fast-validate.sh --full`
