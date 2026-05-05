# Tasks 3782: Left Dashboard Table Overflow Removal

- [x] T1: Add spec-derived failing conformance test for left dashboard table overflow contracts.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3782` failed on the missing left table no-overflow contract.
- [x] T2: Implement scoped compact no-overflow table layout for Active Missions and M334 benchmark.
  - Added panel-level `data-left-table-fit="compact-no-overflow"` and no-horizontal-overflow markers.
  - Applied scoped table-layout rules outside the viewport media query so the narrow dashboard column fits at desktop widths.
- [x] T3: Run targeted and crate-level Rust verification.
  - Green: `cargo test -p tau-dashboard-ui functional_spec_3782`.
  - Regression set: `cargo test -p tau-dashboard-ui functional_spec_377` and `cargo test -p tau-dashboard-ui functional_spec_378`.
  - Crate: `cargo test -p tau-dashboard-ui`.
  - Gateway integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`.
  - Static checks: `cargo fmt --check -p tau-dashboard-ui`, `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`, `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml` regenerated `/tmp/tau-harness-after.html`.
  - Browser Use `iab` was attempted first and reported no active Codex browser pane.
  - Playwright fallback at `1512x1038` confirmed zero console errors, no document horizontal overflow, Active Missions and benchmark tables inside their wrappers, and mission state/gate chips still visible.
- [ ] T5: Commit, push, and confirm clean worktree.
