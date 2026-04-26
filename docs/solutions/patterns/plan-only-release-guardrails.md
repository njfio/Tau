---
title: Plan-only release guardrails
category: patterns
date: '2026-04-26'
tags:
  - release
  - semver
  - guardrails
  - product-proof
  - cargo
related:
  - .gyre/product-proof-actual-release-plan.md
  - docs/planning/product-proof-release-checklist.md
  - CHANGELOG.md
---

# Plan-only release guardrails
## Problem
Release preparation can easily drift into destructive actions such as version bumps, tags, pushes, or published releases before the user has explicitly approved the target version and publication boundary.
## Root cause
The repository has centralized workspace versioning in Cargo.toml and a remote/tag surface, so release prep and release execution look adjacent in the command sequence even though they carry very different risk.
## Solution
Use a plan-only stage that writes an internal release map and a human checklist, verifies metadata/product-proof/fmt/root-drift gates, and explicitly records no version bump, no tag, no push, and no GitHub release publication until a later approved execution stage.
## Prevention

Keep release planning and release execution as separate Gyre stages. Before installing an execution backlog, ask the user for explicit approval of target version, commit, tag, push, and publication behavior.
