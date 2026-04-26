---
title: Product-proof Gateway memory readiness smoke
category: patterns
date: '2026-04-26'
tags:
  - product-proof
  - gateway
  - memory
  - smoke
  - report-json
---

# Product-proof Gateway memory readiness smoke
## Problem
The canonical product-proof script already proved runtime status, webchat readiness, and sessions readiness, but it did not provide opt-in live evidence that the read-only Gateway memory API surface was available.
## Root cause
The product-proof harness grows endpoint coverage one explicit opt-in at a time, and the memory endpoint had route-level Rust tests but no report-backed live smoke in the canonical proof path.
## Solution
Add `--memory-smoke` to `scripts/dev/prove-tau-product.sh`. The smoke fetches `/gateway/memory/default`, validates a JSON object with an `exists` boolean, records `gateway_memory_url`, and inserts `memory_api` into `completed_steps`. Extend `scripts/dev/test-prove-tau-product.sh` with fake-curl success, all-smokes success, invalid JSON, curl failure, and report-shape assertions. Document the operator command and report consumer checks in `docs/guides/canonical-product-proof.md`. Gateway memory readiness smoke stays read-only and avoids the policy-gated write path.
## Prevention

Keep future endpoint product-proof smokes opt-in, read-only unless a policy gate is explicitly part of the requirement, backed by fake-runner/fake-curl contract tests, represented in report JSON, and documented in the canonical product-proof guide.
