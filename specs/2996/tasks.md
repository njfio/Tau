# Tasks: Issue #2996 - panic/unsafe audit script with production-path guardrail evidence

1. [x] T1 (RED): add fixture-based conformance test script proving expected panic/unsafe classification output before implementation.
2. [x] T2 (GREEN): implement `scripts/dev/audit-panic-unsafe.sh` with deterministic reporting and path-class split.
3. [x] T3 (REGRESSION): run fixture conformance and repo audit script; verify expected counts and stable output.
4. [x] T4 (VERIFY): run production-target guardrails:
   - `cargo clippy --workspace --all-features -- -A warnings -D clippy::panic`
   - `cargo clippy --workspace --all-features -- -A warnings -D unsafe_code`
5. [x] T5 (CONFORMANCE): map and confirm C-01..C-05 evidence.

## Tier Mapping
- Unit: fixture conformance script assertions.
- Property: N/A (no randomized algorithm introduced).
- Contract/DbC: N/A (no contract macro surface changes).
- Snapshot: N/A (deterministic textual audit output asserted directly).
- Functional: audit script behavior over fixture and repo scan.
- Conformance: C-01..C-05.
- Integration: script + workspace tooling (`rg`, `cargo clippy`) in repo context.
- Fuzz: N/A (no parser exposed to untrusted runtime input).
- Mutation: N/A (process tooling slice; no runtime control-flow changes).
- Regression: rerun conformance and guardrail checks after implementation.
- Performance: N/A (non-hotpath developer tooling).
