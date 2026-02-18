# Spec #2487 - implement ArcSwap + notify runtime heartbeat profile-policy hot-reload

Status: Implemented

## Problem Statement
`G16` phase-2 requires lock-free runtime config swaps and file-watch-driven updates. The current heartbeat reload path polls `*.policy.json` and mutates a local config struct each tick.

## Acceptance Criteria
### AC-1 Runtime watcher applies valid TOML policy updates
Given a running heartbeat scheduler and a watched policy file `<state-path>.policy.toml`, when `interval_ms` is changed to a valid value, then snapshots converge to the new interval and scheduler remains running.

### AC-2 No-change cycles do not emit false reload signals
Given no effective policy change, when additional heartbeat cycles execute, then interval remains stable and no hot-reload-applied reason code is emitted.

### AC-3 Invalid TOML/policy fails closed
Given an invalid TOML payload or invalid `interval_ms` (0), when reload is triggered, then scheduler preserves last-known-good interval and emits `heartbeat_hot_reload_policy_invalid` diagnostics.

### AC-4 Active scheduler config uses lock-free swaps
Given runtime reload evaluations, when updates are applied, then reads for cycle execution use an ArcSwap-backed active config snapshot (no mutex lock required for read path).

## Scope
In scope:
- `tau-runtime` heartbeat hot-reload implementation.
- Workspace dependency additions needed for this slice (`notify`, `arc-swap`).
- Conformance/regression tests in heartbeat runtime module.

Out of scope:
- Full profile-level config object hot reload across crates.
- Non-heartbeat runtime policy updates.

## Conformance Cases
- C-01 (AC-1, integration): `integration_spec_2487_c01_runtime_heartbeat_profile_toml_hot_reload_applies_interval_update`
- C-02 (AC-2, regression): `regression_spec_2487_c02_runtime_heartbeat_profile_toml_without_change_keeps_interval_stable`
- C-03 (AC-3, regression): `regression_spec_2487_c03_runtime_heartbeat_invalid_profile_toml_preserves_last_good_interval`
- C-04 (AC-4, functional): `functional_spec_2487_c04_runtime_heartbeat_hot_reload_uses_arc_swap_active_config`

## Success Metrics
- C-01..C-04 all pass.
- Existing `spec_2465` heartbeat hot-reload tests are superseded/updated and remain green for equivalent behavior.
- No scheduler panic/crash when policy file is malformed or replaced atomically.
