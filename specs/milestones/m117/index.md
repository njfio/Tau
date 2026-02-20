# M117 - Spacebot G18 Stretch Cron Management UI

## Context
`tasks/spacebot-comparison.md` still has G18 stretch cron-management parity unchecked. Tau already exposes scheduler diagnostics in `GET /gateway/status` (`events` report) and job controls in `/gateway/jobs` + `/gateway/jobs/{job_id}/cancel`, but webchat has no dedicated routines operator surface.

## Linked Work
- Epic: #2732
- Story: #2733
- Task: #2734
- Source parity checklist: `tasks/spacebot-comparison.md` (G18 stretch page)

## Scope
- Add a webchat routines/cron tab.
- Render scheduler diagnostics from gateway status events payload.
- Add jobs list and cancel controls using existing authenticated endpoints.

## Exit Criteria
- Operators can inspect scheduler health/counters/diagnostics in webchat.
- Operators can refresh jobs and cancel active jobs from webchat.
- Existing webchat tabs remain regression-safe.
