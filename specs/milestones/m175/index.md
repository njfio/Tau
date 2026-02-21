# M175 - Gateway OpenResponses Module Split Phase 5 (Channel and Telemetry Runtime)

## Context
After phase 4, `crates/tau-gateway/src/gateway_openresponses.rs` is still above the 2000-line hotspot threshold. Channel lifecycle and UI telemetry handlers remain inline and can be extracted as a cohesive runtime domain.

## Scope
- Extract channel lifecycle and UI telemetry handlers.
- Move tightly-coupled helper plumbing for these handlers.
- Preserve route wiring and behavioral contracts.

## Linked Issues
- Epic: #2986
- Story: #2987
- Task: #2988
