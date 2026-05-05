# Tasks 3787: Proposal Patch Summary Readability

- [x] T1: Add spec-derived failing conformance test for readable Patch Summary.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3787` failed before implementation because Patch Summary did not expose the full-text summary contract.
- [x] T2: Implement scoped Patch Summary full-text layout.
  - Green: added a proposal-detail summary fit contract and allowed only the Patch Summary value to wrap.
- [x] T3: Run targeted and crate-level Rust verification.
  - Verified: `cargo test -p tau-dashboard-ui functional_spec_3787`; `cargo test -p tau-dashboard-ui functional_spec_378`; `cargo test -p tau-dashboard-ui functional_spec_377`; `cargo test -p tau-dashboard-ui`; `cargo fmt --check -p tau-dashboard-ui`; `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`; `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`; `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - Verified: `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml`; Playwright geometry on `file:///tmp/tau-harness-after.html` at 1512x1038; screenshot `/tmp/tau-harness-continue27-after.png`; console errors 0; document horizontal overflow 0; Patch Summary overflow 0; proposal clipped rows 0; audit/review/TUI within viewport.
- [x] T5: Commit, push, and confirm clean worktree.
  - Delivery: this task log is included in the delivery commit for `master`.
