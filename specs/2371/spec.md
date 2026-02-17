# Issue 2371 Spec — G5 Typed Memory + Importance Foundation

Status: Accepted

## Problem Statement

Tau memory entries currently do not carry a first-class memory type taxonomy or
importance score. `tasks/spacebot-comparison.md` gap `G5` requires typed
memories with default importance and search ranking influence so higher-value
memories are favored.

## Scope

In scope:

- Add a typed memory enum for `identity`, `goal`, `decision`, `todo`,
  `preference`, `fact`, `event`, `observation`.
- Add per-type default importance values in the `0.3..=1.0` range.
- Extend `memory_write` arguments with optional `memory_type` and `importance`.
- Persist `memory_type` and `importance` in runtime memory records.
- Return metadata in memory write/read/search outputs.
- Apply importance boost during search ranking.

Out of scope:

- Graph relations (`G6`).
- Lifecycle decay/pruning (`G7`).
- Ingestion pipelines (`G9`).

## Acceptance Criteria

### AC-1: Memory Type Defaults

Given `memory_write` is called without explicit `importance`,
When a valid `memory_type` is provided,
Then the stored record uses the default importance for that type.

### AC-2: Importance Override Validation

Given `memory_write` is called with `importance`,
When `importance` is outside `0.0..=1.0` or non-numeric,
Then the tool returns a validation error and does not write a record.

### AC-3: Metadata Persistence and Output

Given memory records are written,
When `memory_read` and `memory_search` return records,
Then responses include `memory_type` and `importance` from persisted data.

### AC-4: Importance-Aware Ranking

Given two otherwise similar memory candidates with different importance values,
When `memory_search` ranks results,
Then higher-importance records receive higher final ranking scores.

### AC-5: Backward Compatibility

Given legacy records without typed-memory fields and writes without new
arguments,
When read/search paths execute,
Then defaults are applied without parse failures or behavior regressions.

## Conformance Cases

| Case | Maps To | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance/Functional | `memory_write` with `memory_type=identity` and no `importance` | Write entry | Stored/output importance equals identity default and type is `identity` |
| C-02 | AC-2 | Conformance/Unit | `memory_write` with invalid `importance` (e.g. `1.5`) | Execute tool | Returns `memory_invalid_arguments` and no write occurs |
| C-03 | AC-3 | Conformance/Integration | Entries written with and without explicit type/importance | Read and search entries | Both outputs include `memory_type` and `importance` |
| C-04 | AC-4 | Conformance/Integration | Two records with same semantics, different importance values | Run search query | Higher-importance record ranks first |
| C-05 | AC-5 | Regression | Legacy/default write path (no new args) | Write + read + search | Defaults (`observation` + default importance) are used and no errors occur |

## Success Metrics / Observable Signals

- New conformance tests `spec_c01`..`spec_c05` pass.
- Existing memory tool/runtime tests remain green in touched crates.
- No serde regression for existing memory record artifacts.

## AC → Conformance → Test Mapping

- AC-1 → C-01 → `spec_c01_memory_write_applies_type_default_importance`
- AC-2 → C-02 → `spec_c02_memory_write_rejects_invalid_importance_override`
- AC-3 → C-03 → `spec_c03_memory_read_and_search_include_type_and_importance`
- AC-4 → C-04 → `spec_c04_memory_search_boosts_higher_importance_records`
- AC-5 → C-05 → `spec_c05_default_paths_remain_backward_compatible`
