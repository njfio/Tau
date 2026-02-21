# M173 - Gateway OpenResponses Module Split Phase 3 (OpenAI Compat Runtime)

## Context
`crates/tau-gateway/src/gateway_openresponses.rs` remains a hotspot after phase 2. This milestone extracts OpenAI compatibility runtime handlers into a dedicated module while preserving behavior and telemetry contracts.

## Scope
- Extract OpenAI compatibility endpoint handlers from `gateway_openresponses.rs`.
- Keep route constants and endpoint contracts unchanged.
- Validate behavior with existing OpenAI compatibility tests and quality gates.

## Linked Issues
- Epic: #2978
- Story: #2979
- Task: #2980
