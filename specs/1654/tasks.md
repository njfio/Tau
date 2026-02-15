# Issue 1654 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): run baseline marker threshold verification and hotspot scan to
capture pre-change failing/deficit evidence.

T2: add rustdoc comments for undocumented public items in wave-2 scope crates.

T3: regenerate hotspot after-artifacts and marker/threshold/ratchet reports.

T4: run crate-level regression tests for touched crates.

T5: prepare PR evidence mapping ACs to commands/artifacts.

## Tier Mapping

- Functional: undocumented-public coverage in scope crates
- Conformance: baseline/after hotspot + threshold artifacts
- Integration: marker scripts and report generation flow
- Regression: crate tests for touched crates
