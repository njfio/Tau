# Final Validation Package Dry Run

This file records one rehearsal fill of the package template using live-run evidence.

## Final Validation Package: rehearsal-2026-02-14

- Decision owner: Codex automation
- Reviewer(s): pending human reviewer
- Decision timestamp (UTC): 2026-02-14T00:00:00Z
- Decision: GO (rehearsal only)

### 1) Execution transcripts

- Command: `./scripts/demo/live-run-unified.sh --skip-build --timeout-seconds 180 --keep-going`
  - Evidence: `.tau/live-run-unified/report.json`
  - Result summary: `total=5 passed=5 failed=0 duration_ms=8275`

### 2) Logs/traces/screenshots/audio

- Voice: `.tau/live-run-unified/surfaces/voice/stdout.log`
- Browser: `.tau/live-run-unified/surfaces/browser/stdout.log`
- Dashboard: `.tau/live-run-unified/surfaces/dashboard/stdout.log`
- Custom command: `.tau/live-run-unified/surfaces/custom-command/stdout.log`
- Memory: `.tau/live-run-unified/surfaces/memory/stdout.log`

### 3) Artifact manifest

- Manifest: `.tau/live-run-unified/manifest.json`
- Report: `.tau/live-run-unified/report.json`

### 4) Rollback readiness + go/no-go summary

- Rollback trigger matrix review evidence:
  `docs/guides/release-channel-ops.md#rollback-trigger-matrix`
- Current trigger status: none active
- Final decision rationale: rehearsal run passed all five surfaces and produced complete artifact set.
