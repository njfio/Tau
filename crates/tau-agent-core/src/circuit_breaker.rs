//! Circuit breaker for LLM provider protection.
//!
//! Prevents cascading failures by tracking consecutive errors and temporarily
//! rejecting requests when a failure threshold is breached.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests pass through.
    Closed = 0,
    /// Failing — reject requests immediately.
    Open = 1,
    /// Testing recovery — allow one request through.
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(value: u8) -> Self {
        match value {
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Lock-free circuit breaker for LLM provider calls.
pub struct CircuitBreaker {
    state: AtomicU8,
    consecutive_failures: AtomicU32,
    last_failure_time_ms: AtomicU64,
    failure_threshold: u32,
    recovery_timeout_ms: u64,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the given thresholds.
    pub fn new(failure_threshold: u32, recovery_timeout_ms: u64) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            consecutive_failures: AtomicU32::new(0),
            last_failure_time_ms: AtomicU64::new(0),
            failure_threshold,
            recovery_timeout_ms,
        }
    }

    /// Returns the current circuit state.
    pub fn state(&self) -> CircuitState {
        let raw = self.state.load(Ordering::SeqCst);
        let state = CircuitState::from(raw);

        // Auto-transition from Open → HalfOpen after recovery timeout
        if state == CircuitState::Open {
            let last_failure = self.last_failure_time_ms.load(Ordering::SeqCst);
            let now = current_time_ms();
            if now.saturating_sub(last_failure) >= self.recovery_timeout_ms {
                self.state
                    .compare_exchange(
                        CircuitState::Open as u8,
                        CircuitState::HalfOpen as u8,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .ok();
                return CircuitState::HalfOpen;
            }
        }

        state
    }

    /// Returns true if a request should be allowed through.
    pub fn should_allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true, // Allow probe request
            CircuitState::Open => false,
        }
    }

    /// Record a successful request — reset failure count.
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        self.state
            .store(CircuitState::Closed as u8, Ordering::SeqCst);
    }

    /// Record a failed request — potentially trip the circuit.
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure_time_ms
            .store(current_time_ms(), Ordering::SeqCst);

        if failures >= self.failure_threshold {
            self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
        }
    }

    /// Returns the current consecutive failure count.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::new(3, 30_000);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.should_allow_request());
    }

    #[test]
    fn trips_after_threshold() {
        let cb = CircuitBreaker::new(3, 30_000);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.should_allow_request());
    }

    #[test]
    fn resets_on_success() {
        let cb = CircuitBreaker::new(3, 30_000);
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.consecutive_failures(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_after_recovery_timeout() {
        let cb = CircuitBreaker::new(2, 0); // 0ms recovery = immediate
        cb.record_failure();
        cb.record_failure();
        // With 0ms recovery timeout, state() immediately transitions Open → HalfOpen
        let state = cb.state();
        assert!(
            state == CircuitState::Open || state == CircuitState::HalfOpen,
            "expected Open or HalfOpen, got {:?}",
            state
        );
        // Second call should definitely be HalfOpen
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.should_allow_request());
    }

    #[test]
    fn half_open_success_closes_circuit() {
        let cb = CircuitBreaker::new(2, 0);
        cb.record_failure();
        cb.record_failure();
        // Transition to HalfOpen
        let _ = cb.state();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
