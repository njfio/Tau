//! Utility helpers extracted from gateway_openresponses root module.

#[cfg(test)]
use anyhow::{Context, Result};
#[cfg(test)]
use std::net::SocketAddr;

#[cfg(test)]
pub(super) fn validate_gateway_openresponses_bind(bind: &str) -> Result<SocketAddr> {
    bind.parse::<SocketAddr>()
        .with_context(|| format!("invalid gateway socket address '{bind}'"))
}
