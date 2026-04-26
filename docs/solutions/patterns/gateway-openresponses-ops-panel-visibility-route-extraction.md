---
title: Gateway OpenResponses Ops panel visibility route extraction
category: patterns
date: '2026-04-26'
tags:
  - rust
  - tests
  - tau-gateway
  - openresponses
  - route-tests
  - refactor
  - ops-panel-visibility
related:
  - .gyre/gateway-openresponses-ops-panel-visibility-route-slice.md
  - crates/tau-gateway/src/gateway_openresponses/tests.rs
  - crates/tau-gateway/src/gateway_openresponses/tests/ops_panel_visibility.rs
  - docs/solutions/patterns/map-before-rust-test-route-body-extraction.md
---

# Gateway OpenResponses Ops panel visibility route extraction
## Problem
After multiple Gateway OpenResponses route-body slices, two compact Ops panel visibility tests remained in the parent module between larger chat/session tests and the webchat shell test.
## Root cause
The visibility tests belong to the Ops route shell surface, but they sit after broader chat/session behavior and immediately before `functional_webchat_endpoint_returns_html_shell`. Without a precise map, they could be skipped or accidentally batched with much larger chat, session, or webchat API tests.
## Solution
Map the exact two `functional_spec_2858` tests, move only those bodies into `crates/tau-gateway/src/gateway_openresponses/tests/ops_panel_visibility.rs`, wire the parent with `mod ops_panel_visibility;`, and leave chat/session behavior plus `functional_webchat_endpoint_returns_html_shell` in the parent for later slices. Verification passed with `cargo test -p tau-gateway gateway_openresponses --lib`, `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`, clean index/dependency checks, and the unchanged `v0.2.0` tag target invariant.
## Prevention

For route-body extraction near larger behavior clusters, map neighboring non-targets explicitly and prefer tiny descriptive modules over broad route buckets. Keep `use super::*;` for mechanical moves, then run targeted tests, rustfmt, clippy, and dependency/tag boundary checks before continuing.
