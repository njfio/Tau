# M68 - Spacebot G23 Fly.io Deployment Baseline

Milestone objective: deliver a first-party Fly.io deployment baseline for Tau gateway mode with executable conformance checks and operator documentation.

## Scope
- Add repository-root `fly.toml` with gateway-mode defaults.
- Add deployment validation logic/tests to guard Fly manifest drift.
- Update deployment runbook with Fly launch/deploy/verify flow.

## Out of Scope
- Hosted SaaS rollout automation.
- Multi-region traffic management policies.
- Provider/runtime behavior changes.

## Exit Criteria
- Task `#2410` ACs implemented and verified.
- `fly.toml` exists with required gateway deployment contract fields.
- Deployment guide documents repeatable Fly launch/deploy/health commands.
- Scoped quality gates pass (`fmt`, `clippy`, `tau-deployment` tests).
