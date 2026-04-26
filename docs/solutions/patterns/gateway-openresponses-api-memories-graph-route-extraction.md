---
title: Gateway API memories graph route extraction
category: patterns
date: '2026-04-26'
tags:
  - gateway
  - openresponses
  - tests
  - route-split
  - api-memories-graph
---

# Gateway API memories graph route extraction
## Problem
The Gateway OpenResponses parent test module still contained the `/api/memories/graph` compatibility route tests after the Gateway memory graph route had been moved into its own sibling module.
## Root cause
Memory graph behavior has two public surfaces: the Gateway session-template route and the `/api/memories/graph` compatibility route. Keeping both in the parent made the test module harder to scan, but moving them with memory-entry tests would have mixed unrelated route contracts.
## Solution
Extract `integration_spec_2726_c01_api_memories_graph_endpoint_returns_filtered_relations` and `regression_spec_2726_c02_api_memories_graph_endpoint_rejects_unauthorized_requests` into `crates/tau-gateway/src/gateway_openresponses/tests/api_memories_graph.rs`, add `mod api_memories_graph;` to the parent module, and keep the sibling using `use super::*;` for existing helpers/constants. Leave memory-entry CRUD/search and legacy Gateway memory behavior in the parent for later slices.
## Prevention

When splitting Gateway memory tests, separate `/gateway/*` route templates from `/api/*` compatibility routes, keep success and auth regression coverage for the same route together, and verify with Gateway OpenResponses tests, fmt, clippy, and manifest-drift checks before committing.
