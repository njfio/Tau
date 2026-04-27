---
title: 'ADR 0006: Additive OperatorTurnState Snapshot SSE Event'
category: adrs
date: '2026-04-27'
tags:
  - ADR-0006
  - OperatorTurnState
  - SSE
  - tau-tui
  - gateway
---

# ADR 0006: Additive OperatorTurnState Snapshot SSE Event
## Problem
TUI can apply OperatorTurnState snapshots locally, but the live gateway stream still only carries legacy response.* SSE frames, so shared-state consumption needs a stable additive transport event.
## Root cause
The shared operator-state contract was defined after gateway and client streaming paths already depended on response.output_text.delta, response.tool_execution.*, response.completed, and response.failed frames.
## Solution
Use an optional response.operator_turn_state.snapshot SSE event whose data payload is a serialized OperatorTurnState. TUI parses and applies the snapshot through its adapter while continuing to accept all legacy frames.
## Prevention

Treat new gateway stream frames as additive, document the event name in an ADR before code changes, and add compatibility tests proving legacy delta/tool/final handling survives snapshot consumption.
