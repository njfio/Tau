# Spec #2486 - apply profile-policy TOML watcher updates to runtime heartbeat config

Status: Implemented

## Problem Statement
Current runtime heartbeat hot-reload relies on per-tick polling of a JSON sidecar file. This misses `G16` phase-2 goals for file-watch-driven config updates and lock-free config replacement semantics.

## Acceptance Criteria
### AC-1 Watcher-driven reload attempts are available
Given a running heartbeat scheduler, when the profile-policy TOML file changes, then runtime evaluates reload without requiring restart.

### AC-2 Valid policy updates apply atomically
Given a valid profile-policy TOML update, when reload evaluation succeeds, then the active heartbeat config swaps atomically and subsequent snapshots reflect the new effective interval.

### AC-3 Invalid updates fail closed and remain observable
Given malformed/invalid profile-policy TOML, when reload evaluation runs, then last-known-good config remains active and invalid reason codes/diagnostics are emitted.

## Scope
In scope:
- Watcher integration for heartbeat profile-policy TOML file.
- ArcSwap-backed active heartbeat config pointer.
- Deterministic validation + fail-closed semantics.

Out of scope:
- Non-heartbeat config reload surfaces.
- Multi-process routing config (`G15`).

## Conformance Cases
- C-01 (AC-1, integration): watcher observes policy file update and triggers reload path.
- C-02 (AC-2, integration): valid policy update changes effective heartbeat interval.
- C-03 (AC-3, regression): invalid TOML/policy keeps prior interval and records invalid reason code.

## Success Metrics
- C-01..C-03 all pass in scoped runtime tests.
- Heartbeat scheduler remains running across valid/invalid reload attempts.
