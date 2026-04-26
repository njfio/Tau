---
title: Contract-test live shell scripts with injectable command seams
category: patterns
date: '2026-04-26'
tags:
  - bash
  - shell-contract-test
  - product-proof
  - tau-unified
  - strict-mode
  - testability
  - failure-modes
related:
  - docs/solutions/runbooks/canonical-tau-product-proof-command-path.md
  - scripts/dev/prove-tau-product.sh
  - scripts/dev/test-prove-tau-product.sh
  - .gyre/tau-product-proof-failure-modes-plan.md
---

# Contract-test live shell scripts with injectable command seams
## Problem
A live shell proof path can be too expensive or side-effecting to run in routine --check mode, leaving command sequencing, readable failure diagnostics, and cleanup behavior untested until a real operator run fails.
## Root cause
The product-proof --check path originally validated static guide markers and launcher syntax but did not exercise the --run command sequence. Early contract runs exposed two shell-script reliability gaps: an EXIT trap referenced function-local cleanup variables after the function returned under set -u, and jq validation failed without emitting the same readable invalid-object error as the Python fallback.
## Solution
Add narrow injectable command seams for live-only external effects, then test the live sequence with fakes. In this stage, scripts/dev/prove-tau-product.sh uses TAU_PRODUCT_PROOF_CURL_BIN for the /gateway/status fetch and configurable status retry knobs for fast deterministic failure tests. scripts/dev/test-prove-tau-product.sh uses existing tau-unified fake runner hooks plus fake curl to assert success, invalid-json, curl-failure, and status-failure cases without starting the real runtime. The default --check path runs the contract test. Cleanup state now lives at script scope for trap visibility, successful runs clear EXIT/INT/TERM traps before returning, and jq validation emits `gateway/status response is not a JSON object` consistently.
## Prevention

Live-mode shell scripts should have deterministic contract tests for their --run sequencing path, not only static --check validation. External commands used by live scripts should be injectable so tests can assert command order, arguments, and failure handling without side effects. Bash cleanup traps under set -u should avoid referencing function-local variables after return; use trap-visible cleanup state or clear traps before returning. Validator branches should emit consistent diagnostics across tool implementations such as jq and Python fallbacks. Contract tests should include at least one success case, one bad-output case, one transport failure, and one post-start launcher failure that proves teardown runs.
