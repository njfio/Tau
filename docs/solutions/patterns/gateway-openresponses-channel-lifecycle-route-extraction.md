---
title: Gateway OpenResponses channel lifecycle route extraction
category: patterns
date: '2026-04-27'
tags:
  - tau-gateway
  - gateway-openresponses
  - route-extraction
  - channel-lifecycle
  - tests
---

# Gateway OpenResponses channel lifecycle route extraction
## Problem
The Gateway OpenResponses parent test module still carried direct channel lifecycle endpoint coverage for logout/status success and invalid-channel/action/unauthorized refusal, keeping another route family mixed with adjacent config and broader Gateway API tests.
## Root cause
The channel lifecycle tests sit in the same large parent module as several Gateway API endpoint families. Without an explicit boundary map, the success path could be separated from its refusal regression or accidentally batched with config, safety, ops-channel marker, multi-channel status, or product-proof work.
## Solution
Map the route family first, then move only integration_spec_2670_c01_channel_lifecycle_endpoint_supports_logout_and_status_contract and regression_spec_2670_c04_channel_lifecycle_endpoint_rejects_invalid_channel_action_and_auth into crates/tau-gateway/src/gateway_openresponses/tests/gateway_channel_lifecycle_api.rs with use super::* and a parent mod gateway_channel_lifecycle_api declaration. Preserve bearer auth, lifecycle report assertions, persisted state-file assertion, gateway status discovery assertion, invalid channel/action errors, unauthorized status, and handle.abort calls. Verify with cargo test -p tau-gateway gateway_openresponses --lib, cargo fmt --check, cargo clippy -p tau-gateway --tests --no-deps -- -D warnings, and git diff --quiet -- Cargo.toml Cargo.lock.
## Prevention

For direct Gateway API route extractions, keep success and refusal tests together when they exercise the same endpoint. List adjacent non-target endpoint families before moving code, especially when similarly named Ops route-marker tests or multi-channel status tests share vocabulary with the route under extraction.
