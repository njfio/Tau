# AGENTS.md

## Repository Workflow Policy

This repository uses an issue-first workflow. All implementation work must be tracked in GitHub Issues.

### 1) Issue-first execution (mandatory)
- Before any code change, create or identify a GitHub issue that defines scope and acceptance criteria.
- If work is broad, create a parent epic issue and linked child issues.
- Keep issue status current through progress comments and scope updates.

### 2) Branching and commits
- Create branches using: `codex/issue-<id>-<short-topic>`.
- Keep commits atomic and logically scoped.
- Reference the issue ID in commit messages where practical.

### 3) Pull requests
- Every PR must link its issue (for example: `Closes #<id>`).
- PR description must include:
  - Summary of behavior changes
  - Risks and compatibility notes
  - Validation evidence (`cargo fmt`, strict `clippy`, tests)
- Add a PR comment with explicit test-matrix evidence.

### 4) Test quality bar (mandatory)
For each completed issue, include or update tests across these categories as applicable:
- Unit
- Functional
- Integration
- Regression

### 5) Merge and closure
- Merge only after CI passes and issue acceptance criteria are met.
- After merge, update/close the issue with final outcome and follow-up items.

### 6) Roadmap tracking
- Use milestone-driven planning for multi-wave work.
- Keep roadmap issues precise: problem, scope, acceptance criteria, and required test matrix.
