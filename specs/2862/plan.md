# Plan: Issue #2862 - chat token-counter marker contracts

## Approach
1. Add deterministic chat token-counter marker attributes in `tau-dashboard-ui` using existing active-session usage snapshot values.
2. Add UI and gateway tests for marker presence/value correctness on `/ops`, `/ops/chat`, and `/ops/sessions`.
3. Add visible aggregate rows for rendered assistant token streams while keeping stream aggregates zeroed on hidden non-chat routes.
4. Re-run chat/session/detail regression suites and full verification gates.

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

## Interface / Contract Notes
- Additive SSR marker only.
- Token counter exposes `data-assistant-stream-count`, `data-assistant-stream-tokens`, and `data-latest-assistant-token-count` from rendered assistant transcript rows.
- No endpoint or payload schema changes.
