# Plan: Issue #2862 - chat token-counter marker contracts

## Approach
1. Add deterministic chat token-counter marker attributes in `tau-dashboard-ui` using existing active-session usage snapshot values.
2. Add UI and gateway tests for marker presence/value correctness on `/ops`, `/ops/chat`, and `/ops/sessions`.
3. Add visible aggregate rows for rendered assistant token streams while keeping stream aggregates zeroed on hidden non-chat routes.
4. Wrap token accounting details in a collapsed-by-default secondary manager so the transcript remains the default chat surface.
5. Make the collapsed token-counter summary identify the source of its headline count: persisted usage tokens when available, rendered assistant stream tokens when usage totals are unavailable.
6. Re-run chat/session/detail regression suites and full verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: existing chat marker substring assertions break due attribute ordering changes.
  - Mitigation: preserve existing attribute ordering and place new marker in additive element.
- Risk: hidden-route behavior drift.
  - Mitigation: explicit integration checks on `/ops` and `/ops/sessions` plus `spec_2858` rerun.
- Risk: rendered stream-token totals are mistaken for persisted provider usage accounting.
  - Mitigation: keep persisted input/output/total token fields separate from assistant stream count/token aggregates.
- Risk: always-visible token accounting distracts from the primary chat transcript.
  - Mitigation: preserve the token-counter article and markers inside a collapsed `<details>` manager with a concise total/stream summary.
- Risk: the collapsed summary says `0 total` while assistant stream-token rows prove non-zero rendered tokens.
  - Mitigation: add summary-specific markers and use rendered stream-token totals with an `assistant-stream` source label whenever persisted provider usage totals are zero.

## Interface / Contract Notes
- Additive SSR marker only.
- Token counter exposes `data-assistant-stream-count`, `data-assistant-stream-tokens`, and `data-latest-assistant-token-count` from rendered assistant transcript rows.
- Token counter is wrapped by `id="tau-ops-chat-token-counter-details"` with `data-collapsed-by-default="true"` and deterministic session/total markers.
- Token counter summary exposes `data-summary-tokens` and `data-summary-token-source` on both the collapsed manager and detail article; source is `usage` when usage totals exist and `assistant-stream` otherwise.
- No endpoint or payload schema changes.
