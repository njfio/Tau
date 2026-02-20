# Spec: Issue #2750 - Discord guild allowlist filtering for live connectors (G10)

Status: Implemented

## Problem Statement
Tau live Discord polling currently accepts messages from configured channel IDs but does not expose an explicit guild-level allowlist. Operators need a first-class permission filter so ingress only processes Discord traffic from approved guild scopes.

## Acceptance Criteria

### AC-1 CLI and validation support Discord guild allowlist configuration
Given `--multi-channel-live-connectors-runner` with Discord polling enabled,
When operators pass Discord guild allowlist IDs,
Then CLI parsing, normalization, and validation accept non-empty IDs and preserve fail-closed behavior for invalid combinations.

### AC-2 Startup mapping propagates guild allowlist into live connector config
Given a parsed CLI instance with Discord guild IDs,
When `build_multi_channel_live_connectors_config` is called,
Then `MultiChannelLiveConnectorsConfig` includes normalized Discord guild allowlist IDs.

### AC-3 Polling ingress filters Discord messages by allowlisted guild IDs
Given Discord polling mode with guild allowlist configured,
When fetched messages include mixed guild IDs (or no guild ID),
Then only messages from allowlisted guild IDs are ingested into live ingress NDJSON.

### AC-4 Backward compatibility is preserved when no guild allowlist is configured
Given Discord polling mode with no guild allowlist configured,
When messages are fetched,
Then existing ingestion behavior remains unchanged.

### AC-5 Verification and roadmap evidence are updated
Given implementation is complete,
When scoped quality gates and live validation run,
Then tests pass and `tasks/spacebot-comparison.md` marks G10 guild/channel filtering with issue evidence.

## Scope

### In Scope
- CLI flag/env for Discord guild allowlist IDs.
- CLI validation adjustments for new allowlist input.
- Startup config mapping into multi-channel live connector runtime config.
- Discord polling filter enforcement in `tau-multi-channel`.
- Unit/functional/integration/regression tests and roadmap checklist update.

### Out of Scope
- Serenity dependency introduction.
- New Discord transport mode (webhook remains unsupported).
- Thread creation or streaming-edit transport semantics.

## Conformance Cases
- C-01 (unit): CLI default and explicit overrides include Discord guild allowlist field.
- C-02 (functional): CLI validation rejects polling mode when guild allowlist values are provided but empty/whitespace-only.
- C-03 (integration): startup config builder maps trimmed guild allowlist values to runtime config.
- C-04 (functional): live connector poll cycle ingests only allowlisted Discord guild messages.
- C-05 (regression): live connector poll cycle behavior remains unchanged without guild allowlist.
- C-06 (verify/live): fmt, clippy, targeted tests, and localhost live validation pass.

## Success Metrics / Observable Signals
- Operators can constrain Discord ingestion with guild allowlists via CLI/env.
- Off-scope guild messages are filtered before normalization/ingress writes.
- G10 checklist item for guild/channel filtering is closed with linked issue evidence.
