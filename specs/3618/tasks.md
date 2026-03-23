# Tasks: Issue #3618 - Interactive skill auto-selection

Status: Completed

## Ordered Tasks
1. [x] T1 (RED): add failing `tau-skills` tests for merged catalog loading and prompt-based web-game auto-selection.
2. [x] T2 (RED): add failing `tau-coding-agent` integration coverage proving an interactive Phaser/web-game prompt picks the bundled skill in the real prompt path.
3. [x] T3 (RED): add failing gateway/runtime coverage proving the graphical TUI transport path applies the same bundled skill guidance.
4. [x] T4 (RED): add failing `tau-tui` tests proving active skills are visible for matching prompts and absent for non-matching prompts.
5. [x] T5 (GREEN): implement shared catalog merge and auto-selection helpers in `tau-skills`.
6. [x] T6 (GREEN): wire per-turn runtime prompt recomposition in `tau-coding-agent` using the shared selector.
7. [x] T7 (GREEN): wire gateway openresponses prompt augmentation using the same conservative selection heuristics without introducing a crate cycle.
8. [x] T8 (GREEN): wire active-skill visibility into the graphical TUI using the shared selector.
9. [x] T9 (GREEN): add the bundled repo skill content under `skills/web-game-phaser.md`.
10. [x] T10 (VERIFY): run focused crate tests and a root-path launcher smoke.

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | skill merge/selection heuristics, TUI visibility helpers |
| Functional | runtime prompt recomposition for interactive turns and gateway execution |
| Integration | real coding-agent prompt path, gateway path, and interactive TUI visibility |
| Regression | preserve explicit skill selections and no-match fallback |
| Snapshot | N/A |
| Property | N/A |
| Contract/DbC | N/A |
| Fuzz | N/A |
| Mutation | N/A |
| Performance | N/A |

## TDD Evidence
### RED
- `cargo test -p tau-skills --lib red_spec_3618 -- --nocapture`
- `cargo test -p tau-gateway red_spec_3618_openresponses_request_uses_bundled_web_game_skill_guidance -- --nocapture`
- `cargo test -p tau-coding-agent red_spec_3618_apply_interactive_turn_skill_selection_uses_bundled_repo_skill_for_phaser_prompt -- --nocapture`
- `cargo test -p tau-tui red_spec_3618 -- --nocapture`

### GREEN
- Shared catalog merge and prompt-based auto-selection live in `tau-skills`.
- Coding-agent interactive turns recompose the system prompt per turn with auto-selected skills.
- Gateway openresponses augments the system prompt for the graphical TUI transport path.
- Graphical TUI surfaces the selected skill names in the status bar.
- Bundled Phaser skill content ships in `skills/web-game-phaser.md`.

### VERIFY
- `CARGO_TARGET_DIR=target/3618-b CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-C debuginfo=0' cargo fmt --all --check`
- `CARGO_TARGET_DIR=target/3618-b CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-C debuginfo=0' cargo test -p tau-skills --lib red_spec_3618 -- --nocapture`
- `CARGO_TARGET_DIR=target/3618-b CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-C debuginfo=0' cargo test -p tau-gateway red_spec_3618_openresponses_request_uses_bundled_web_game_skill_guidance -- --nocapture`
- `CARGO_TARGET_DIR=target/3618-b CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-C debuginfo=0' cargo test -p tau-coding-agent red_spec_3618_apply_interactive_turn_skill_selection_uses_bundled_repo_skill_for_phaser_prompt -- --nocapture`
- `CARGO_TARGET_DIR=target/3618-b CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-C debuginfo=0' cargo test -p tau-tui red_spec_3618 -- --nocapture`
- `just stack-down || true && just stack-up-fresh && just stack-down`
