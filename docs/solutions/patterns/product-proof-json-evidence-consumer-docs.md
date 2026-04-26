---
title: Product-proof JSON evidence consumer docs
category: patterns
date: '2026-04-26'
tags:
  - product-proof
  - json-report
  - docs
  - ci
  - review
  - runbook
related:
  - docs/guides/canonical-product-proof.md
  - README.md
  - docs/solutions/runbooks/canonical-tau-product-proof-command-path.md
  - scripts/dev/prove-tau-product.sh
  - .gyre/tau-product-proof-report-consumer-docs-plan.md
---

# Product-proof JSON evidence consumer docs
## Problem
A product-proof script can emit machine-readable JSON evidence, but reviewers and CI users still need copy-pasteable parsing guidance to know which fields prove the intended path actually ran.
## Root cause
Without consumer documentation, `--report` output is discoverable only by reading the shell script or guessing the JSON shape. That makes evidence easy to produce but hard to consume consistently across review, CI, and release handoff contexts.
## Solution
Document report consumption at every likely entry point: the canonical product-proof guide now includes `## Consuming Report JSON` with Python parser examples for `--check` and `--run`, the README names `--report` commands in the unified runtime lifecycle path, and the runbook tells operators to assert `mode`, `status`, `checks`, `gateway_status_url`, and `completed_steps`. Verification generated a real `--check --report` file, parsed it with Python, reran `scripts/dev/prove-tau-product.sh --check`, ran `cargo fmt --check`, and confirmed root Cargo files stayed unchanged.
## Prevention

When adding report output to a proof script, document the consumer contract immediately: include field meanings, one local-review parser, one CI-style assertion shape, and a warning not to treat partial output from a failed command as success.
