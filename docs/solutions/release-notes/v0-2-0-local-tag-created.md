---
title: v0.2.0 local tag created
category: release-notes
date: '2026-04-26'
tags:
  - release-notes
  - v0.2.0
  - tag
  - product-proof
  - local-only
related:
  - Cargo.toml
  - CHANGELOG.md
  - Cargo.lock
  - scripts/dev/prove-tau-product.sh
---

# v0.2.0 local tag created
## Problem
The v0.2.0 product-proof release had local commits and needed a local tag without crossing into push or GitHub release publication.
## Root cause
Release execution was intentionally decomposed into local metadata, local commit, Gateway lockfile synchronization, and local tag boundaries so each higher-risk action required explicit approval.
## Solution
Created annotated local tag `v0.2.0` with message `Release v0.2.0 product-proof readiness` at verified HEAD `373857ba`, which includes the product-proof release metadata commit and the Cargo.lock metadata sync. Post-tag checks passed: product-proof `--check`, locked cargo metadata, formatting, clean Cargo.lock, and tag-to-HEAD verification.
## Prevention

Keep push and GitHub release publication as separate explicit stages. Before pushing tags, verify `git rev-parse v0.2.0^{}` matches the intended commit and that local validation still passes.
