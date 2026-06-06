# Plan

Status: Implemented

1. Add a shell regression to `scripts/run/test-tau-unified.sh status_contract` for jobs state propagation and recovery evidence markers.
2. Add a `TAU_UNIFIED_JOBS_STATE_DIR` / `--jobs-state-dir` option to `scripts/run/tau-unified.sh up` and bootstrap `tui`.
3. Extend the control-plane snapshot with `background_jobs.*` artifact paths and a truthful restart-recovery marker.
4. Update README/runtime documentation only if needed after the product surface is stable.

README updates were not needed for this slice because `docs/guides/background-jobs-ops.md` already documents the persisted job layout and restart-recovery reason code now surfaced by `tau-unified status`.

## Affected Modules

- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `scripts/dev/runtime-reality-gate.sh`
- `scripts/dev/test-runtime-reality-gate.sh`
- `specs/3773-tau-unified-job-recovery-status/*`

## Risks

- Status output can become noisy. Mitigation: keep machine-greppable `control_plane.background_jobs.*` markers.
- Jobs state dir could drift from the launched runtime command. Mitigation: persist it in the control-plane snapshot at `up` time and print from that snapshot.
- This could overstate autonomy. Mitigation: explicitly retain the existing autonomy boundary marker and use narrow recovery wording.
