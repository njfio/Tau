# ADR-003: Runtime Heartbeat Hot-Reload Uses `notify` + `arc-swap` + TOML Policy

## Context
`G16` phase-2 requires runtime heartbeat interval updates without process restart, while preserving fail-closed behavior for invalid configuration.

The prior implementation polled a sidecar file every cycle and used a mutable local scheduler config. This was sufficient for phase-1 but does not satisfy phase-2 goals for:
- event-driven reload triggers,
- lock-free read path for active runtime config,
- profile-policy alignment using TOML.

## Decision
For heartbeat runtime hot-reload:
- Use `notify` to watch `<state-path>.policy.toml` parent directory and mark reload as pending on matching file events.
- Use `arc-swap` (`ArcSwap<RuntimeHeartbeatSchedulerConfig>`) as the active config holder so cycle reads remain lock-free.
- Parse policy with `toml` and validate `interval_ms` before applying updates.
- Keep fail-closed semantics: invalid read/parse/value preserves last-known-good config and emits `heartbeat_hot_reload_policy_invalid` diagnostics.

## Consequences
### Positive
- Reload path is event-driven rather than poll/fingerprint-driven.
- Active config reads avoid mutex locking during heartbeat cycles.
- Policy format aligns with profile-policy TOML direction.
- Invalid updates remain observable and non-disruptive.

### Negative
- Adds three workspace dependencies (`notify`, `arc-swap`, `toml`).
- File watcher behavior may vary across platforms and can emit coalesced/noisy events.

### Mitigations
- Treat any relevant watcher event as dirty and re-parse current file content.
- Maintain deterministic reason codes and diagnostics for watch/read/parse failures.
- Cover update/no-change/invalid paths with conformance and regression tests.
