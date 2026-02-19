# Plan: Trusted Execution Environment (TEE) Integration

## Overview

Add a `tau-tee` crate that provides a provider-agnostic TEE abstraction layer for
hardware-backed isolation, remote attestation, and sealed storage. The design
follows Tau's existing security architecture (trust roots, signed envelopes, RBAC)
and extends it with hardware-rooted trust.

---

## Architecture

### Where TEE Fits in Tau's Security Stack

```
┌─────────────────────────────────────────────────────────┐
│                    tau-agent-core                        │
│                   (turn loop)                            │
└──────────────┬──────────────────────────────────────────┘
               │
       ┌───────┴────────┐
       │                 │
  tau-access         tau-tee  ◄── NEW CRATE
  (RBAC, signed      (TEE provider trait,
   envelopes,         attestation, sealed
   trust roots)       storage, policy)
       │                 │
       └───────┬─────────┘
               │
         tau-tools / tau-runtime
         (tool policy enforcement,
          WASM sandbox, OS sandbox)
```

`tau-tee` sits alongside `tau-access` as a peer security primitive. It:
- **Consumes** trust roots from `tau-access` for attestation verification
- **Provides** sealed storage that `tau-tools` and `tau-runtime` can use for secrets
- **Exposes** attestation documents that transport bridges can exchange for
  agent-to-agent trust establishment

### Core Trait: `TeeProvider`

```rust
#[async_trait]
pub trait TeeProvider: Send + Sync {
    /// Initialize the TEE environment (load keys, verify platform).
    async fn initialize(&self) -> Result<TeeCapabilities>;

    /// Generate an attestation document proving code integrity.
    async fn attest(&self, user_data: &[u8]) -> Result<AttestationDocument>;

    /// Verify a remote attestation document against trust roots.
    fn verify_attestation(
        &self,
        document: &AttestationDocument,
        trust_roots: &[TrustedRootRecord],
        now_unix_ms: u64,
    ) -> AttestationVerdict;

    /// Encrypt data bound to this TEE instance (sealed storage).
    async fn seal(&self, plaintext: &[u8], context: &SealContext) -> Result<SealedBlob>;

    /// Decrypt data previously sealed by this TEE instance.
    async fn unseal(&self, blob: &SealedBlob, context: &SealContext) -> Result<Vec<u8>>;

    /// Report the platform type.
    fn platform(&self) -> TeePlatform;
}
```

### Key Types

```rust
/// Detected TEE platform
pub enum TeePlatform {
    None,               // No TEE available
    Simulated,          // Software simulation (dev/test)
    IntelSgx,           // Intel SGX enclave
    IntelTdx,           // Intel TDX trust domain
    AmdSev,             // AMD SEV-SNP
    AwsNitro,           // AWS Nitro Enclaves
    ArmTrustZone,       // ARM CCA / TrustZone
}

/// Attestation document produced by a TEE
pub struct AttestationDocument {
    pub platform: TeePlatform,
    pub quote: Vec<u8>,              // Platform-specific attestation quote
    pub user_data: Vec<u8>,          // Caller-provided nonce/binding data
    pub timestamp_ms: u64,           // When the attestation was generated
    pub pcrs: BTreeMap<u32, Vec<u8>>,// Platform Configuration Registers
    pub certificates: Vec<Vec<u8>>,  // Certificate chain (DER-encoded)
}

/// Result of verifying an attestation
pub enum AttestationVerdict {
    Trusted {
        reason_code: String,
        platform: TeePlatform,
        matched_root_id: String,
    },
    Untrusted {
        reason_code: String,
    },
}

/// Encrypted blob from sealed storage
pub struct SealedBlob {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,          // AES-256-GCM nonce (12 bytes)
    pub tag: Vec<u8>,            // Authentication tag (16 bytes)
    pub context_hash: Vec<u8>,   // SHA-256 of SealContext for binding
    pub sealed_at_ms: u64,
}

/// Context binding for sealed data (prevents cross-context unseal)
pub struct SealContext {
    pub agent_id: String,
    pub scope: String,           // e.g., "provider-keys", "session-state"
}

/// TEE capabilities reported after initialization
pub struct TeeCapabilities {
    pub platform: TeePlatform,
    pub attestation_available: bool,
    pub sealed_storage_available: bool,
    pub max_enclave_memory_bytes: Option<u64>,
}

/// TEE operational policy
pub struct TeePolicy {
    pub mode: TeeMode,
    pub attestation_freshness_ms: u64,     // Max age of accepted attestation
    pub required_platforms: Vec<TeePlatform>,
    pub seal_provider_keys: bool,          // Auto-seal API keys
    pub require_attestation_for_tools: bool,
}

pub enum TeeMode {
    Disabled,        // No TEE usage
    Opportunistic,   // Use TEE if available, fallback to software
    Required,        // Fail if TEE unavailable
}
```

