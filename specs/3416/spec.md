# Spec: Issue #3416 - Refresh README with verified feature inventory and user-first onboarding

Status: Implemented

## Problem Statement
`README.md` is feature-dense but difficult for first-time users to scan quickly. The repository needs a clearer top-level guide that is informative, user-friendly, and aligned with implemented code/doc surfaces.
The current draft still reads like a component inventory and does not make integrated end-to-end paths, maturity boundaries, and gap-closure plans explicit enough.

## Scope
In scope:
- Review workspace features at crate/docs level and reflect current capabilities in `README.md`.
- Improve onboarding flow: prerequisites, quickstart, common tasks, and where-to-go-next links.
- Validate that README command examples and doc links resolve to existing files/scripts.

Out of scope:
- Runtime behavior changes.
- New feature implementation or API changes.
- Full documentation rewrite beyond `README.md`.

## Acceptance Criteria
### AC-1 README is structured for fast onboarding
Given a new user at repository root,  
when they open `README.md`,  
then they can identify what Tau is, who it is for, prerequisites, and a minimal first-run path within the first sections.

### AC-2 README reflects implemented capabilities
Given current crates/docs in workspace,  
when feature summaries are read in `README.md`,  
then claims are bounded to implemented surfaces and do not advertise removed/unimplemented contract runners as live product features.

### AC-3 README navigation links and commands are valid
Given referenced file links and command snippets in `README.md`,  
when validated from repo root,  
then linked docs/scripts exist and primary command paths resolve.

### AC-4 README presents integrated operating paths and maturity/gap transparency
Given a user evaluating whether Tau is truly integrated,  
when they read `README.md`,  
then they can identify:
- concrete integrated paths that work now end-to-end,
- a maturity matrix that distinguishes production-ready vs staged/partial areas,
- explicit gap-closure links for True RL, dashboard maturity, auth verification, and TUI improvements.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | existing README | headings are inspected | clear onboarding sections exist (`Who`, `Quickstart`, `Common Workflows`) |
| C-02 | AC-2 | Functional/Conformance | crate/docs inventory | README capability text is cross-checked | feature claims map to real crates/docs |
| C-03 | AC-3 | Conformance | README links/scripts | path checks are executed | all referenced docs/scripts exist |
| C-04 | AC-3 | Conformance | README command examples | command target checks are executed | command entrypoints (`cargo run -p ...`, scripts) are present |
| C-05 | AC-4 | Functional/Conformance | existing README | section headings are inspected | `Integrated End-to-End Paths`, `Maturity Matrix`, and `Current Gaps and Execution Plan` sections exist |
| C-06 | AC-4 | Conformance | README gap rows | link checks are executed | each gap row includes a valid repository path link for follow-through |

## Success Metrics / Observable Signals
- `README.md` is rewritten with user-first flow and concise capability map.
- Validation checks for linked docs/scripts pass.
- README contains no references to missing files in this repository.
- README includes integrated-path-first narrative before crate inventory details.
- README includes explicit maturity table and gap-closure plan links.
