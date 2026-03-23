# Tasks: Issue #3618 - Interactive skill auto-selection

Status: In Progress

## Ordered Tasks
1. [ ] T1 (RED): add failing `tau-skills` tests for merged catalog loading and prompt-based web-game auto-selection.
2. [ ] T2 (RED): add failing `tau-coding-agent` integration coverage proving an interactive Phaser/web-game prompt picks the bundled skill in the real prompt path.
3. [ ] T3 (RED): add failing `tau-tui` tests proving active skills are visible for matching prompts and absent for non-matching prompts.
4. [ ] T4 (GREEN): implement shared catalog merge and auto-selection helpers in `tau-skills`.
5. [ ] T5 (GREEN): wire per-turn runtime prompt recomposition in `tau-coding-agent` using the shared selector.
6. [ ] T6 (GREEN): wire active-skill visibility into the graphical TUI using the shared selector.
7. [ ] T7 (GREEN): add the bundled repo skill content under `skills/web-game-phaser.md`.
8. [ ] T8 (VERIFY): run focused crate tests and a root-path launcher smoke.

## Test Tier Intent
| Tier | Planned |
| --- | --- |
| Unit | skill merge/selection heuristics, TUI visibility helpers |
| Functional | runtime prompt recomposition for interactive turns |
| Integration | real coding-agent prompt path and interactive TUI submission path |
| Regression | preserve explicit skill selections and no-match fallback |
| Snapshot | N/A |
| Property | N/A |
| Contract/DbC | N/A |
| Fuzz | N/A |
| Mutation | N/A |
| Performance | N/A |

## TDD Evidence
### RED
- Pending targeted failing tests for `tau-skills`, `tau-coding-agent`, and `tau-tui`.

### GREEN
- Pending implementation.

### VERIFY
- Pending focused validation commands.
