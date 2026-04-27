---
title: TUI Live Operator State Snapshot Consumption
category: patterns
date: '2026-04-27'
tags:
  - tau-tui
  - OperatorTurnState
  - SSE
  - snapshot
  - transcript-first
---

# TUI Live Operator State Snapshot Consumption
## Problem
tau-tui needed to accept shared OperatorTurnState snapshots from the live gateway SSE stream while continuing to support existing response.output_text.delta and tool lifecycle frames.
## Root cause
The first #3582 adapter slice made OperatorTurnState applyable inside TUI, but gateway_client still ignored any shared-state frame and App only consumed gateway-specific GatewayTurnEvent variants.
## Solution
Add an additive GatewayTurnEvent::OperatorStateSnapshot variant, parse response.operator_turn_state.snapshot SSE data as tau_contract::operator_state::OperatorTurnState, and route the snapshot through interactive::operator_state::apply_operator_turn_state. Reconcile snapshot assistant_text with the current assistant message so delta-plus-snapshot streams do not duplicate transcript rows.
## Prevention

Before adding a new shared-state stream event, record the event name in an ADR, add RED tests for snapshot-only and mixed legacy-delta/snapshot streams, and keep legacy response.* event handling unchanged.
