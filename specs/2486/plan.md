# Plan #2486

## Approach
1. Introduce watcher-aware hot-reload state in `tau-runtime` heartbeat module.
2. Store active heartbeat scheduler config in `ArcSwap<RuntimeHeartbeatSchedulerConfig>`.
3. On watcher-triggered changes: parse TOML policy, validate, swap active config, and preserve last-known-good on failure.
4. Preserve existing heartbeat cycle contracts and snapshot diagnostics.

## Risks / Mitigations
- Risk: notify backend event differences across OS targets.
  Mitigation: treat events as "dirty" signals; re-read file for authoritative state.
- Risk: concurrent reads while updating config.
  Mitigation: use ArcSwap atomic pointer swaps with immutable config snapshots.

## Interfaces / Contracts
- Internal heartbeat policy file contract changes from polled JSON sidecar to watched TOML policy sidecar.
- Existing snapshot reason-code contracts remain stable; invalid reloads continue to emit deterministic diagnostics.
- ADR: `docs/architecture/adr-003-runtime-heartbeat-hot-reload-notify-arcswap.md`.
