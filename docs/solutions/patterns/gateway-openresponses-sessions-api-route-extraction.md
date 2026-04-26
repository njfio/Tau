---
title: Gateway OpenResponses sessions API route extraction
category: patterns
date: '2026-04-26'
tags:
  - tau-gateway
  - openresponses
  - test-splitting
  - sessions-api
  - route-tests
---

# Gateway OpenResponses sessions API route extraction
## Problem
The Gateway OpenResponses test module still contains route endpoint families that are easy to over-extract when splitting the monolith, especially around the sessions API followed immediately by memory endpoint tests.
## Root cause
The sessions API route test shares parent-scope helpers and endpoint constants with adjacent memory route tests, so moving a broad block would mix unrelated behavior and make focused review harder.
## Solution
Map the target first, then extract only `functional_gateway_sessions_endpoints_support_list_detail_append_and_reset` into `crates/tau-gateway/src/gateway_openresponses/tests/gateway_sessions_api.rs` with `use super::*;`. Add `mod gateway_sessions_api;` to the parent module and leave Gateway memory, memory entry, memory graph, Ops sessions, and webchat tests in their existing locations for dedicated future slices.
## Prevention

For future Gateway route slices, require a `.gyre/*-route-slice.md` map that names the exact target test and non-target neighbors before editing, then verify `cargo test -p tau-gateway gateway_openresponses --lib`, `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`, and no Cargo manifest or lockfile drift.
