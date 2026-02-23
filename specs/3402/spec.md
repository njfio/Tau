# Spec: Issue #3402 - Close remaining C5/K13/R8 conformance gaps

Status: Implemented

## Problem Statement
After closing F10, `specs/3386/conformance-matrix.md` still leaves `13` scenarios as `N/A`: `C5-01/02/03/04/07/08`, `K13-06`, and `R8-01/02/03/04/05/08`. The repository already contains substantial deterministic coverage in multi-channel runtime, auth/provider rotation flows, and live RL reward/training paths, but these scenarios are not fully mapped and two scenario assertions need explicit focused tests.

## Scope
In scope:
- Map C5 scenarios to deterministic multi-channel runtime tests:
  - `C5-01` Telegram inbound processing.
  - `C5-02` Discord inbound processing.
  - `C5-03` valid signed inbound envelope accepted.
  - `C5-04` invalid signature rejected fail-closed.
  - `C5-07` multi-channel routing to correct sessions.
  - `C5-08` media attachment handling.
- Map `K13-06` credential rotation lifecycle to deterministic auth/provider command coverage.
- Map R8 scenarios to deterministic live RL + reward inference tests:
  - `R8-01` rollout created after completed session.
  - `R8-02` reward breakdown dimensions/signals deterministic.
  - `R8-03` session completion signal behavior asserted.
  - `R8-04` tool errors reduce reliability score.
  - `R8-05` token-efficiency behavior asserted.
  - `R8-08` optimizer update fires on configured rollout interval.
- Add focused deterministic tests where scenario claims are not yet explicit.
- Update conformance mapping so all listed scenarios are no longer `N/A`.

Out of scope:
- New gateway public route surfaces.
- Live external network/provider integration.

## Acceptance Criteria
### AC-1 C5 multi-channel inbound/security/routing/media scenarios are executable and traceable
Given deterministic multi-channel runtime fixtures and routing state,
when inbound events are processed,
then Telegram/Discord ingress, signature validation/rejection, routing, and attachment handling are asserted by executable tests.

### AC-2 K13-06 credential rotation lifecycle is executable and traceable
Given keyed credential-store state,
when rotate-key flow executes,
then new credential material is accepted and old-key access fails closed.

### AC-3 R8 rollout/reward/optimizer scenarios are executable and traceable
Given deterministic live RL/reward inference inputs,
when completed sessions and optimizer cadence execute,
then rollout persistence, reward breakdown signals (including reliability/tool-error and token-efficiency), and optimizer interval behavior are asserted.

### AC-4 Conformance traceability is fully updated
Given issue-local and milestone conformance artifacts,
when reviewed,
then `C5-01/02/03/04/07/08`, `K13-06`, and `R8-01/02/03/04/05/08` are mapped to executable tests instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | Telegram and Discord inbound fixtures | live runner processes ingress | both transports are ingested and persisted to channel/session stores |
| C-02 | AC-1 | Integration | secure messaging mode required with trusted signer | signed inbound message arrives | valid signature accepted and event allowed |
| C-03 | AC-1 | Regression | secure messaging mode required with tampered payload | forged signature arrives | event is denied with fail-closed reason |
| C-04 | AC-1 | Integration | multi-transport events and route bindings | events process in one cycle | transport/session routing remains isolated and deterministic |
| C-05 | AC-1 | Integration | inbound event with mixed attachments | runtime processes media understanding | attachment outcomes and context enrichment are asserted |
| C-06 | AC-2 | Functional/Regression | keyed credential store with provider/integration entries | rotate-key command runs | new key decrypts and old key fails closed |
| C-07 | AC-3 | Functional | completed live RL event sequence | bridge persists rollout/span | rollout exists with reward metadata |
| C-08 | AC-3 | Unit/Regression | reward inference with/without tool errors and completion state | inference executes | breakdown signals and penalties are deterministic |
| C-09 | AC-3 | Unit/Conformance | deterministic input/output ratios | inference executes | token-efficiency behavior matches expected ordering |
| C-10 | AC-3 | Functional | update interval rollouts configured to trigger | completed run reaches interval | optimizer report indicates executed update |
| C-11 | AC-4 | Conformance | rows for C5/K13/R8 in matrix | conformance docs updated | all listed rows marked `Covered` |

## Success Metrics / Observable Signals
- No remaining `N/A` rows for `C5-01/02/03/04/07/08`, `K13-06`, `R8-01/02/03/04/05/08`.
- Deterministic test selectors for each mapped scenario pass in touched crates.
- `specs/3386/conformance-matrix.md` and `specs/3402/conformance-matrix.md` are consistent.
