---
title: Gateway payload tracing and TUI tool-call-id lifecycle reconciliation
category: patterns
date: '2026-04-27'
tags:
  - tau-gateway
  - tau-tui
  - openresponses
  - tool-call-id
  - payload-tracing
  - mission-iterations
  - sse
related:
  - specs/3671/spec.md
  - crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs
  - crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs
  - crates/tau-tui/src/interactive/tools.rs
  - crates/tau-tui/src/interactive/app_gateway_tests.rs
---

# Gateway payload tracing and TUI tool-call-id lifecycle reconciliation
## Problem
Gateway OpenResponses mission attempts preserved lossy summaries only, while the TUI could receive concurrent same-name tool executions and needed to reconcile streamed completion frames without leaving stale running rows.
## Root cause
The gateway mission iteration schema lacked structured request and response payload evidence, and the TUI reducer discarded streamed tool_call_id values before falling back to name-based completion matching.
## Solution
Add defaulted request_payload and response_payload fields to GatewayMissionIterationRecord and populate them from each OpenResponses attempt using serialized messages and observed tool execution payloads. In tau-tui, store tool_call_id on ToolEntry, start streamed tool rows with the id, and complete rows by id before falling back to legacy name matching. Cover the boundary with a persisted mission payload regression and a TUI regression that starts two read_file calls with different ids and completes them independently.
## Prevention

When streaming tool lifecycle events into UI state, preserve provider/gateway correlation ids all the way into the reducer. For gateway retry/mission traces, store bounded structured evidence fields additively with serde defaults so older state files remain readable while new attempts are inspectable.
