# Plan: Issue #2750 - Discord guild allowlist filtering for live connectors (G10)

## Approach
1. Add RED tests for CLI defaults/overrides, validation, startup config mapping, and Discord poll filtering behavior.
2. Extend CLI args + validation with Discord guild allowlist input.
3. Extend `MultiChannelLiveConnectorsConfig` and startup mapping for normalized guild allowlist IDs.
4. Enforce guild filtering in `poll_discord_messages` before payload ingestion.
5. Run scoped gates + live localhost validation and update checklist evidence.

## Affected Modules
- `crates/tau-cli/src/cli_args.rs`
- `crates/tau-cli/src/validation.rs`
- `crates/tau-coding-agent/src/tests/cli_validation.rs`
- `crates/tau-onboarding/src/startup_transport_modes.rs`
- `crates/tau-onboarding/src/startup_transport_modes/tests.rs`
- `crates/tau-multi-channel/src/multi_channel_live_connectors.rs`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: over-filtering could drop valid Discord events.
  - Mitigation: only enforce guild filter when allowlist is non-empty; add regression test for empty allowlist behavior.
- Risk: CLI surface drift across startup/test crates.
  - Mitigation: add tests at CLI parsing, validation, and startup config layers.

## Interfaces / Contracts
- New CLI option:
  - `--multi-channel-discord-ingress-guild-id`
  - env: `TAU_MULTI_CHANNEL_DISCORD_INGRESS_GUILD_ID`
  - repeatable / CSV-delimited, normalized by trim.
- Runtime contract:
  - `MultiChannelLiveConnectorsConfig.discord_ingress_guild_ids: Vec<String>`
  - Discord polling ingests only events whose `guild_id` matches allowlist when allowlist is non-empty.

## ADR
- Not required: no dependency or protocol changes.
