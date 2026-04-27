---
title: TUI Operator State Consumption Adapter
category: patterns
date: '2026-04-27'
tags:
  - tau-tui
  - operator-state
  - OperatorTurnState
  - transcript-first
  - adapter
---

# TUI Operator State Consumption Adapter
## Problem
Tau TUI needed to consume the shared OperatorTurnState contract without removing the existing gateway SSE parser or duplicating the schema locally.
## Root cause
The TUI previously updated transcript, status, and tool rows directly from gateway-specific event variants, while #3581 moved the durable operator turn vocabulary into tau-contract.
## Solution
Add a small additive interactive::operator_state adapter in tau-tui that imports tau_contract::operator_state::OperatorTurnState, binds session and mission identity, writes assistant_text into the transcript, reconciles tools by tool_call_id, and surfaces blocked/timeout/failure reason codes as system messages. Keep GatewayTurnEvent parsing unchanged until the gateway emits shared snapshots.
## Prevention

When adding a new operator surface, consume tau-contract state through a narrow adapter and keep transport-specific streaming code separate. Add RED tests for assistant text, tool reconciliation, timeout errors, and blocked mission status before wiring the dependency.
