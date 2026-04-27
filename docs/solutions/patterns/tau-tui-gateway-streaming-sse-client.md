---
title: tau-tui gateway streaming SSE client
category: patterns
date: '2026-04-27'
tags:
  - tau-tui
  - gateway
  - sse
  - streaming
  - tool-lifecycle
  - issue-3669
related:
  - specs/3669/spec.md
  - specs/3669/plan.md
  - crates/tau-tui/src/interactive/gateway_client.rs
  - crates/tau-tui/src/interactive/app_gateway_tests.rs
---

# tau-tui gateway streaming SSE client
## Problem
The tau-tui interactive client posted non-streaming /v1/responses requests and only updated chat/tools after the gateway returned a final JSON response, making Ralph-loop progress appear frozen during long turns.
## Root cause
The TUI modeled a gateway turn as one terminal Result from a blocking JSON request, so it had no intermediate event channel for response.output_text.delta or response.tool_execution lifecycle SSE frames.
## Solution
Change the TUI gateway worker to request stream:true, parse text/event-stream frames line by line, and send GatewayTurnEvent values over the existing worker channel. App::tick drains those events, appends assistant deltas to a streaming assistant message, maps tool start/completion frames into the ToolPanel, and preserves final completion/error handling through a Finished event.
## Prevention

Keep deterministic app_gateway_tests coverage for stream:true request bodies, incremental response.output_text.delta handling, response.tool_execution.started/completed tool-panel updates, and response.failed error state. Preserve the JSON fallback path for non-streaming test fixtures.
