---
title: Gateway memory API route extraction
category: patterns
date: '2026-04-26'
tags:
  - gateway
  - openresponses
  - tests
  - route-split
  - memory
---

# Gateway memory API route extraction
## Problem
The Gateway OpenResponses parent test module still contained the focused `/gateway/memory/{session_key}` read/write policy-gate route test even though nearby route-family tests had been split into sibling modules.
## Root cause
The parent module accumulated multiple memory-related concerns: the legacy memory endpoint, memory-entry CRUD/search compatibility, memory graph routes, Ops memory UI tests, and product-proof documentation. Moving them together would have mixed behavior surfaces and increased review risk.
## Solution
Extract only `functional_gateway_memory_endpoint_supports_read_and_policy_gated_write` into `crates/tau-gateway/src/gateway_openresponses/tests/gateway_memory_api.rs`, add `mod gateway_memory_api;` to the parent module, and keep the new sibling using `use super::*;` to preserve existing helpers/constants without behavior changes. Leave memory-entry, graph, Ops memory, webchat, sessions, and product-proof script/docs files out of this slice.
## Prevention

For future Gateway route splits, write a boundary map first, move one route family at a time, list explicit non-targets, and verify with `cargo test -p tau-gateway gateway_openresponses --lib`, `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`, and manifest-drift checks.
