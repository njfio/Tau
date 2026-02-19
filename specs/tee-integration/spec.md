# Trusted Execution Environment (TEE) Integration

**Status:** Reviewed
**Priority:** P1
**Area:** backend, security

## Problem Statement

Tau agents execute sensitive operations—API key management, tool execution, prompt
processing—in environments where the host OS and hypervisor are implicitly trusted.
This trust model is insufficient for regulated workloads, multi-tenant deployments,
and federated agent-to-agent communication where one party must prove it has not
tampered with the runtime.

A Trusted Execution Environment (TEE) integration provides hardware-backed isolation,
remote attestation, and sealed storage so that:

1. Secrets (provider API keys, signing keys) remain encrypted outside TEE boundaries.
2. Sensitive tool outputs are processed inside an attested enclave.
3. Remote agents and gateways can verify the integrity of a Tau instance before
   sharing data.

## Scope

### In Scope

- `tau-tee` crate with provider-agnostic TEE abstraction layer
- `TeeProvider` trait with async lifecycle (initialize → attest → seal/unseal → execute)
- Attestation document types with quote verification against trust roots
- Sealed storage API for encrypting/decrypting secrets at rest
- Secure execution context for isolating sensitive operations
- Simulated (software-only) provider for development and testing
- TEE policy configuration (mode, required attestation freshness, allowed platforms)
- Integration surface with `tau-access` trust roots for attestation verification
- Platform detection for runtime TEE availability

### Out of Scope

- Actual hardware enclave SDKs (Intel SGX SDK, AWS Nitro Enclaves SDK) — those are
  future provider implementations behind the trait
- Kernel-level attestation drivers
- Key management service (KMS) integration
- UI/dashboard changes

## Acceptance Criteria

- **AC-1:** `TeeProvider` trait compiles and is object-safe (`dyn TeeProvider`)
- **AC-2:** `SimulatedProvider` passes all trait methods with deterministic behavior
- **AC-3:** Attestation documents serialize/deserialize and verify against trust roots
- **AC-4:** Sealed storage encrypts with AES-256-GCM and decrypts roundtrip correctly
- **AC-5:** Platform detection correctly identifies `Simulated` when no hardware is present
- **AC-6:** TEE policy configuration parses from JSON and validates constraints
- **AC-7:** All public APIs have comprehensive test coverage (unit + functional)
- **AC-8:** Crate compiles with `cargo check`, passes `cargo clippy`, and `cargo test`

## Conformance Cases

| ID   | AC  | Input                                                        | Expected Output                                           |
|------|-----|--------------------------------------------------------------|-----------------------------------------------------------|
| C-01 | AC-2 | `SimulatedProvider::initialize()` then `attest()`           | Returns valid `AttestationDocument` with simulated quote  |
| C-02 | AC-3 | Verify attestation with matching trust root                  | `AttestationVerdict::Trusted`                             |
| C-03 | AC-3 | Verify attestation with unknown trust root                   | `AttestationVerdict::Untrusted` with reason code          |
| C-04 | AC-3 | Verify attestation with expired document                     | `AttestationVerdict::Untrusted` with `attestation_expired`|
| C-05 | AC-4 | `seal("secret", context)` then `unseal(sealed, context)`    | Returns original `"secret"` bytes                         |
| C-06 | AC-4 | `unseal()` with tampered ciphertext                          | Returns error                                             |
| C-07 | AC-5 | `detect_platform()` on non-TEE host                         | `TeePlatform::None`                                       |
| C-08 | AC-6 | Parse valid TEE policy JSON                                  | `TeePolicy` with correct fields                           |
| C-09 | AC-6 | Parse TEE policy with invalid attestation freshness          | Validation error                                          |
| C-10 | AC-1 | Create `Box<dyn TeeProvider>` from `SimulatedProvider`       | Compiles and dispatches correctly                         |

## Success Metrics

- Zero `unsafe` blocks in `tau-tee` (delegates to well-audited crypto crates)
- All 10 conformance cases pass
- `cargo clippy -- -D warnings` clean
