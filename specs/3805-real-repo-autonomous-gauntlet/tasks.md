# Tasks: Tau-Internal Autonomous Coding Gauntlet

- [x] T1: Add the conformance script before relying on documentation claims.
- [x] T2: Seed a temporary Tau worktree with docs-only, single-file, multi-file,
  failing-test, CLI flag, flaky, malformed-provider, and crash/stale-lease
  fixtures.
- [x] T3: Drive successful cases through `issue-to-merge` with bounded provider
  repair authority and PR-ready assertions.
- [x] T4: Assert malformed provider output blocks with rejected repair evidence.
- [x] T5: Assert stale lease status exposes recoverable operator classification
  and a recovery command.
- [x] T6: Emit JSONL case results and a final JSON report with suite totals.
- [x] T7: Document the gauntlet in the README evidence table and autonomous
  coding jobs guide.
- [x] T8: Run the gauntlet and static validation before commit.

## Test Tiers

| Tier | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Unit | N/A | Shell benchmark only | Runtime unit coverage exists in earlier specs. |
| Property | N/A | Not parser/algorithm work | No randomized invariant added. |
| Contract/DbC | N/A | Existing CLI/runtime contracts | This script validates the public CLI contract. |
| Snapshot | N/A | JSON report is dynamic temp evidence | No stable snapshot needed. |
| Functional | Done | `scripts/dev/test-real-repo-autonomous-coding-gauntlet.sh` | Covers eight product cases. |
| Conformance | Done | C-01..C-08 in `spec.md` | All cases passed in the gauntlet. |
| Integration | Done | Temporary Tau worktree + durable job state | Exercises CLI, runtime, git, provider repair, and operator status. |
| Fuzz | N/A | Not an untrusted parser change | Malformed provider case covers fail-closed behavior. |
| Mutation | N/A | Benchmark/docs slice | No critical algorithm changed. |
| Regression | Done | Malformed provider and stale lease cases | Prevents two prior overclaim gaps. |
| Performance | N/A | Not performance-sensitive | No performance contract changed. |
