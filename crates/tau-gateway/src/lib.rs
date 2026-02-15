//! Gateway contracts and HTTP/WebSocket runtime surface for Tau.
//!
//! Exposes gateway contract replay, OpenResponses-compatible endpoints,
//! service-mode lifecycle helpers, and remote profile planning utilities.

/// Public `mod` `gateway_contract` in `tau-gateway`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod gateway_contract;
/// Public `mod` `gateway_openresponses` in `tau-gateway`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod gateway_openresponses;
/// Public `mod` `gateway_runtime` in `tau-gateway`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod gateway_runtime;
/// Public `mod` `gateway_ws_protocol` in `tau-gateway`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod gateway_ws_protocol;
/// Public `mod` `remote_profile` in `tau-gateway`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod remote_profile;

pub use gateway_contract::*;
pub use gateway_openresponses::*;
pub use gateway_runtime::*;
pub use gateway_ws_protocol::*;
pub use remote_profile::*;
