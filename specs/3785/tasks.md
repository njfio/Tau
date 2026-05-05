# Tasks 3785: Review Queue Full-Label Readability

- [x] T1: Add spec-derived failing conformance test for readable review queue labels.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3785` failed before implementation because `#tau-ops-harness-learning-queue` did not expose the full-label readability contract.
- [x] T2: Implement compact full-label queue layout for Learning & Proposals.
  - Green: changed the queue to compact full-width rows with preserved learning/proposal IDs and DOM order.
- [x] T3: Run targeted and crate-level Rust verification.
  - Verified: `cargo test -p tau-dashboard-ui functional_spec_3785`; `cargo test -p tau-dashboard-ui functional_spec_378`; `cargo test -p tau-dashboard-ui functional_spec_377`; `cargo test -p tau-dashboard-ui`; `cargo fmt --check -p tau-dashboard-ui`; `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`; `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`; `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - Verified: `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml`; Playwright geometry on `file:///tmp/tau-harness-after.html` at 1512x1038; screenshot `/tmp/tau-harness-continue25-after.png`; console errors 0; document horizontal overflow 0; queue label truncation 0; queue IDs preserved as LR-219, LR-220, PR-044, PR-045; review window and TUI companion within viewport.
- [x] T5: Commit, push, and confirm clean worktree.
  - Delivery: this task log is included in the delivery commit for `master`.
