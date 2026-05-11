# Plan: Issue #2866 - chat inline tool-result card contracts

## Approach
1. Add deterministic inline tool-card marker rendering in `tau-dashboard-ui` transcript rows for `role == "tool"`.
2. Add UI and gateway tests covering tool and non-tool row behavior on `/ops/chat`, plus hidden-panel omission on `/ops` and `/ops/sessions`.
3. Re-run required chat/panel regression suites and full verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: introducing markup changes that break existing chat row assertions.
  - Mitigation: keep existing row IDs/attributes stable and add card marker as additive nested element.
- Risk: route-hidden chat panel payload leakage.
  - Mitigation: explicit integration assertions that `/ops` and `/ops/sessions` preserve the hidden panel shell but omit transcript row payloads.

## Interface / Contract Notes
- Additive SSR marker behavior only.
- No transport/API/schema changes.
