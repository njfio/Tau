# Tasks 3783: Right Review Pane Contained Proof Rows

- [x] T1: Add spec-derived failing conformance test for right review pane contained proof rows.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3783` failed on the missing right-pane contained-proof contract.
- [x] T2: Implement compact density contracts for learning queue, proposal detail, and audit log.
  - Added `data-review-overflow-contract="contained-proof-rows"` plus section-level density markers.
  - Compacted the learning queue into a two-column grid, proposal detail into contained proof rows, and audit rows into a four-row visible proof table.
- [x] T3: Run targeted and crate-level Rust verification.
  - Green: `cargo test -p tau-dashboard-ui functional_spec_3783`.
  - Regression set: `cargo test -p tau-dashboard-ui functional_spec_377` and `cargo test -p tau-dashboard-ui functional_spec_378`.
  - Crate: `cargo test -p tau-dashboard-ui`.
  - Gateway integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`.
  - Static checks: `cargo fmt --check -p tau-dashboard-ui`, `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`, `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml` regenerated `/tmp/tau-harness-after.html`.
  - Browser Use `iab` was attempted first and reported no active Codex browser pane.
  - Playwright fallback at `1512x1038` confirmed zero console errors, no document horizontal overflow, no child overflow in the learning queue/proposal detail/audit log, all four queue items visible, seven proposal rows present, four audit rows visible, and TUI still in viewport.
- [x] T5: Commit, push, and confirm clean worktree.
  - Delivery commit contains this completed task log and was pushed to `origin/master`.
