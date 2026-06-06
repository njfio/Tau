# Tasks: Issue #3770 - tau-unified control-plane status snapshot

Status: Implemented

- [x] T1 (SPEC): create reviewed spec/plan/tasks for the status snapshot slice.
- [x] T2 (RED): add status snapshot conformance test and record failing output.
- [x] T3 (GREEN): persist and print `control_plane.*` status markers.
- [x] T4 (GATE): add the focused status snapshot check to the runtime reality gate.
- [x] T5 (DOCS): update README caveat language and verification evidence.
- [x] T6 (VERIFY): run targeted, regression, gate, reality evidence, and static checks.

## Verification Evidence

- RED: `scripts/run/test-tau-unified.sh status_contract` failed before implementation with missing `tau-unified: control_plane.health=running`.
- GREEN: `scripts/run/test-tau-unified.sh status_contract` passed.
- Regression: `scripts/run/test-tau-unified.sh` passed.
- Gate unit: `scripts/dev/test-runtime-reality-gate.sh` passed and asserted `tau_unified_status_control_plane_test`.
- Reality evidence: `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality-control-plane.json --output-md /tmp/tau-runtime-reality-control-plane.md` passed.
- Reality output: `/tmp/tau-runtime-reality-control-plane.json` lists fast checks `tau_product_proof_check`, `tau_unified_launcher_test`, `tau_unified_status_control_plane_test`, `agent_canvas_proof_loop_test`, and `roadmap_status_sync_check` as passed.
- Reality boundary: `unified_runtime_control_plane` remains `partial`; the gate states status visibility is deterministic while polished command-center UX and durable proactive recovery remain incomplete.
- Static: `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh scripts/dev/runtime-reality-gate.sh scripts/dev/test-runtime-reality-gate.sh` passed.
- Static: `scripts/dev/roadmap-status-sync.sh --check --quiet` passed.
- Static: `cargo fmt --check` passed.
- Static: `git diff --check` passed.
