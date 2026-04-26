---
title: Gateway OpenResponses webchat route extraction
category: patterns
date: '2026-04-26'
tags:
  - rust
  - tests
  - tau-gateway
  - openresponses
  - webchat
  - route-tests
  - refactor
related:
  - .gyre/gateway-openresponses-webchat-route-slice.md
  - crates/tau-gateway/src/gateway_openresponses/tests.rs
  - crates/tau-gateway/src/gateway_openresponses/tests/webchat_route.rs
---

# Gateway OpenResponses webchat route extraction
## Problem
After the Ops panel visibility route slice, the Gateway OpenResponses parent test module still contained one standalone functional webchat endpoint shell test between extracted Ops route modules and the broader Gateway API endpoint family.
## Root cause
The webchat endpoint test is route-local, but nearby unit renderer tests, Ops chat route tests, and Gateway JSON API tests are separate ownership bands. Moving more than the single route test would have mixed renderer, chat/session, and API concerns into a tiny route-split stage.
## Solution
Document the target boundary in `.gyre/gateway-openresponses-webchat-route-slice.md`, then move only `functional_webchat_endpoint_returns_html_shell` into `crates/tau-gateway/src/gateway_openresponses/tests/webchat_route.rs` with `use super::*;` and add `mod webchat_route;` to the parent. Preserve request path, endpoint assertions, marker strings, status/content-type checks, and server cleanup. Verification passed with `cargo test -p tau-gateway gateway_openresponses --lib`, `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`, and root Cargo manifest drift checks.
## Prevention

For small route extractions after a larger route split, write a boundary map first and name adjacent non-targets explicitly. Keep unit renderer tests, Ops chat/session tests, and broad JSON API endpoint tests in separate future slices so each move remains reviewable.
