# Tasks: Issue #3633 - Repair broken relative links in March 24 planning docs

- [x] T1 Red: run `python3 .github/scripts/docs_link_check.py --repo-root .`
      and capture the two current March 24 plan link failures. Covers C-03.
- [x] T2 Create milestone/spec artifacts for `#3633` and self-accept the P2
      docs-only scope. Covers AC-1 through AC-3.
- [x] T3 Update the first March 24 plan so ideation links resolve correctly
      from `docs/plans/`. Covers C-01 / AC-1.
- [x] T4 Update the second March 24 plan so plan-to-plan links resolve
      correctly from `docs/plans/`. Covers C-02 / AC-2.
- [x] T5 Green: rerun `python3 .github/scripts/docs_link_check.py --repo-root .`
      and confirm `issues=0`. Covers C-03 / AC-3.
- [ ] T6 Update issue/PR state, commit, push, and open a dedicated PR. Covers
      delivery/closure.
