# Tasks: Issue #3656 - Wire gateway mission loop into Tau action history and learning insights

- [x] T1 (RED): add unit coverage for gateway learning-insight distillation from
      seeded action history.
- [x] T2 (RED): add regression coverage proving a gateway tool failure is
      persisted to the gateway-local action-history store.
- [x] T3 (RED): add regression coverage proving a follow-up request includes
      learned `## Learning Insights` prompt context from prior action history.
- [x] T4 (GREEN): add gateway learning-runtime helpers for action-history path,
      load/save, and `LearningInsight` distillation.
- [x] T5 (GREEN): capture tool outcomes in the execution handler, persist them
      into action history, and append learned guidance to the system prompt.
- [x] T6 (VERIFY): run targeted `tau-gateway` learning, retry, and
      session-roundtrip verification.

## Tier Mapping
- Unit: learning-insight distillation helper behavior
- Regression: action-history persistence and learned prompt injection
- Functional: follow-up gateway request sees learned guidance
- Integration: existing gateway retry/session persistence remains green
