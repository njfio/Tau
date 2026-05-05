# Tasks 3784: Center Proof Evidence Containment

- [x] T1: Add spec-derived failing conformance test for center proof evidence containment.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3784` failed before implementation because `#tau-ops-harness-tool-evidence` did not expose the compact containment marker.
- [x] T2: Implement compact no-overflow table rules for Tool Execution Evidence and all-visible chip rules for Acceptance Criteria.
  - Green: added scoped `compact-no-overflow` table rules and `all-criteria-visible` acceptance chip rules.
- [x] T3: Run targeted and crate-level Rust verification.
  - Verified: `cargo test -p tau-dashboard-ui functional_spec_3784`; `cargo test -p tau-dashboard-ui functional_spec_377`; `cargo test -p tau-dashboard-ui functional_spec_378`; `cargo test -p tau-dashboard-ui`; `cargo fmt --check -p tau-dashboard-ui`; `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`; `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`; `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - Verified: `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml`; Playwright geometry on `file:///tmp/tau-harness-after.html` at 1512x1038; screenshot `/tmp/tau-harness-continue24-after.png`; console errors 0; document horizontal overflow 0; tool table overflow -1px; acceptance child overflow 0; VG-05 visible.
- [x] T5: Commit, push, and confirm clean worktree.
  - Delivery: this task log is included in the delivery commit for `master`.
