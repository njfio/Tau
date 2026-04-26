---
title: 'Runbook: Canonical Tau product proof command path'
category: runbooks
date: '2026-04-26'
tags:
  - runbook
  - tau-unified
  - operator-proof
  - gateway
  - tui
  - product-proof
  - json-report
related:
  - docs/guides/canonical-product-proof.md
  - scripts/dev/prove-tau-product.sh
  - scripts/dev/test-prove-tau-product.sh
  - scripts/run/tau-unified.sh
  - scripts/run/test-tau-unified.sh
---

# Runbook: Canonical Tau product proof command path
## Problem
Tau has many runnable scripts and verification gates, so reviewers can struggle to answer which single command path proves the actual operator runtime works locally today and how to capture durable evidence from it.
## Root cause
The project spans CLI runtime, gateway, dashboard, TUI, readiness checks, and milestone scripts. Without a canonical short proof path and machine-readable report option, maintainers may conflate deep GA gates, static launcher tests, live local operator proof, and ad hoc transcript evidence.
## Solution
Use `docs/guides/canonical-product-proof.md` as the short local product proof. For review and CI-style confidence, run `scripts/dev/prove-tau-product.sh --check`; it validates the canonical guide, the `scripts/run/tau-unified.sh` launcher contract, and the fake-runner/fake-curl live-mode contract without starting the real runtime. To capture JSON evidence, run `scripts/dev/prove-tau-product.sh --check --report /tmp/tau-product-proof-check.json` and parse `mode`, `status`, and the `checks` object. For live local proof, run `scripts/dev/prove-tau-product.sh --run`; it executes `./scripts/run/tau-unified.sh up --auth-mode localhost-dev`, `status`, `/gateway/status`, `tui --live-shell --iterations 1 --interval-ms 1000 --no-color`, and `down` with cleanup. To opt into webchat readiness, run `scripts/dev/prove-tau-product.sh --run --webchat-smoke --report /tmp/tau-product-proof-webchat.json`; this checks `/webchat` for stable product-surface markers and adds `webchat_url` plus `webchat` in the `completed_steps` sequence. To capture live JSON evidence without webchat, run `scripts/dev/prove-tau-product.sh --run --report /tmp/tau-product-proof-run.json` and assert `gateway_status_url` plus the `completed_steps` sequence. Expected evidence includes started/running markers, webchat/ops/dashboard URLs, runtime artifact paths, gateway JSON, optional webchat marker validation, live-shell artifact status, clean shutdown, and report fields such as mode, status, gateway_status_url, webchat_url, and completed_steps.
## Prevention

Keep one documented local proof path separate from deeper release readiness gates. When launcher behavior changes, update the guide, `scripts/dev/prove-tau-product.sh`, `scripts/dev/test-prove-tau-product.sh`, and this runbook together so operator proof remains discoverable, executable, and machine-readable.
