---
title: Gateway OperatorTurnState Snapshot SSE Emission
category: patterns
date: '2026-04-27'
tags:
  - tau-gateway
  - OperatorTurnState
  - SSE
  - response.operator_turn_state.snapshot
  - compatibility
---

# Gateway OperatorTurnState Snapshot SSE Emission
## Problem
The TUI could consume response.operator_turn_state.snapshot frames, but gateway streaming responses did not emit a shared OperatorTurnState snapshot for live clients.
## Root cause
Gateway streaming had legacy response.created, output_text, completion, failure, and tool events, while shared operator turn state was introduced later in tau-contract and had not been projected onto the SSE transport.
## Solution
Emit an additive response.operator_turn_state.snapshot SseFrame::Json before response.completed on successful /v1/responses streams. Build the payload from tau_contract::operator_state::OperatorTurnState using the response id as turn_id, translated session/mission identity, completed/succeeded status, assistant output text, and a final_answer event. Keep all legacy response.* frames unchanged.
## Prevention

For future shared-state transport additions, add RED stream tests that assert both the new event and the legacy compatibility frames, and keep the event additive/ignorable by clients that have not opted in.
