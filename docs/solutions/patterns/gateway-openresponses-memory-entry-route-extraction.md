---
title: Gateway OpenResponses memory-entry route extraction
category: patterns
date: '2026-04-27'
tags:
  - tau-gateway
  - gateway-openresponses
  - route-extraction
  - memory-entry
  - tests
---

# Gateway OpenResponses memory-entry route extraction
## Problem
Gateway OpenResponses memory-entry CRUD/search and unauthorized route tests lived in the large parent test module, making route-family ownership harder to scan and increasing the risk of mixing direct Gateway memory-entry contracts with adjacent memory API, graph, E2E, ops, or product-proof coverage.
## Root cause
The parent Gateway OpenResponses test module accumulated many route families over time, and memory-entry behavior shared enough helper constants with neighboring memory routes that a split could accidentally pull unrelated coverage without an explicit boundary map.
## Solution
Create a small Gyre boundary map before moving code, then extract only integration_spec_2667_c01_memory_entry_endpoints_support_crud_search_and_legacy_compatibility and regression_spec_2667_c05_memory_entry_endpoints_reject_unauthorized_requests into crates/tau-gateway/src/gateway_openresponses/tests/gateway_memory_entries_api.rs with use super::* and a parent mod gateway_memory_entries_api declaration. Preserve policy-gate, search, legacy compatibility, unauthorized, and handle.abort assertions exactly, then run cargo test -p tau-gateway gateway_openresponses --lib, cargo fmt --check, cargo clippy -p tau-gateway --tests --no-deps -- -D warnings, and git diff --quiet -- Cargo.toml Cargo.lock.
## Prevention

For each future Gateway OpenResponses route split, write the target tests, target module, and explicit non-targets first. Keep direct /gateway/* route-template tests separate from /api/* compatibility routes, E2E harness usage, ops memory tests, and product-proof scripts unless the stage intentionally spans those surfaces.
