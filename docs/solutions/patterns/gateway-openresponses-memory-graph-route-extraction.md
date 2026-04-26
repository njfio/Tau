---
title: Gateway memory graph route extraction
category: patterns
date: '2026-04-26'
tags:
  - gateway
  - openresponses
  - tests
  - route-split
  - memory-graph
---

# Gateway memory graph route extraction
## Problem
The Gateway OpenResponses parent test module still contained the focused `/gateway/memory/{session_key}/graph` route test after the base memory API route test was split into a sibling module.
## Root cause
Memory-related tests cover multiple surfaces: the Gateway memory graph route, the compatibility `/api/memories/graph` route, unauthorized regressions, memory-entry CRUD/search, and Ops memory shell behavior. Moving them together would have mixed route contracts and made review riskier.
## Solution
Extract only `functional_gateway_memory_graph_endpoint_returns_filtered_relations` into `crates/tau-gateway/src/gateway_openresponses/tests/gateway_memory_graph_api.rs`, add `mod gateway_memory_graph_api;` to the parent module, and keep the new sibling using `use super::*;` for existing helpers and constants. Leave the `/api/memories/graph` compatibility and unauthorized tests in the parent for their own later slice.
## Prevention

For future memory-adjacent route splits, separate Gateway route templates from `/api/*` compatibility routes, list explicit non-targets in a Gyre map, and verify with Gateway OpenResponses tests, fmt, clippy, and manifest-drift checks before committing.
