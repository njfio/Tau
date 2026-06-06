# Tasks: Issue #3768 - Runtime reality gate for agentic-runtime claims

Status: Implemented

- [x] T1 (SPEC): create conflict-safe spec/plan/tasks for the runtime reality gate.
- [x] T2 (RED): add shell regression requiring JSON/markdown claim-boundary evidence.
- [x] T3 (GREEN): implement `runtime-reality-gate.sh` with fast checks, opt-in markers, unsupported-claim enforcement, and hotspot counts.
- [x] T4 (DOCS): update README proof guidance to point to the reality gate.
- [x] T5 (VERIFY): run new/existing fast proof gates and static checks.

## Tier Mapping

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A | | shell gate; no Rust unit boundary changed |
| Property | N/A | | no randomized invariant introduced |
| Contract/DbC | N/A | | no public API contract macro changed |
| Snapshot | N/A | | JSON/markdown assertions cover generated evidence |
| Functional | ✅ | `scripts/dev/test-runtime-reality-gate.sh` | |
| Conformance | ✅ | default runtime reality gate plus fast proof checks | |
| Integration | N/A | | live provider/browser/full-release checks are opt-in in this slice |
| Fuzz | N/A | | no parser/codec fuzz boundary changed |
| Mutation | N/A | | shell/product-claim gate |
| Regression | ✅ | unsupported claim and failing fast-check paths in shell regression | |
| Performance | N/A | | no runtime performance path changed |

## Verification Evidence

- RED: `bash scripts/dev/test-runtime-reality-gate.sh` failed with
  `missing executable runtime reality gate`, proving the gate was absent.
- GREEN: `scripts/dev/test-runtime-reality-gate.sh` passed with fake fast
  checks, failing-fast-check coverage, and unsupported-overclaim coverage.
- CONFORMANCE: `scripts/dev/runtime-reality-gate.sh --output-json
  /tmp/tau-runtime-reality-current.json --output-md
  /tmp/tau-runtime-reality-current.md` passed on current `master`.
- CONFORMANCE: `/tmp/tau-runtime-reality-current.json` recorded all four fast
  checks as passed and unsupported overstatements as an empty list.
- STATIC: `bash -n scripts/dev/runtime-reality-gate.sh
  scripts/dev/test-runtime-reality-gate.sh` passed.
- STATIC: `scripts/dev/roadmap-status-sync.sh --check --quiet` passed.
- STATIC: `git diff --check` passed.
