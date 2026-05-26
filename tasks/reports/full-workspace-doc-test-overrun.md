# Full Workspace Doc-Test Overrun Classification And Resolution

Date: 2026-05-23

## Classification

The remaining full-workspace validation blocker is a reproducible compile-time
overrun in the doc-test path, not an Agent Canvas or ops chat functional
failure.

The focused reproducer timed out while compiling `tau-agent-core` doc tests:

```bash
/opt/homebrew/bin/timeout 300s env RUST_MIN_STACK=16777216 \
  cargo test -p tau-agent-core --doc --no-fail-fast -- --nocapture
```

Result: exit `124` after 300 seconds, still in the compile phase.

A default-target debug binary build also hit the same overrun class:

```bash
/opt/homebrew/bin/timeout 300s env CARGO_INCREMENTAL=0 RUST_MIN_STACK=16777216 \
  cargo build -p tau-coding-agent --bin tau-coding-agent
```

Result: exit `124` after 300 seconds, still compiling downstream crates.

## Mitigation Used For This Slice

Touched-crate validation was run in an isolated target dir to avoid stale
default-target compiler pressure:

```bash
env CARGO_INCREMENTAL=0 RUST_MIN_STACK=16777216 CARGO_TARGET_DIR=target-ui-check \
  cargo check -p tau-dashboard-ui -vv
```

Result: passed.

The live UI proof server was built from a temporary target dir outside the repo
and launched with the same stack setting:

```bash
env CARGO_INCREMENTAL=0 RUST_MIN_STACK=16777216 CARGO_TARGET_DIR=/tmp/rust-pi-canvas-v2-target \
  cargo build -p tau-coding-agent --bin tau-coding-agent
```

Result: passed.

## 2026-05-26 Remediation

The original default-target reproducer was rerun and still timed out:

```bash
/opt/homebrew/bin/timeout 300s env RUST_MIN_STACK=16777216 \
  cargo test -p tau-agent-core --doc --no-fail-fast -- --nocapture
```

Result: exit `124` after 300 seconds in the shared default `target/` path.

The same doctest gate passes in an isolated target directory:

```bash
/opt/homebrew/bin/timeout 300s env \
  CARGO_TARGET_DIR=/tmp/rust-pi-agent-core-docfix-target \
  RUST_MIN_STACK=16777216 \
  cargo test -p tau-agent-core --doc --no-fail-fast -- --nocapture
```

Result: passed. Compile finished in `40.18s`; doctests finished in `3.34s`;
`9 passed; 0 failed`.

`scripts/dev/fast-validate.sh --full` now isolates full-validation target state
when the caller has not supplied `CARGO_TARGET_DIR`, and preserves caller
overrides when they are present. Script coverage:

```bash
bash -n scripts/dev/fast-validate.sh scripts/dev/test-fast-validate.sh
scripts/dev/test-fast-validate.sh
```

Result: passed.

Full release-grade validation was rerun through the isolated target path:

```bash
/opt/homebrew/bin/timeout 1800s env RUST_MIN_STACK=16777216 \
  scripts/dev/fast-validate.sh --full
```

Target selected by the script:

```text
/tmp/rust_pi-codex-issue-3758-deploy-stop-process-lifecycle-fast-validate-full-target
```

Result: passed.

- `cargo fmt --all -- --check`: passed in `3s`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed in `163s`.
- `cargo test --workspace`: passed in `404s`, including
  `tau-agent-core` doctests: `9 passed; 0 failed`.

## Release-Gate Impact

The branch can now claim a clean full-workspace release-grade gate when run via
the isolated full-validation path. The shared default `target/` directory still
exhibits a local stale/contended compiler-state timeout and should not be used
as release evidence for this branch.
