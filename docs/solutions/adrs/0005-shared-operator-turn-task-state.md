---
title: 'ADR 0005: Define shared operator turn/task state in tau-contract'
category: adrs
date: '2026-04-27'
tags:
  - adr
  - operator-state
  - tau-contract
  - tui
  - webchat
  - gateway
related:
  - docs/adrs/0005-shared-operator-turn-task-state.md
  - specs/3581/spec.md
---

# ADR 0005: Define shared operator turn/task state in tau-contract
## Problem
TUI, webchat, gateway mission APIs, and dashboard streams describe the same operator-visible runtime turn through different local shapes, which can make timeouts, blocked missions, partial output, and tool failures render inconsistently across clients.
## Root cause
The existing gateway SSE, TUI parser, webchat JavaScript, mission snapshot, and dashboard stream surfaces evolved independently without a shared additive contract fixture that both UI clients can validate against.
## Solution
Define OperatorTurnState v1 and supporting phase, status, event, tool, error, and fixture types in tau-contract as an additive serde-serializable contract while preserving legacy response.* SSE and mission/dashboard endpoints during migration.
## Prevention

Keep future operator-facing client work fixture-first against tau-contract, preserve legacy SSE compatibility until consumers migrate, and require a new ADR or migration guide before removing or renaming existing gateway surfaces.
