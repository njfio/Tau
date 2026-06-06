# Issue #3773: tau-unified Background Job Recovery Status

Status: Implemented

## Problem

`tau-unified status` names the jobs endpoint, but it does not show the durable background-job state directory or recovery evidence files. Operators cannot tell from the default control-plane command where job manifests, health counters, and restart-recovery events are persisted.

## Scope

In:
- Add a `tau-unified` jobs state configuration path that is passed to `tau-coding-agent`.
- Persist and print stable `control_plane.background_jobs.*` status markers.
- State the narrow recovery truth: persisted running job manifests are requeued after runtime restart.

Out:
- No new background-job scheduler semantics.
- No proactive stuck-job recovery claim.
- No full crash-resume or forever-autonomous runtime claim.

## Acceptance Criteria

### AC-1: Jobs State Is Configurable

Given `tau-unified up` receives a jobs state directory, when it builds the runtime command, then `tau-coding-agent` is launched with `--jobs-state-dir` pointing at that directory.

### AC-2: Status Shows Durable Job Evidence

Given `tau-unified status` runs for a started runtime, when operators inspect the output, then it includes stable markers for the background-job state dir, manifest dir, event log, health snapshot, and operations guide.

### AC-3: Status States Recovery Boundary Truthfully

Given background-job recovery is represented in status output, when operators read the markers, then status says running manifests are requeued after restart and does not claim full crash-resume or proactive stuck-job recovery.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | runner-backed `tau-unified up` with `--jobs-state-dir /tmp/jobs` | command file is inspected | command contains `--jobs-state-dir /tmp/jobs` |
| C-02 | AC-2 | Conformance | runner-backed runtime with explicit jobs state dir | `status` is invoked | output contains `control_plane.background_jobs.state_dir`, `manifest_dir`, `events_file`, `health_file`, and `ops_guide` |
| C-03 | AC-3 | Functional | same status output | recovery markers are inspected | output contains `restart_recovery=running_manifests_requeued_after_restart` and keeps the autonomy boundary marker |

## Success Signals

- `scripts/run/test-tau-unified.sh status_contract` fails before implementation and passes after implementation.
- `scripts/run/test-tau-unified.sh` passes.
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passes.
