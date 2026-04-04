# Tasks: Issue #3662 - Bound no-tool gateway retry attempts so TUI does not time out first

- [x] T1 (RED): add gateway regression coverage for bounded retry timeout on a
      later no-tool action retry attempt and blocked mission persistence.
- [x] T2 (GREEN): add bounded later-action-retry timeout logic in the
      OpenResponses execution handler.
- [x] T3 (VERIFY): run scoped `tau-gateway` retry/timeout regression coverage,
      including the existing successful retry path.

## Tier Mapping
- Regression: bounded later retry timeout and blocked mission persistence
- Functional: existing successful no-tool-then-tool retry path remains green