---

## Implementation Plan

### Phase 1: Core Crate and Simulated Provider

**New crate:** `crates/tau-tee/`

```
crates/tau-tee/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module root, re-exports
    ├── provider.rs         # TeeProvider trait definition
    ├── types.rs            # TeePlatform, AttestationDocument, SealedBlob, etc.
    ├── attestation.rs      # Attestation verification logic (uses tau-access trust roots)
    ├── sealed_storage.rs   # AES-256-GCM seal/unseal with context binding
    ├── policy.rs           # TeePolicy, TeeMode, JSON parsing, validation
    ├── platform.rs         # Runtime platform detection
    └── simulated.rs        # SimulatedProvider implementation (software-only)
```

**Dependencies:**
- `tau-access` (for `TrustedRootRecord`)
- `tau-core` (for `write_text_atomic`)
- `aes-gcm` (new workspace dep for AES-256-GCM encryption)
- `rand` (new workspace dep for nonce generation)
- `sha2`, `serde`, `serde_json`, `anyhow`, `async-trait`, `tracing` (existing workspace deps)

**Steps:**

1. **`types.rs`** — Define all core types (`TeePlatform`, `AttestationDocument`,
   `SealedBlob`, `SealContext`, `TeeCapabilities`, `AttestationVerdict`)
   with `Serialize`/`Deserialize` derives.

2. **`provider.rs`** — Define the `TeeProvider` async trait. Must be object-safe
   (`Box<dyn TeeProvider>`).

3. **`platform.rs`** — Implement `detect_platform() -> TeePlatform` that checks
   for hardware TEE availability at runtime:
   - Check `/dev/sgx_enclave` for Intel SGX
   - Check `/dev/sev` for AMD SEV
   - Check `/sys/devices/platform/nitro_enclaves` for AWS Nitro
   - Check `cpuid` for TDX support
   - Default to `TeePlatform::None`

4. **`sealed_storage.rs`** — Implement AES-256-GCM encryption:
   - `seal(key, plaintext, context) -> SealedBlob`
   - `unseal(key, blob, context) -> Vec<u8>`
   - Context binding via SHA-256 hash of `SealContext` fields used as AAD
   - Random 12-byte nonce per seal operation

5. **`attestation.rs`** — Implement attestation verification:
   - `verify_attestation(doc, trust_roots, now_ms) -> AttestationVerdict`
   - Check document timestamp freshness
   - Match platform-specific certificate chain against trust roots
   - For `Simulated` platform: verify against simulated root key
   - Return structured verdict with reason codes (mirrors `SignedEnvelopeDecision` pattern)

6. **`policy.rs`** — Implement `TeePolicy`:
   - JSON deserialization with defaults
   - Validation (freshness > 0, mode consistency)
   - `evaluate_tee_requirement(capabilities, policy) -> TeeGateResult`

7. **`simulated.rs`** — Implement `SimulatedProvider`:
   - Uses a deterministic Ed25519 key for attestation signing
   - Generates attestation documents with `TeePlatform::Simulated`
   - Uses in-memory AES-256-GCM key derived from a seed
   - Full trait compliance for dev/test without hardware

