# Spec: Issue #3719 - Surface unresolved-placeholder Enter blocking in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview currently says `Enter runs typed: ...`
even when the typed command still contains unresolved scaffold placeholders such
as `resume <mission-id>`. Actual submit-time behavior already blocks those
commands, so the preview guidance is misleading precisely when the operator
needs the most help.

## Scope
In scope:
- preview-only Enter guidance for typed command-palette input with unresolved placeholders
- preserving normal Enter guidance for typed commands with resolved arguments
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3719/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- submit-time placeholder guardrail changes
- selected-command Enter preview changes
- new commands or scaffold changes

## Acceptance Criteria
### AC-1 Typed unresolved placeholders surface blocking guidance
Given the command-palette input is treated as an explicit typed command and still
contains unresolved placeholders,
when Tau renders the Enter preview line,
then it says Enter is blocked and names the placeholder(s) that must be replaced.

### AC-2 Supported resolved typed commands keep normal Enter guidance
Given the command-palette input is treated as an explicit typed command, still
has palette context, and no unresolved placeholders remain,
when Tau renders the Enter preview line,
then it still says Enter runs the typed command.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette Enter-preview behavior,
when placeholder-blocking guidance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `resume <mission-id>` in the command palette shows
  `Enter blocked: replace <mission-id>` in the preview. Maps to AC-1.
  Tier: Functional.
- C-02 Typing `search <query>` in the command palette shows
  `Enter blocked: replace <query>` in the preview. Maps to AC-1.
  Tier: Functional.
- C-03 Typing `copy-last assistant` in the command palette still shows
  `Enter runs typed: copy-last assistant`. Maps to AC-2. Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after
  blocking guidance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell before pressing Enter when a typed scaffold cannot run yet
- Preview guidance matches the existing submit-time placeholder guardrail
- Resolved typed commands keep their existing execution-preview wording

## Key Decisions
- Limit the slice to typed-command preview guidance; do not change actual submit behavior
- Reuse the existing unresolved-placeholder detection instead of introducing a second ruleset
- Keep the preview string concise and placeholder-specific
