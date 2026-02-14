# Memory Architecture Boundary

Run all commands from repository root.

## Goal

Define a strict boundary between:

- deterministic contract replay/runtime behavior
- production live-memory persistence used by agent runtime recall

## Crate responsibilities

### `tau-memory`

- Owns fixture schema/contracts (`memory_contract.rs`)
- Owns deterministic runtime replay for contract validation (`memory_runtime.rs`)
- Re-exports the live backend abstraction for consumers that need both surfaces

### `tau-memory-backend`

- Owns shared live backend abstraction (`LiveMemoryBackend`)
- Owns default JSONL backend implementation (`JsonlLiveMemoryBackend`)
- Owns workspace normalization and persisted message read/append lifecycle
- Contains no fixture replay/runtime orchestration logic

### `tau-agent-core`

- Owns runtime retrieval/scoring/recall injection behavior
- Consumes `LiveMemoryBackend` for persisted cross-session memory
- Does not own backend file-format/storage implementation details

## Public boundary contract

Live runtime consumers should depend on:

- `tau_memory_backend::LiveMemoryBackend`
- `tau_memory_backend::JsonlLiveMemoryBackend`
- `tau_memory_backend::LiveMemoryMessage`
- `tau_memory_backend::LiveMemoryRole`
- `tau_memory_backend::normalize_workspace_id`

Fixture replay consumers should depend on:

- `tau_memory::memory_contract::*`
- `tau_memory::memory_runtime::*`

## Migration notes

1. Replace direct JSONL helper logic in runtime crates with `LiveMemoryBackend`.
2. Keep fixture replay behavior in `tau-memory`; avoid importing replay driver logic into `tau-agent-core`.
3. Preserve on-disk compatibility for existing backend files (`live-backend/<workspace>.jsonl`).
4. Add/maintain boundary tests in both crates:
   - `tau-memory-backend`: storage/normalization/cap behavior
   - `tau-agent-core`: integration with persisted retrieval and fallback behavior

