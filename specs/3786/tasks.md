# Tasks 3786: Proof Header Metadata No-Wrap

- [x] T1: Add spec-derived failing conformance test for proof header metadata no-wrap.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3786` failed before implementation because `#tau-ops-harness-proof-header` did not expose the no-wrap metadata contract.
- [x] T2: Implement scoped no-wrap metadata layout for the proof header.
  - Green: added proof-header-only metadata fit rules that override general `overflow-wrap: anywhere` behavior for metadata labels and values.
- [x] T3: Run targeted and crate-level Rust verification.
  - Verified: `cargo test -p tau-dashboard-ui functional_spec_3786`; `cargo test -p tau-dashboard-ui functional_spec_378`; `cargo test -p tau-dashboard-ui functional_spec_376`; `cargo test -p tau-dashboard-ui functional_spec_377`; `cargo test -p tau-dashboard-ui`; `cargo fmt --check -p tau-dashboard-ui`; `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`; `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`; `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - Verified: `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml`; Playwright geometry on `file:///tmp/tau-harness-after.html` at 1512x1038; screenshot `/tmp/tau-harness-continue26-after.png`; console errors 0; document horizontal overflow 0; Run ID one-line true; proof window within viewport true.
- [x] T5: Commit, push, and confirm clean worktree.
  - Delivery: this task log is included in the delivery commit for `master`.
