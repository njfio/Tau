# Plan: Issue #3770 - tau-unified control-plane status snapshot

## Approach

1. Add a focused shell regression selector, `scripts/run/test-tau-unified.sh status_contract`, that starts the runner-backed runtime and asserts the missing `control_plane.*` status markers.
2. Make `tau-unified up` write a small control-plane metadata file under the runtime dir. This avoids parsing the last command and keeps `status` deterministic.
3. Make `tau-unified status` print both the existing process artifact lines and the new status snapshot lines.
4. Add the focused status-contract selector to the runtime reality gate fast checks.
5. Update README and this spec with the precise boundary: visibility improves, durable proactive autonomy and production RL/live proof do not become complete claims.

## Affected Modules

- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `scripts/dev/runtime-reality-gate.sh`
- `scripts/dev/test-runtime-reality-gate.sh`
- `README.md`
- `specs/3770-tau-unified-control-plane-status/`

## Risks / Mitigations

- Risk: status output grows noisy. Mitigation: use stable prefixed `control_plane.*` markers that are easy to scan and grep.
- Risk: metadata file drifts from the running runtime. Mitigation: write the snapshot immediately before launch from the same parsed arguments used to build the command.
- Risk: endpoints are visible but not live-validated. Mitigation: name them as surfaces and keep live/provider/deep autonomy proof separate in the reality gate.
- Risk: older runtime dirs lack the snapshot. Mitigation: `status` should report `control_plane.snapshot=missing` instead of failing process status.

## Verification

- RED: `scripts/run/test-tau-unified.sh status_contract`
- GREEN: `scripts/run/test-tau-unified.sh status_contract`
- Regression: `scripts/run/test-tau-unified.sh`
- Gate unit: `scripts/dev/test-runtime-reality-gate.sh`
- Reality evidence: `scripts/dev/runtime-reality-gate.sh --output-json /tmp/tau-runtime-reality-control-plane.json --output-md /tmp/tau-runtime-reality-control-plane.md`
- Static: `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh scripts/dev/runtime-reality-gate.sh scripts/dev/test-runtime-reality-gate.sh`
- Static: `git diff --check`
