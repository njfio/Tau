---
title: Opt-in product-proof live surface smoke
category: patterns
date: '2026-04-26'
tags:
  - product-proof
  - webchat
  - smoke-test
  - shell
  - json-report
  - runtime-proof
related:
  - scripts/dev/prove-tau-product.sh
  - scripts/dev/test-prove-tau-product.sh
  - docs/guides/canonical-product-proof.md
  - README.md
  - docs/solutions/runbooks/canonical-tau-product-proof-command-path.md
  - .gyre/tau-product-proof-webchat-readiness-plan.md
---

# Opt-in product-proof live surface smoke
## Problem
The canonical product proof verified runtime startup, gateway status, TUI live-shell, cleanup, and report output, but it did not exercise an actual browser-facing product surface unless a reviewer manually fetched it.
## Root cause
Making every live smoke hit `/webchat` would increase default runtime assumptions and failure surface. At the same time, leaving webchat checks manual makes local product proof weaker and less machine-readable for reviewers who want higher confidence.
## Solution
Add an opt-in `--webchat-smoke` flag to `scripts/dev/prove-tau-product.sh --run`. The flag fetches `/webchat`, validates stable markers (`Tau Gateway Webchat`, `Dashboard`, `dashboardStatus`), adds `webchat_url` to run reports, and inserts `webchat` into `completed_steps` between `gateway_status` and `tui`. The default `--run` sequence remains unchanged. Deterministic fake-runner/fake-curl tests now cover default success, opt-in webchat success, missing-marker failure, webchat fetch failure, invalid gateway JSON, gateway curl failure, and launcher status failure. Docs were updated in the canonical guide, README, and runbook.
## Prevention

When strengthening a local product proof, add heavier live-surface checks behind explicit opt-in flags, cover the flag with fake transport tests, and make report output distinguish default and opt-in proof sequences.
