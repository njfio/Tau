# Spec: Issue #3626 - Route MCP skill catalog and install flows through tau-skills

Status: Implemented
Priority: P1
Milestone: M329
Parent: #3623

## Problem Statement
The MCP skills handlers currently reimplement skill discovery and install logic
instead of using `tau-skills`. That creates real behavior drift:
directory-backed skills with `SKILL.md` are named/resolved differently, and MCP
installs do not update the skills lockfile metadata that the repo-standard
skills flow maintains.

## Scope
- Route MCP skill list/info behavior through `tau_skills::load_catalog`.
- Route MCP skill install through `tau-skills` install + lockfile helpers.
- Add regression/conformance tests for nested `SKILL.md` resolution and
  lockfile-aware installs.

## Out of Scope
- Remote or registry-backed skill installs via MCP.
- Package-manifest lifecycle operations beyond the existing local skill install
  path.
- Training/orchestration MCP lifecycle work.

## Acceptance Criteria
- AC-1: `tau.skills_list` and `tau.skills_info` resolve directory-backed skills
- consistently with `tau_skills::load_catalog`.
- AC-2: `tau.skills_install` uses the `tau-skills` install flow and writes
  lockfile metadata for installed local skills.
- AC-3: MCP response payloads remain structured and useful while reflecting the
  repo-standard `tau-skills` source of truth.
- AC-4: Scoped `tau-tools` tests prove the parity contract and guard against the
  old drift.

## Conformance Cases
- C-01 (AC-1, regression): a nested `foo/SKILL.md` skill appears as `foo` in
  `tau.skills_list`, and `tau.skills_info` resolves it by `skill_name=foo`.
- C-02 (AC-2, regression): `tau.skills_install` installs a local markdown skill
  via `tau-skills` and writes/updates `skills.lock.json` with the installed
  entry.
- C-03 (AC-3, conformance): MCP list/info/install responses remain structured,
  including install status counts and lockfile path metadata.
- C-04 (AC-4, conformance): scoped `tau-tools` tests stay green with the new
  parity behavior.

## Success Signals
- MCP callers see the same skill naming/resolution behavior as internal
  `tau-skills` consumers.
- Local MCP installs leave lockfile metadata behind instead of silently copying
  files without provenance.
- Regressions around nested skill directories and lockfile drift are covered by
  tests.
