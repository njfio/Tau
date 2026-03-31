# Plan: Issue #3659 - Expose gateway mission inspection and TUI resume controls for checkpointed Ralph-loop missions

## Approach
1. Add gateway mission inventory/detail endpoints that read the existing
   persisted mission state instead of inventing a second operator store.
2. Reuse the interactive TUI's command surface to call those endpoints for
   mission listing, inspection, and resume.
3. Make resume explicit in the TUI by binding both `mission_id` and
   `session_key` into the active gateway runtime config before the next turn.
4. Keep compatibility-safe behavior:
   - normal interactive chat without a resumed mission still uses the current
     session-key-only path
   - mission resume is additive and opt-in

## Proposed Design
### Gateway mission API
- Add `GET /gateway/missions?limit=N`
- Add `GET /gateway/missions/{mission_id}`
- Response model should derive from `GatewayMissionState` and expose:
  - `mission_id`
  - `session_key`
  - `status`
  - `goal_summary`
  - `latest_output_summary`
  - `latest_verifier`
  - `latest_completion`
  - `iteration_count`
  - `updated_unix_ms`
- Keep the detail route close to the raw mission state so future operator UIs
  can reuse it without another adapter layer.

### TUI mission control
- Extend `GatewayRuntimeConfig` with optional `mission_id`
- Add gateway client helpers for mission list/detail
- Add command handling for:
  - `missions`
  - `mission <id>`
  - `resume <id>`
- Resume behavior:
  - fetch mission detail from gateway
  - set `config.gateway.mission_id = Some(mission_id)`
  - set `config.gateway.session_key = detail.session_key`
  - update status/chat with an operator-visible summary

### Status surface
- Add an active mission label to the interactive status bar so operators can see
  when they are continuing a persisted mission instead of the default implicit
  path.

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3659"
  change_surface:
    - symbol: "Gateway operator API"
      location: "crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs"
      change_type: "addition"
      current: "no public mission inventory/detail surface exists"
      proposed: "gateway exposes mission list and mission detail endpoints"
      compatibility: "additive"
      reason: "new authorized routes only"
    - symbol: "Interactive TUI gateway runtime config"
      location: "crates/tau-tui/src/interactive/gateway_client.rs"
      change_type: "modification"
      current: "interactive turns bind only session_id metadata"
      proposed: "interactive turns may also bind an explicit mission_id"
      compatibility: "additive"
      reason: "mission binding is optional and existing callers keep default behavior"
  overall_compatibility: "safe"
  approach:
    strategy: "Additive mission-control surfaces over existing persisted state"
    steps:
      - "Expose gateway mission inventory/detail endpoints"
      - "Teach the TUI to inspect and resume missions through those endpoints"
      - "Display the active mission so operators can see resumed context"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: TUI resume points at the wrong session lineage.
  Mitigation: source the session key directly from the persisted mission detail
  endpoint and bind both mission and session together.
- Risk: mission list/detail responses drift from persisted mission shape.
  Mitigation: build list/detail responses from `GatewayMissionState` fields
  rather than re-deriving status from logs.
- Risk: operator surfaces become ambiguous about which mission is active.
  Mitigation: show the active mission id in the TUI status bar after resume.

## Verification
- Gateway regression coverage for mission list and mission detail endpoints
- Interactive TUI command tests for listing missions, resuming a mission, and
  including `mission_id` in the next request
- Existing mission/verifier/learning gateway tests remain green
