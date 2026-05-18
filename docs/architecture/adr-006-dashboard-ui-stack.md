# ADR-006: Dashboard UI Architecture and Stack Selection

## Context
Tau's dashboard capabilities are now delivered through gateway-served web surfaces:

- `/webchat` for operator workflows.
- `/ops/*` and `/dashboard` gateway-served operator surfaces.
- `tau-dashboard-ui`, a Leptos SSR crate that renders deterministic operator
  shell markup consumed by gateway routes and tests.

The remaining stack question is no longer "separate frontend repo vs gateway"
or "React/Vite vs server-rendered shell". The repo now contains substantial
Leptos SSR route coverage, while some gateway-owned HTML/JS surfaces still
exist for compatibility and incremental migration.

The unresolved decision is whether to keep expanding ad hoc embedded HTML/JS or
standardize new operator UI work on the Leptos SSR shell.

Without a formal update, old React/Vite guidance conflicts with the current
crate graph and contributor expectations.

## Decision
1. **Architecture location:** keep dashboard UI implementation consolidated with
   gateway in the main Tau repository and served by `tau-gateway`, not a
   separate frontend repository.
2. **Selected stack:** standardize new operator dashboard route work on
   **Leptos SSR in `tau-dashboard-ui`**.
3. **Compatibility posture:** maintain existing gateway-owned HTML/JS shell
   surfaces only as compatibility and bridging layers. Do not add new large
   ad hoc HTML/JS route implementations when a Leptos SSR route can carry the
   same operator contract.
4. **Rejected direction:** do not pursue the prior React + TypeScript + Vite
   direction for this dashboard stack without a new ADR that justifies the
   extra runtime/toolchain split.

## Consequences
### Positive
- Single deployment boundary: gateway runtime owns API + UI delivery.
- Rust-native SSR contracts keep operator markup close to gateway state types.
- Existing `tau-dashboard-ui` tests remain the canonical route-marker proof.
- Incremental migration keeps existing operator workflows stable while reducing
  ad hoc HTML/JS growth.

### Negative
- Leptos compile cost is significant; targeted tests should be preferred during
  iteration.
- Rich client-side interactions may still require careful JS islands or a later
  ADR if SSR becomes the wrong boundary.

### Neutral / Follow-on
- Current embedded shell routes remain valid compatibility baselines.
- Route migrations should preserve existing endpoint contracts and deterministic
  SSR markers before removing old HTML/JS sections.
