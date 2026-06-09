# Tasks 3803: First-Class OpenRouter Repair Adapter

- [x] T1 - Write conformance tests first:
  - Unit: model resolution, model-id conversion, JSON extraction validation.
  - Functional/conformance: mock OpenRouter endpoint through `issue-to-merge`.
- [x] T2 - Add built-in `openrouter-repair-adapter` subcommand.
- [x] T3 - Add `--provider-repair-openrouter` policy expansion for submit and
  issue-to-merge.
- [x] T4 - Resolve `.env`/env configuration without writing secrets to
  artifacts.
- [x] T5 - Persist provider metadata beside the durable repair context.
- [x] T6 - Default successful job publication to draft PR mode while preserving
  explicit PR-ready and explicit auto-merge behavior.
- [x] T7 - Include the built-in adapter proof in the autonomous coding gauntlet.
- [x] T8 - Run focused Rust tests, clippy, mock-provider script, and gauntlet.

## Test Tier Matrix

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-coding-agent --bin tau_autonomous_coding_job spec_3803` |  |
| Property | N/A |  | No parser invariant beyond existing runtime parser in this slice |
| Contract/DbC | N/A |  | No new contract macro surface |
| Snapshot | N/A |  | No stable snapshot output |
| Functional | ✅ | `scripts/dev/test-openrouter-repair-adapter.sh` |  |
| Conformance | ✅ | C-01..C-05 via unit and shell tests |  |
| Integration | ✅ | `issue-to-merge` mock OpenRouter loop |  |
| Fuzz | N/A |  | Existing strict parser is reused; fuzzing remains parser follow-up work |
| Mutation | N/A |  | Not required for this narrow adapter slice |
| Regression | ✅ | `scripts/dev/test-autonomous-coding-gauntlet.sh` includes built-in adapter case |  |
| Performance | N/A |  | No hot path or performance-sensitive code |
