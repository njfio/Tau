# Tasks: Issue #3440 - Planning docs status reconciliation

1. [x] T1 (RED, Functional): capture stale/future-only wording in both planning docs.
2. [x] T2 (GREEN, Docs): update integration-gap plan language to reflect implemented baseline + remaining expansion.
3. [x] T3 (GREEN, Docs): update true-RL roadmap boundary language to reflect integrated baseline and hardening path.
4. [x] T4 (VERIFY, Regression): run docs checks and roadmap sync check.
5. [x] T5 (VERIFY): set spec status to `Implemented` and record evidence.

## RED / GREEN Evidence
### RED
- `rg -n "planned/future|not yet|staged primitives|gap" docs/planning/integration-gap-closure-plan.md docs/planning/true-rl-roadmap-skeleton.md`

### GREEN
- `python3 .github/scripts/architecture_docs_check.py --repo-root .`
- `python3 .github/scripts/runbook_ownership_docs_check.py --repo-root .`
- `scripts/dev/test-roadmap-status-sync.sh`
- `scripts/dev/roadmap-status-sync.sh --check --quiet`
