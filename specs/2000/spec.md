# Issue 2000 Spec

Status: Implemented

Issue: `#2000`  
Type: Governance  
Scope: Milestone index contract remediation

## Problem Statement

Active implementation milestones `m11`, `m12`, `m13`, `m14`, `m15`, `m16`, and
`m19` are missing required milestone spec index files under
`specs/milestones/<id>/index.md`. This violates repository contract requirements
for implementation governance and blocks compliant execution.

## Scope

In scope:

- add missing milestone index files for `m11`, `m12`, `m13`, `m14`, `m15`,
  `m16`, and `m19`
- include scope, active issue references, and contract notes in each index file
- patch GitHub milestone descriptions so each references its corresponding
  `specs/milestones/<id>/index.md` path

Out of scope:

- implementation work under those milestones
- label taxonomy or issue hierarchy redesign
- milestone reprioritization

## Acceptance Criteria

AC-1 (index file presence):
Given required active milestones (`11`, `12`, `13`, `14`, `15`, `16`, `19`),
when inspecting `specs/milestones/`,
then each has `specs/milestones/m<id>/index.md`.

AC-2 (index content contract):
Given each new index file,
when reviewed,
then it includes milestone identity, scope summary, active issue references, and
contract note requiring per-issue `spec/plan/tasks` artifacts.

AC-3 (GitHub milestone description linkage):
Given GitHub milestones (`11`, `12`, `13`, `14`, `15`, `16`, `19`),
when reading milestone descriptions,
then each contains `Spec index: specs/milestones/m<id>/index.md`.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given repository tree, when listing milestone spec paths, then all required index files exist. |
| C-02 | AC-2 | Conformance | Given each index file, when scanning sections, then scope, active issue list, and contract note are present. |
| C-03 | AC-3 | Integration | Given GitHub milestone metadata, when queried via API, then each description includes its spec-index path. |

## Success Metrics

- all required milestone index files exist in repository
- GitHub milestone descriptions link to in-repo index paths
- governance blocker issue `#2000` can be closed
