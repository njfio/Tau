# Tasks: Issue #3733 - Credential store KDF hardening + legacy cipher migration

- [x] T1 (RED): add regression coverage for v3-vs-v2 KDF divergence and v2
      backward-compatible decryption.
- [x] T2 (RED): add regression coverage for v1 auto-migration on load and the
      v3 minimum key length requirement.
- [x] T3 (GREEN): add the crate-local Argon2 dependency and v3 keyed envelope
      helpers in `credential_store.rs`.
- [x] T4 (GREEN): keep legacy v1/v2 decrypt compatibility and auto-migrate
      legacy keyed payloads during store load.
- [x] T5 (VERIFY): run targeted `tau-provider` credential-store tests plus
      `cargo check -p tau-provider` and `cargo fmt --check`.
