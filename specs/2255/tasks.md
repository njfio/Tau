# Tasks #2255

Status: Done
Spec: specs/2255/spec.md
Plan: specs/2255/plan.md

- T1 (tests first): add failing conformance tests for token limit derivation and
  pre-flight rejection behavior (C-01, C-02, C-03).
- T2: add shared model-aware pre-flight token limit derivation helper.
- T3: wire derived limits into local runtime startup and training executor.
- T4: apply derived limits in onboarding agent builder settings/config mapping.
- T5: run scoped fmt/clippy/tests and verify conformance mappings.
