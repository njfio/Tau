---
title: Define shared operator state contracts fixture-first
category: patterns
date: '2026-04-27'
tags:
  - pattern
  - operator-state
  - contracts
  - tau-contract
  - tdd
  - tui
  - webchat
related:
  - docs/adrs/0005-shared-operator-turn-task-state.md
  - docs/architecture/shared-operator-state-v1.md
  - crates/tau-contract/src/operator_state.rs
  - specs/3581/spec.md
---

# Define shared operator state contracts fixture-first
## Problem
TUI, webchat, gateway missions, and dashboard streams can drift when each surface represents the same operator turn with local event/state shapes instead of a shared contract.
## Root cause
The runtime and UI surfaces evolved around legacy SSE frames and snapshots, while tau-contract only provided generic fixture helpers and had no operator-state schema for parity tests.
## Solution
Use a spec and ADR first, then add RED tau-contract tests for the desired schema, implement additive serde types in tau-contract, and document runtime mappings plus legacy compatibility boundaries before wiring clients. Preserve existing response.* SSE and mission/dashboard endpoints while future clients opt into OperatorTurnState fixtures.
## Prevention

For cross-client operator state work, add the shared contract and fixtures before runtime/UI migration. Keep schema changes additive, verify Cargo manifests stay untouched when no dependency is needed, and require a separate ADR or migration guide before removing legacy gateway surfaces.
