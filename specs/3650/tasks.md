# Tasks: Issue #3650 - Fail just tui closed when runtime bootstrap is unavailable or unready

- [x] T1 (RED): add focused shell regressions for TUI bootstrap readiness failure, bind/conflict-like failed startup, and healthy readiness preservation.
  - Verify with: `bash scripts/run/test-tau-unified.sh tui_bootstrap_readiness`

- [x] T2 (GREEN): update `scripts/run/tau-unified.sh` so TUI bootstrap fails closed before launching TUI when runtime readiness cannot be proven.
  - Preserve `--no-bootstrap-runtime` for deliberate manual attachment.
  - Preserve existing timeout/retry/auth/model argument wiring.

- [x] T3 (VERIFY): run scoped launcher and Rust gates.
  - `bash scripts/run/test-tau-unified.sh tui_bootstrap_readiness`
  - `bash scripts/run/test-tau-unified.sh`
  - `bash -n scripts/run/tau-unified.sh`
  - `cargo fmt --check`
  - `git diff --quiet -- Cargo.toml Cargo.lock`

- [x] T4 (CLOSE): path-limit commit/push the #3650 artifacts and launcher changes, then close issue #3650 with verification evidence.
