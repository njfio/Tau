---
title: Release execution parameter approval matrix
category: patterns
date: '2026-04-26'
tags:
  - release
  - approval
  - semver
  - product-proof
  - guardrails
related:
  - .gyre/product-proof-release-execution-parameters.md
  - docs/planning/product-proof-release-checklist.md
---

# Release execution parameter approval matrix
## Problem
A release execution request can hide several independent decisions: target version, metadata bump, changelog promotion, release commit, tag creation, push, and publication.
## Root cause
These actions are often listed together in a release checklist, but only some are low-risk local edits while others are irreversible or externally visible once pushed or published.
## Solution
Before executing a release, write a parameter sheet that records the current version, recommended target version, approval state, and an action matrix where version bump, changelog promotion, commit, tag, push, and GitHub release are approved independently. Verify metadata and product-proof checks while keeping Cargo files unchanged until the user approves execution.
## Prevention

Treat release execution as a parameterized stage. Do not infer approval for tag, push, or publication from approval of a local metadata bump.
