# Plan: Issue #3535 - M319 unified one-command runtime entrypoint

Status: Implemented

## Approach
1. Add RED-first contract test for missing launcher script.
2. Implement `scripts/run/tau-unified.sh` lifecycle commands with deterministic
   artifact/pid handling.
3. Add fail-closed argument validation and status/down edge-path handling.
4. Update README and operator deployment guide with one-command usage.
5. Run RED->GREEN->REGRESSION checks and update spec evidence.

## Affected Modules
- `scripts/run/tau-unified.sh` (new)
- `scripts/run/test-tau-unified.sh` (new)
- `README.md`
- `docs/guides/operator-deployment-guide.md`
- `specs/milestones/m319/index.md`
- `specs/3535/spec.md`
- `specs/3535/plan.md`
- `specs/3535/tasks.md`

## Risks / Mitigations
- Risk: launcher lifecycle races around stale pid files.
  - Mitigation: explicit pid liveness checks and stale-pid cleanup.
- Risk: script behavior diverges from existing gateway launch contracts.
  - Mitigation: reuse documented gateway flags and default values.
- Risk: brittle tests if launcher must start real runtime.
  - Mitigation: add runner-hook env for deterministic contract tests.

## Interfaces / Contracts
- Launcher command:
  - `scripts/run/tau-unified.sh <up|status|down|tui>`
- Runtime artifact directory:
  - `${TAU_UNIFIED_RUNTIME_DIR:-.tau/unified}`
- Runtime files:
  - `tau-unified.pid`, `tau-unified.log`, `tau-unified.last-cmd`

## ADR
No ADR required (script/docs operational orchestration scope).

## Execution Summary
1. Added RED-first contract test `scripts/run/test-tau-unified.sh` and captured
   expected pre-implementation failure while launcher script was absent.
2. Implemented `scripts/run/tau-unified.sh` lifecycle commands:
   `up`, `status`, `down`, and `tui`.
3. Added deterministic runtime artifacts under `.tau/unified/` and fail-closed
   lifecycle/argument checks.
4. Updated README and operator deployment guide with one-command launcher flow.

## Verification Notes
- RED evidence:
  - `bash scripts/run/test-tau-unified.sh`
  - Result before implementation:
    - `error: launcher script missing or not executable: .../scripts/run/tau-unified.sh`
- GREEN evidence:
  - `bash scripts/run/test-tau-unified.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
  - `cargo fmt --check` passed.
