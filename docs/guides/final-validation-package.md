# Final Validation Package Guide

Run commands from repository root.

Use this guide to assemble the go/no-go package for rollout-sensitive changes.

Related guides:

- [Release Sign-Off Checklist](release-signoff-checklist.md)
- [Release Channel Ops](release-channel-ops.md)
- [Unified Live-Run Harness Guide](live-run-unified-ops.md)
- [Final Validation Package Dry Run](final-validation-package-dry-run.md)

## Required package contents

Every package must include these sections:

1. Execution transcripts:
   - exact commands used
   - timestamped summary lines
2. Logs/traces/screenshots/audio (as applicable):
   - per-surface output logs
   - runtime traces or summaries
   - screenshot/audio links only for surfaces that produce them
3. Artifact manifest:
   - machine-readable manifest (for example `.tau/live-run-unified/manifest.json`)
   - machine-readable report (for example `.tau/live-run-unified/report.json`)
4. Go/no-go summary:
   - explicit `GO` or `NO-GO`
   - owner + reviewer names
   - decision timestamp (UTC)
   - rollback trigger status

## Minimal package layout

Store package evidence in a deterministic path (or CI artifact bundle):

```text
.tau/validation-package/<release-id>/
  checklist.md
  command-transcript.log
  go-no-go-summary.md
  live-run-unified/
    manifest.json
    report.json
    surfaces/<surface>/stdout.log
    surfaces/<surface>/stderr.log
```

## Checklist template

Copy this into a release issue/PR comment or package file:

```markdown
## Final Validation Package: <release-id>

- Decision owner: <name>
- Reviewer(s): <name(s)>
- Decision timestamp (UTC): <YYYY-MM-DDTHH:MM:SSZ>
- Decision: <GO|NO-GO>

### 1) Execution transcripts
- Command: <exact command>
  - Evidence: <link/path>
- Command: <exact command>
  - Evidence: <link/path>

### 2) Logs/traces/screenshots/audio
- Voice: <link/path or n/a>
- Browser: <link/path or n/a>
- Dashboard: <link/path or n/a>
- Custom command: <link/path or n/a>
- Memory: <link/path or n/a>

### 3) Artifact manifest
- Manifest: <link/path>
- Report: <link/path>

### 4) Rollback readiness + go/no-go summary
- Rollback trigger matrix review evidence: <link/path>
- Current trigger status: <none active | list active triggers>
- Final decision rationale: <short rationale>
```

## Assembly workflow

1. Run unified harness:
   `./scripts/demo/live-run-unified.sh --skip-build --timeout-seconds 180 --keep-going`
2. Save transcript output to package evidence.
3. Attach `manifest.json`, `report.json`, and per-surface logs.
4. Complete release sign-off checklist.
5. Fill final validation package template and record explicit decision.
