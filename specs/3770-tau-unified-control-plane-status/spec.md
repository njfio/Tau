# Spec: Issue #3770 - tau-unified control-plane status snapshot

Status: Implemented

## Problem Statement

`tau-unified` is the obvious runtime launcher, but `status` currently reports only process artifact paths. That makes the runtime control-plane claim weaker than the actual integrated substrate: operators cannot see, from one stable command, which health, log, session, memory, job/routine, and deploy/process surfaces are available and which autonomy claims remain out of scope.

## Scope

In scope:

- Add stable `control_plane.*` status markers to `scripts/run/tau-unified.sh status`.
- Persist a status snapshot during `up` so `status` does not infer operator state from prose or command scraping.
- Cover health, logs, command/fingerprint artifacts, web/operator endpoints, sessions, memory, jobs, routines, and deploy/process state.
- Keep unsupported claims explicit: the snapshot may expose job/routine surfaces, but it must not claim durable forever autonomy, replay, stuck-job recovery, or crash-resume as complete.
- Wire the runtime reality gate to validate the status snapshot contract.
- Update README evidence so the caveat is precise.

Out of scope:

- Building a new TUI/dashboard command-center UI.
- Adding new durable job, replay, crash-resume, or stuck-job recovery semantics.
- Running live provider credentials or headed-browser pixel proof.
- Refactoring large dashboard/gateway hotspot modules.

## Acceptance Criteria

### AC-1 Status emits a deterministic control-plane snapshot

Given `tau-unified up` has started a runtime, when `tau-unified status` runs, then it prints stable `tau-unified: control_plane.*` markers for process health, log path, command path, fingerprint path, runtime state dir, profile, and core web/operator endpoints.

### AC-2 Status surfaces runtime work-state contracts honestly

Given `tau-unified status` runs, when sessions, memory, jobs/routines, and deploy/process surfaces are inspected, then the output names the relevant gateway/dashboard endpoint or state path and separately states the autonomy boundary that is not claimed complete.

### AC-3 Reality gate validates the snapshot

Given the runtime reality gate runs fast checks, when it evaluates unified runtime evidence, then it runs a focused status snapshot regression and reports that evidence separately from the broader launcher regression.

### AC-4 Documentation keeps the caveats executable

Given README describes Tau maturity, when it mentions the unified runtime experience, then it links the status snapshot and keeps production RL, live provider validation, headed-browser pixel proof, forever autonomy, and maintainability-solved claims out of the completed bucket.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | runner-backed `tau-unified up` | `status` is invoked | output contains `control_plane.health=running`, artifact paths, profile, runtime dir, and endpoint markers |
| C-02 | AC-2 | Conformance | runner-backed runtime with explicit state dirs | `status` is invoked | output contains sessions, memory, jobs, routines, deploy endpoint/state markers, and an autonomy boundary marker |
| C-03 | AC-3 | Functional | fake fast-check scripts | runtime reality gate runs | JSON contains `tau_unified_status_control_plane_test` as a passed fast check |
| C-04 | AC-4 | Documentation | README maturity section | docs are inspected | unified status is described as visibility proof without upgrading unsupported claims |

## Success Metrics / Observable Signals

- `scripts/run/test-tau-unified.sh status_contract` fails before implementation and passes after.
- `scripts/run/test-tau-unified.sh` passes.
- `scripts/dev/test-runtime-reality-gate.sh` passes and asserts the new fast-check ID.
- `scripts/dev/runtime-reality-gate.sh --output-json ... --output-md ...` emits the new check and keeps unsupported claims guarded.
- No new dependencies are introduced.