8. **`lib.rs`** — Module root with re-exports

9. **Add to workspace** — Update root `Cargo.toml` members list

### Phase 2: Integration Points (Future Work)

These are NOT implemented in Phase 1 but represent the designed integration surface:

1. **`tau-access` integration** — Add `TeeAttestedPrincipal` to RBAC that accepts
   attestation documents as identity proof. An agent running in a verified TEE gets
   elevated trust.

2. **`tau-tools` integration** — Add `TeeMode` to `ToolPolicy`. When `Required`,
   tools that handle secrets (HTTP with auth headers, env-reading bash) must
   execute within a TEE context or be denied.

3. **`tau-runtime` integration** — Add TEE-backed WASM execution. When a TEE is
   available, WASM extensions run inside the enclave with sealed memory.

4. **`tau-startup` integration** — During startup pipeline, detect TEE platform and
   initialize the provider. Report capabilities in diagnostics.

5. **`tau-gateway` integration** — Expose `/v1/attestation` endpoint for remote
   attestation exchange. Clients can verify the gateway is running in a genuine TEE
   before sending sensitive prompts.

6. **Hardware provider implementations** — Separate crates (`tau-tee-sgx`,
   `tau-tee-nitro`, etc.) behind the `TeeProvider` trait, gated by Cargo features.

---

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| No hardware TEE in CI/dev environments | High | `SimulatedProvider` enables full test coverage without hardware |
| AES-256-GCM nonce reuse | Critical | Random 12-byte nonce per seal; never reuse keys across contexts |
| Attestation document forgery in simulation mode | Medium | Simulated attestation is clearly tagged; policy can require hardware platforms |
| Platform detection false positives (device files exist but TEE disabled) | Low | Detection returns `None` if initialization fails; `Opportunistic` mode falls back |
| Workspace dependency additions (`aes-gcm`, `rand`) | Low | Well-audited, widely-used crates; `rand` is already transitively pulled |

---

## Dependency Graph

```
tau-tee
├── tau-access (TrustedRootRecord for attestation verification)
├── tau-core (write_text_atomic for policy file persistence)
├── aes-gcm (AES-256-GCM sealed storage) ← new
├── rand (nonce generation) ← new
├── sha2 (context binding hashes) ← existing
├── ed25519-dalek (simulated attestation signing) ← existing
├── base64 (encoding) ← existing
├── serde / serde_json ← existing
├── anyhow ← existing
├── async-trait ← existing
└── tracing ← existing
```

---

## Testing Strategy

| Tier | Scope | Example |
|------|-------|---------|
| Unit | Individual functions (seal, unseal, verify, detect) | `seal()` then `unseal()` roundtrip |
| Functional | Full provider lifecycle | Initialize → attest → verify → seal → unseal |
| Conformance | All C-01 through C-10 from spec | Verify each acceptance criterion |
| Property | Sealed storage never decrypts with wrong context | Randomized context/key combinations |
| Regression | Tampered ciphertext detected | Flip a byte in `SealedBlob.ciphertext` |

---

## File Changes Summary

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Add `crates/tau-tee` to members, add `aes-gcm` and `rand` to workspace deps |
| `crates/tau-tee/Cargo.toml` | New crate manifest |
| `crates/tau-tee/src/lib.rs` | Module root |
| `crates/tau-tee/src/types.rs` | Core types |
| `crates/tau-tee/src/provider.rs` | `TeeProvider` trait |
| `crates/tau-tee/src/platform.rs` | Platform detection |
| `crates/tau-tee/src/attestation.rs` | Attestation verification |
| `crates/tau-tee/src/sealed_storage.rs` | AES-256-GCM seal/unseal |
| `crates/tau-tee/src/policy.rs` | Policy types and validation |
| `crates/tau-tee/src/simulated.rs` | Software simulation provider |
| `specs/tee-integration/spec.md` | Specification (already created) |
| `specs/tee-integration/plan.md` | This plan |
