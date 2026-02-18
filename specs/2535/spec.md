# Spec #2535 - Story: profile-based process/task model routing for G15

Status: Reviewed

## Problem Statement
Runtime prompt dispatch currently uses a single baseline model (except plan-first role overrides), so process-level routing and task/complexity-based model selection are not available.

## Acceptance Criteria
### AC-1 profile routing schema
Given profile defaults are loaded, when routing fields are configured, then process-level model fields and task overrides are parsed and available to runtime dispatch.

### AC-2 deterministic routing policy
Given a prompt and process type, when routing policy is evaluated, then complexity (`light|standard|heavy`) and task overrides (`coding|summarization`) deterministically select an effective model.

### AC-3 scoped dispatch behavior
Given an effective routed model exists, when a prompt runs, then dispatch uses the routed model for that run and restores baseline afterward.

## Scope
In scope:
- `tau-onboarding` profile defaults schema updates.
- `tau-coding-agent` runtime dispatch policy evaluation + scoped override wiring.

Out of scope:
- New process runtime architecture.
- Provider/catalog changes.

## Conformance Cases
- C-01 (AC-1): `spec_2536_c01_profile_defaults_parse_routing_fields`
- C-02 (AC-2): `spec_2536_c02_prompt_complexity_and_task_override_select_model`
- C-03 (AC-3): `spec_2536_c03_dispatch_uses_scoped_model_override_and_restores_baseline`
- C-04 (AC-3): `regression_2536_default_profile_without_routing_keeps_baseline_model`

## Success Metrics
- C-01..C-04 pass.
- No behavior regression for baseline model dispatch when routing config is absent.
