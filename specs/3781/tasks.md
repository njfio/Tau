# Tasks 3781: Active Missions Whole-Row Scroll Boundary

- [x] T1: Add spec-derived failing conformance test for whole-row Active Missions scroll boundary.
  - Red: `cargo test -p tau-dashboard-ui functional_spec_3781` failed on the missing whole-row boundary marker.
- [x] T2: Implement HTML/CSS contract for the Active Missions whole-row boundary.
  - Added `data-active-mission-scroll-boundary="whole-row"` and a `388px` scroll cap that exposes three complete mission rows.
- [x] T3: Run targeted and crate-level Rust verification.
  - Green: `cargo test -p tau-dashboard-ui functional_spec_3781`.
  - Regression set: `cargo test -p tau-dashboard-ui functional_spec_377` and `cargo test -p tau-dashboard-ui functional_spec_378`.
  - Crate: `cargo test -p tau-dashboard-ui`.
  - Gateway integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`.
  - Static checks: `cargo fmt --check -p tau-dashboard-ui`, `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`, `git diff --check`.
- [x] T4: Regenerate static harness preview and validate browser geometry.
  - `cargo run --manifest-path /tmp/tau-preview-render/Cargo.toml` regenerated `/tmp/tau-harness-after.html`.
  - Browser Use `iab` was attempted first and reported no active Codex browser pane.
  - Playwright fallback at `1512x1038` confirmed zero console errors, zero horizontal overflow, three whole visible mission rows, zero partial rows, and benchmark/TUI still in viewport.
- [ ] T5: Commit, push, and confirm clean worktree.
