# Plan: Issue #3766 - Refresh roadmap status snapshot after proof-loop landing

## Approach

Use the repository's generated status updater rather than editing roadmap text
by hand. The previous failing check shows only the status snapshot date changed,
so the intended implementation is the smallest generated refresh that makes the
check deterministic again.

## Affected Modules

- `tasks/todo.md`
- `tasks/tau-vs-ironclaw-gap-list.md`
- `specs/3766-roadmap-status-proof-loop-refresh/*`

## Risks and Mitigations

- Risk: Generated roadmap refresh could include unrelated issue state changes.
  Mitigation: Review the diff before staging and keep only generated status
  output.
- Risk: This cleanup could distract from the Agent Canvas proof-loop result.
  Mitigation: Do not change proof-loop code; rerun its focused regression only
  as a guard.

## Verification

Run the generated updater, then verify `scripts/dev/roadmap-status-sync.sh
--check --quiet`, `scripts/dev/test-ops-chat-canvas-proof.sh`, and `git diff
--check`.
