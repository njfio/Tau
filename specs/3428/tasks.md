# Tasks: Issue #3428 - Dashboard/TUI operator-flow convergence

- [x] T1 (RED, Unit/Integration/Conformance): add failing tests for TUI parity markers and cross-surface dashboard/TUI parity assertions.
- [x] T2 (GREEN, Implementation): extend TUI operator-shell model/renderer to include auth + control-plane parity fields.
- [x] T3 (REGRESSION, Functional): run targeted dashboard/TUI test selectors and quality gates.
- [x] T4 (VERIFY, Conformance): capture AC -> test evidence and mark spec status implemented.

## AC -> Conformance -> Tests
| AC | Conformance Case(s) | Test(s) |
|---|---|---|
| AC-1 | C-01 | `spec_c28_regression_operator_shell_auth_panel_requires_auth_mode_and_required_markers` |
| AC-2 | C-02 | `spec_c28_regression_operator_shell_status_and_alert_panels_require_control_plane_markers` |
| AC-3 | C-03 | `spec_c28_regression_dashboard_and_tui_require_shared_operator_flow_markers` |
| AC-4 | C-04 | `specs/3428/{spec.md,plan.md,tasks.md}` |

## TDD Evidence
### RED
- `cargo test -p tau-tui spec_c28 -- --test-threads=1`
- Result: failed on missing `auth.mode`, `auth.required`, `health.reason`, `queue.depth`, `failure.streak`, and `primary_alert.code` markers in TUI output.
- `cargo test -p tau-dashboard-ui spec_c28 -- --test-threads=1`
- Result: failed on missing shared marker `auth.mode     : password-session` in TUI output while dashboard markers were present.

### GREEN
- `cargo test -p tau-tui spec_c28 -- --test-threads=1` -> `2 passed; 0 failed`.
- `cargo test -p tau-dashboard-ui spec_c28 -- --test-threads=1` -> `1 passed; 0 failed`.

### REGRESSION / VERIFY
- `cargo test -p tau-tui operator_shell_renderer -- --test-threads=1` -> `2 passed`.
- `cargo test -p tau-dashboard-ui spec_2786_c03 -- --test-threads=1` -> `3 passed`.
- `cargo fmt --all -- --check` -> pass.
- `cargo clippy -p tau-tui -p tau-dashboard-ui --tests -- -D warnings` -> pass.

## Tier Mapping
- Unit: TUI renderer field/marker coverage.
- Property: N/A (no randomized invariant parser/algorithm change).
- Contract/DbC: N/A (no DbC annotation changes).
- Snapshot: N/A (no snapshot harness introduced).
- Functional: dashboard and TUI rendering behavior checks.
- Conformance: AC/C-case mapping selectors for #3428.
- Integration: cross-surface parity assertion between dashboard HTML and TUI rendering.
- Fuzz: N/A (no new untrusted input surface).
- Mutation: N/A (UI contract hardening slice, non-critical algorithm path).
- Regression: targeted existing dashboard/TUI selectors.
- Performance: N/A (no performance-sensitive path changes).
