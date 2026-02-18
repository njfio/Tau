# Plan #2487

## Approach
1. Replace per-tick fingerprint polling state with watcher-aware dirty flag tracking.
2. Add ArcSwap active config pointer and read config snapshot each cycle.
3. Add TOML policy parser for `<state-path>.policy.toml` (`interval_ms` initially).
4. Keep fail-closed invalid handling and deterministic reason-code diagnostics.
5. Add/update conformance tests first (RED), then implement (GREEN), then regressions.

## Risks / Mitigations
- Risk: watcher events are noisy/coalesced.
  Mitigation: treat any watcher event as "changed" and re-parse current file content.
- Risk: test flakiness around async reload timing.
  Mitigation: use bounded polling helpers for interval convergence + reason-code presence.

## Interfaces / Contracts
- Hot-reload policy path changes to `<state-path>.policy.toml`.
- Reason codes preserved:
  - `heartbeat_hot_reload_policy_applied`
  - `heartbeat_hot_reload_policy_invalid`
- Active cycle execution reads config through ArcSwap snapshot.
- ADR: `docs/architecture/adr-003-runtime-heartbeat-hot-reload-notify-arcswap.md`.
