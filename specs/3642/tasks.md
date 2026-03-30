# Tasks: Issue #3642 - Stabilize tau-coding-agent package-scoped tests exposed by fast-validate

- [ ] T1 Red: reproduce the focused failing `tau-coding-agent` runtime/startup
      tests exposed by package-scoped validation.
- [ ] T2 Green: replace the blocking live-RL snapshot path used during local
      runtime startup. Covers C-01.
- [ ] T3 Green: harden the cwd-mutating tests so the original directory is
      restored on failure paths. Covers C-02.
- [ ] T4 Verify: rerun the focused failing tests and then the exact
      `fast-validate` reproduction. Covers C-03 and C-04.
