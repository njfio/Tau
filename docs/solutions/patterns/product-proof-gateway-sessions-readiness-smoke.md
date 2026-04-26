---
title: Product-proof Gateway sessions readiness smoke
category: patterns
date: '2026-04-26'
tags:
  - product-proof
  - tau-gateway
  - sessions-api
  - shell-contract-tests
  - reports
---

# Product-proof Gateway sessions readiness smoke
## Problem
The product-proof live run covered gateway status, optional webchat, live-shell, and cleanup, but it did not expose an operator-visible proof that the Gateway sessions API surface is reachable and returning the expected JSON shape.
## Root cause
The Gateway sessions API route behavior was covered by Rust tests, but product-proof evidence is consumed through shell scripts and JSON reports. Without a dedicated opt-in smoke, reviewers could not verify sessions readiness through the canonical proof path.
## Solution
Add `--sessions-smoke` to `scripts/dev/prove-tau-product.sh` as an opt-in `--run` extension. The live path fetches `/gateway/sessions`, validates that the response is a JSON object with a `sessions` array, records `gateway_sessions_url`, and inserts `sessions_api` into `completed_steps`. The deterministic fake-runner/fake-curl harness covers success, invalid response shape, curl failure, and combined webchat plus sessions ordering.
## Prevention

Keep product-proof surface expansions opt-in, report-backed, and fake-harness-covered. New endpoint smokes should preserve the default `--run` sequence, validate only stable response contracts, add report fields with explicit step names, and avoid dependency-manifest drift.
