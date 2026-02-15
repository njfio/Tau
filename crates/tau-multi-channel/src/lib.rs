//! Multi-channel transport runtime building blocks for Tau.
//!
//! Provides connector, routing, lifecycle, ingress, outbound, policy, and
//! telemetry components for Telegram/Discord/WhatsApp-style channels.
//!
//! Architecture reference:
//! - [`docs/guides/multi-channel-event-pipeline.md`](../../../docs/guides/multi-channel-event-pipeline.md)
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use tau_multi_channel::parse_multi_channel_live_inbound_envelope;
//!
//! let raw = r#"{
//!   "schema_version": 1,
//!   "transport": "telegram",
//!   "provider": "telegram-bot-api",
//!   "payload": {
//!     "update_id": 42,
//!     "message": {
//!       "message_id": 7,
//!       "date": 1700000000,
//!       "text": "hello",
//!       "chat": { "id": "chat-1", "type": "private" },
//!       "from": { "id": "user-1", "username": "operator" }
//!     }
//!   }
//! }"#;
//!
//! let event = parse_multi_channel_live_inbound_envelope(raw)?;
//! assert_eq!(event.transport.as_str(), "telegram");
//! assert_eq!(event.event_id, "7");
//! # Ok(())
//! # }
//! ```

/// Public `mod` `multi_channel_contract` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_contract;
/// Public `mod` `multi_channel_credentials` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_credentials;
/// Public `mod` `multi_channel_incident` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_incident;
/// Public `mod` `multi_channel_lifecycle` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_lifecycle;
/// Public `mod` `multi_channel_live_connectors` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_live_connectors;
/// Public `mod` `multi_channel_live_ingress` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_live_ingress;
/// Public `mod` `multi_channel_media` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_media;
/// Public `mod` `multi_channel_outbound` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_outbound;
/// Public `mod` `multi_channel_policy` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_policy;
/// Public `mod` `multi_channel_route_inspect` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_route_inspect;
/// Public `mod` `multi_channel_routing` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_routing;
/// Public `mod` `multi_channel_runtime` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_runtime;
/// Public `mod` `multi_channel_send` in `tau-multi-channel`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub mod multi_channel_send;

pub use multi_channel_contract::*;
pub use multi_channel_credentials::*;
pub use multi_channel_incident::*;
pub use multi_channel_lifecycle::*;
pub use multi_channel_live_connectors::*;
pub use multi_channel_live_ingress::*;
pub use multi_channel_media::*;
pub use multi_channel_outbound::*;
pub use multi_channel_policy::*;
pub use multi_channel_route_inspect::*;
pub use multi_channel_routing::*;
pub use multi_channel_runtime::*;
pub use multi_channel_send::*;
