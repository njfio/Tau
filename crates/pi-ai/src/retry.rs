use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const BASE_BACKOFF_MS: u64 = 200;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);
static JITTER_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn should_retry_status(status: u16) -> bool {
    status == 408 || status == 409 || status == 425 || status == 429 || status >= 500
}

pub fn next_backoff_ms(attempt: usize) -> u64 {
    let shift = attempt.min(6);
    BASE_BACKOFF_MS.saturating_mul(1_u64 << shift)
}

pub fn next_backoff_ms_with_jitter(attempt: usize, jitter_enabled: bool) -> u64 {
    let base = next_backoff_ms(attempt);
    if !jitter_enabled || base <= 1 {
        return base;
    }

    // Bounded jitter in [50%, 100%] of the deterministic backoff.
    let low = base / 2;
    let width = base.saturating_sub(low);
    let seed = JITTER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mixed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17) ^ 0xA24B_AED4_963E_E407;
    let jitter = if width == 0 {
        0
    } else {
        mixed % width.saturating_add(1)
    };
    low.saturating_add(jitter)
}

pub fn retry_budget_allows_delay(elapsed_ms: u64, delay_ms: u64, retry_budget_ms: u64) -> bool {
    if retry_budget_ms == 0 {
        return true;
    }
    elapsed_ms.saturating_add(delay_ms) <= retry_budget_ms
}

pub fn is_retryable_http_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() || error.is_body()
}

pub fn new_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let count = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("pi-rs-{millis}-{count}")
}

#[cfg(test)]
mod tests {
    use super::{
        new_request_id, next_backoff_ms, next_backoff_ms_with_jitter, retry_budget_allows_delay,
        should_retry_status,
    };

    #[test]
    fn retry_status_selection_is_correct() {
        assert!(should_retry_status(429));
        assert!(should_retry_status(503));
        assert!(!should_retry_status(400));
        assert!(!should_retry_status(404));
    }

    #[test]
    fn backoff_increases_per_attempt() {
        assert_eq!(next_backoff_ms(0), 200);
        assert_eq!(next_backoff_ms(1), 400);
        assert_eq!(next_backoff_ms(2), 800);
    }

    #[test]
    fn jittered_backoff_stays_within_expected_bounds() {
        let attempt = 3;
        let base = next_backoff_ms(attempt);
        let low = base / 2;
        for _ in 0..64 {
            let value = next_backoff_ms_with_jitter(attempt, true);
            assert!(value >= low, "expected {value} >= {low}");
            assert!(value <= base, "expected {value} <= {base}");
        }
    }

    #[test]
    fn retry_budget_math_respects_zero_and_bounded_budgets() {
        assert!(retry_budget_allows_delay(50, 100, 0));
        assert!(retry_budget_allows_delay(50, 50, 100));
        assert!(!retry_budget_allows_delay(50, 60, 100));
    }

    #[test]
    fn request_ids_are_unique() {
        let a = new_request_id();
        let b = new_request_id();
        assert_ne!(a, b);
        assert!(a.starts_with("pi-rs-"));
    }
}
