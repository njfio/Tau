# M322 - Full legacy mini-model reference purge

Status: Active

## Context
M321 moved runtime defaults to `openai/gpt-5.2`, but repository-wide legacy
mini-model references remain in scripts,
fixtures, docs, and tests. Those leftovers create policy drift and confuse
operators about supported defaults.

## Issue Hierarchy
- Epic: #3545
- Story: #3546
- Task: #3547

## Scope
- Remove active guidance/default/test references to legacy mini-model aliases.
- Replace with GPT-5 family references (`openai/gpt-5.2` baseline; keep
  codex-specific `openai/gpt-5.3-codex` where required by provider mode).
- Add a conformance guard that fails CI if legacy model references are
  reintroduced.

## Exit Criteria
- `specs/3547/spec.md` is `Implemented` with AC verification evidence.
- Repo guard validates legacy mini-model aliases do not remain in tracked
  sources.
- Focused tests and verification scripts pass for touched modules.
