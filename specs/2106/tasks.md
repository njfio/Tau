# Tasks #2106

Status: Implemented
Spec: specs/2106/spec.md
Plan: specs/2106/plan.md

## Ordered Tasks

- T1 (RED): add regression script assertions first and capture failing evidence
  before rustdoc additions.
- T2 (GREEN): add `///` docs to scoped first-wave files.
- T3 (VERIFY): run
  `bash scripts/dev/test-split-module-rustdoc.sh`,
  compile checks for `tau-github-issues`, `tau-ai`, `tau-runtime`,
  and targeted tests listed in plan.
- T4 (CLOSE): set `specs/2106/*` status Implemented and close subtask issue.
