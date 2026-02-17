# Plan #2320

Status: Reviewed
Spec: specs/2320/spec.md

## Approach

1. Gather claim-to-evidence mappings for items 5-15 using existing targeted tests and contract scripts.
2. Add `scripts/dev/verify-gap-claims-wave2.sh` with strict shell behavior and deterministic command sequence.
3. Update `tasks/resolution-roadmap.md` with a dated wave-2 table that records status per claim and references the evidence commands.
4. Run the new script end-to-end and capture GREEN evidence.

## Affected Modules

- `scripts/dev/verify-gap-claims-wave2.sh`
- `tasks/resolution-roadmap.md`
- `specs/milestones/m51/index.md`
- `specs/2320/spec.md`
- `specs/2320/plan.md`
- `specs/2320/tasks.md`

## Risks and Mitigations

- Risk: a mapped command may be environment-conditional (for example postgres DSN).
  - Mitigation: classify as `Partial` where live dependencies are optional and document requirement in evidence notes.
- Risk: script drift if test names change.
  - Mitigation: keep command list explicit and maintain alongside tests.

## Interfaces / Contracts

- Verification script contract: non-zero exit on first failing command.
- Roadmap contract: each claim row includes status and command-level evidence.
