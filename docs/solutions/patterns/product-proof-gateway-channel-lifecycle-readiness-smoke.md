---
title: Product-proof Gateway channel lifecycle readiness smoke
category: patterns
date: '2026-04-27'
tags:
  - product-proof
  - tau-gateway
  - channel-lifecycle
  - scripts
  - operator-readiness
---

# Product-proof Gateway channel lifecycle readiness smoke
## Problem
The canonical Tau product proof covered gateway status, optional webchat, sessions, and base memory readiness, but did not exercise the Gateway channel lifecycle status surface even after its route tests were extracted and verified.
## Root cause
Product-proof endpoint coverage intentionally expands through opt-in live smokes so the default run stays short. The channel lifecycle route requires a POST action body, so adding it safely needed fake-curl coverage for method/body tolerant invocation, report step ordering, and malformed/curl failure behavior.
## Solution
Add --channel-lifecycle-smoke to scripts/dev/prove-tau-product.sh for --run only. The live smoke POSTs {"action":"status"} to /gateway/channels/discord/lifecycle, validates a JSON object with report.action == status and report.channel == discord, records gateway_channel_lifecycle_url, and appends channel_lifecycle_api to completed_steps before tui/down. Extend scripts/dev/test-prove-tau-product.sh with channel-lifecycle-success, channel-lifecycle-invalid-json, and channel-lifecycle-curl-failure cases, plus all-smokes report ordering. Document the flag, report field, and manual curl check in docs/guides/canonical-product-proof.md.
## Prevention

Keep new product-proof endpoint coverage opt-in unless it is already part of the shortest proof path. For POST-backed endpoint smokes, use a non-mutating status action, validate the response shape, add fake-curl success and failure cases, and require report fields to be machine-checkable.
