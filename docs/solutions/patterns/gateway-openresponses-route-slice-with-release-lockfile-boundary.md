---
title: Gateway OpenResponses route slice with release lockfile boundary
category: patterns
date: '2026-04-26'
tags:
  - gateway-openresponses
  - route-split
  - cargo-lock
  - release-boundary
  - v0.2.0
related:
  - crates/tau-gateway/src/gateway_openresponses/tests.rs
  - crates/tau-gateway/src/gateway_openresponses/tests/ops_memory.rs
  - .gyre/gateway-openresponses-ops-memory-route-slice.md
  - Cargo.lock
---

# Gateway OpenResponses route slice with release lockfile boundary
## Problem
A Gateway OpenResponses route-family extraction overlapped with a local v0.2.0 release metadata state where running Cargo tests updated Cargo.lock version metadata.
## Root cause
The local release commit updated workspace package version metadata without Cargo.lock, and `cargo test` synchronized lockfile package versions while the Gateway test split was being verified. Treating all lockfile drift as unrelated would have blocked the route extraction, while accepting arbitrary lockfile drift would weaken release discipline.
## Solution
Move the route-family tests mechanically into a sibling module, run targeted `tau-gateway` tests, rustfmt, and clippy, and guard Cargo.lock with a predicate that accepts only balanced `version = "0.1.0"` to `version = "0.2.0"` metadata changes. Keep `v0.2.0` tag creation, push, and publication out of scope until explicitly approved.
## Prevention

When release metadata has changed locally, prefer a lockfile-specific predicate over a blanket `git diff --quiet -- Cargo.lock` in unrelated verification stages, and surface any expected lockfile sync as a user-approved boundary before continuing.
