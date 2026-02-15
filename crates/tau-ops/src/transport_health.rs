//! Transport health facade for ops/report surfaces.
//!
//! This module re-exports canonical transport-health types/helpers from
//! `tau-runtime` so operator commands depend on one shared health contract.

pub use tau_runtime::transport_health::*;
