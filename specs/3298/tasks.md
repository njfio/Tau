# Tasks: Issue #3298 - stabilize tau-coding-agent baseline tests to unblock mutation gate

- [x] T1 (RED): reproduce listed failing tests and capture red evidence.
- [x] T2 (GREEN): update credential-store test assertions for keyed v2 prefix contract.
- [x] T3 (GREEN): update startup prompt composition test to assert skill summary-mode markers.
- [x] T4 (GREEN): update gateway auth validation error assertions for token/password id wording.
- [x] T5 (GREEN): add required telemetry schema metadata in audit summary fixture rows.
- [x] T6 (GREEN): update tool policy schema version assertion to current contract value.
- [x] T7 (VERIFY): run targeted test set covering C-01..C-05.
- [x] T8 (VERIFY): run full `tau-coding-agent` baseline suite + fmt + clippy.
- [x] T9 (GREEN): add APO regression tests C-07..C-09 in `live_rl_runtime.rs`.
- [x] T10 (VERIFY): run focused mutation checks for `run_live_apo_update` branches (`803`, `808`, `904`) and archive outcomes.
