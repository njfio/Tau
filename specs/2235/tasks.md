# Tasks #2235

Status: Implemented
Spec: specs/2235/spec.md
Plan: specs/2235/plan.md

- T1 (RED): Add failing conformance/unit tests for Codex routing, Responses
  parser mapping, and chat fallback behavior.
- T2 (GREEN): Implement Responses endpoint selection/fallback and response
  parsing in `OpenAiClient`.
- T3 (REFACTOR): Keep helper functions isolated and preserve current
  chat-completions stream behavior for non-Codex models.
- T4 (VERIFY): Run `cargo test -p tau-ai` and targeted provider checks.
- T5 (DOC/TRACE): Update issue status logs and include AC-to-test mapping in PR.
