# Tasks #2332

Status: Completed
Spec: specs/2332/spec.md
Plan: specs/2332/plan.md

- T1 (tests first): capture RED evidence that wave-2 verifier does not call the
  live Postgres verifier script.
- T2: update `scripts/dev/verify-gap-claims-wave2.sh` claim #6 mapping.
- T3: run `scripts/dev/verify-gap-claims-wave2.sh` and capture GREEN evidence.
- T4: run `cargo fmt --check` and prepare PR.
