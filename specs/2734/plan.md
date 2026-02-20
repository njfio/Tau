# Plan: Issue #2734 - G18 stretch routines cron management webchat panel

## Approach
1. Add RED unit tests asserting routines tab markup and routines script markers.
2. Implement routines tab/view HTML controls in webchat.
3. Implement routines status renderer from `gateway.status.events` and jobs list/cancel handlers.
4. Reuse existing telemetry helper with routines-specific reason codes.
5. Run scoped verification and update checklist/tasks artifacts.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: new tab wiring could regress existing tab activation.
  - Mitigation: preserve generic tab activation and assert existing view markers remain.
- Risk: cancel action path mismatch with template endpoint.
  - Mitigation: use deterministic template replacement helper and targeted marker tests.

## Interfaces / Contracts
- Reuse existing endpoints:
  - `GET /gateway/status` (events diagnostics)
  - `GET /gateway/jobs`
  - `POST /gateway/jobs/{job_id}/cancel`
- No backend API changes.

## ADR
- Not required: no dependency/protocol change.
