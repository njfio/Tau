# Tasks

Status: Implemented

- [x] T1 (RED): Add failing status contract assertions for jobs state propagation and recovery markers.
- [x] T2 (GREEN): Pass `--jobs-state-dir` through `tau-unified up` and TUI bootstrap.
- [x] T3 (GREEN): Persist and print `control_plane.background_jobs.*` status markers.
- [x] T4 (VERIFY): Run focused shell tests and syntax checks.

## TDD Evidence

- RED: `scripts/run/test-tau-unified.sh status_contract` failed before implementation with exit 2 because `tau-unified up` rejected `--jobs-state-dir`; direct reproducer printed `unknown up option: --jobs-state-dir`.
- GREEN: `scripts/run/test-tau-unified.sh status_contract` passed.
- Regression: `scripts/run/test-tau-unified.sh` passed.
- Gate unit: `scripts/dev/test-runtime-reality-gate.sh` passed.
- Reality evidence: `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality-3773.json --output-md /tmp/tau-runtime-reality-3773.md` passed.
- Reality output: `/tmp/tau-runtime-reality-3773.json` lists `tau_unified_status_control_plane_test` as passed and keeps `unified_runtime_control_plane` classified as `partial`.
- Static: `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh scripts/dev/runtime-reality-gate.sh scripts/dev/test-runtime-reality-gate.sh` passed.
- Diff hygiene: `git diff --check` passed.

## Test Tiers

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A | | Shell launcher/status slice; no Rust unit surface changed. |
| Property | N/A | | No parser/serializer invariant changed. |
| Contract/DbC | N/A | | No contract-annotated API changed. |
| Snapshot | N/A | | No snapshot framework used. |
| Functional | PASS | `scripts/run/test-tau-unified.sh`; `scripts/dev/test-runtime-reality-gate.sh` | |
| Conformance | PASS | `scripts/run/test-tau-unified.sh status_contract`; `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality-3773.json --output-md /tmp/tau-runtime-reality-3773.md` | |
| Integration | PASS | Runner-backed `tau-unified up/status/down/tui` shell lifecycle in `scripts/run/test-tau-unified.sh` | |
| Fuzz | N/A | | No untrusted input/parser path changed. |
| Mutation | N/A | | Shell/status visibility slice; no critical Rust algorithm changed. |
| Regression | PASS | `scripts/run/test-tau-unified.sh status_contract` | |
| Performance | N/A | | No hot path or performance-sensitive code changed. |
