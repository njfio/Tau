//! Per-session agent metrics collected during execution.
//!
//! Provides lock-free counters for LLM requests, tool executions, context
//! compaction, and replanning events. Per-tool health statistics are tracked
//! via a `Mutex<HashMap>` for infrequent reads.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Aggregate session-level metrics for an agent run.
#[derive(Debug, Default)]
pub struct AgentMetrics {
    // LLM request metrics
    pub llm_requests_total: AtomicU64,
    pub llm_retries_total: AtomicU64,
    pub llm_failures_total: AtomicU64,
    pub llm_latency_sum_ms: AtomicU64,

    // Tool metrics
    pub tool_executions_total: AtomicU64,
    pub tool_failures_total: AtomicU64,
    pub tool_timeouts_total: AtomicU64,
    pub tool_latency_sum_ms: AtomicU64,

    // Context metrics
    pub compactions_total: AtomicU64,
    pub compaction_messages_dropped: AtomicU64,

    // Replan metrics
    pub replans_total: AtomicU64,
    pub replan_successes: AtomicU64,

    // Per-tool breakdown
    pub tool_health: Mutex<HashMap<String, ToolHealthStats>>,
}

/// Per-tool health statistics aggregated over the session.
#[derive(Debug, Clone, Default)]
pub struct ToolHealthStats {
    pub executions: u64,
    pub failures: u64,
    pub timeouts: u64,
    pub total_latency_ms: u64,
}

impl ToolHealthStats {
    /// Returns the success rate as a value between 0.0 and 1.0.
    pub fn success_rate(&self) -> f64 {
        if self.executions == 0 {
            return 1.0;
        }
        let successes = self.executions.saturating_sub(self.failures);
        successes as f64 / self.executions as f64
    }

    /// Returns the average latency in milliseconds.
    pub fn avg_latency_ms(&self) -> u64 {
        if self.executions == 0 {
            return 0;
        }
        self.total_latency_ms / self.executions
    }
}

/// A read-only snapshot of all agent metrics at a point in time.
#[derive(Debug, Clone)]
pub struct AgentMetricsSnapshot {
    pub llm_requests_total: u64,
    pub llm_retries_total: u64,
    pub llm_failures_total: u64,
    pub llm_latency_sum_ms: u64,
    pub tool_executions_total: u64,
    pub tool_failures_total: u64,
    pub tool_timeouts_total: u64,
    pub tool_latency_sum_ms: u64,
    pub compactions_total: u64,
    pub compaction_messages_dropped: u64,
    pub replans_total: u64,
    pub replan_successes: u64,
    pub tool_health: HashMap<String, ToolHealthStats>,
}

impl AgentMetrics {
    /// Record a completed LLM request.
    pub fn record_llm_request(&self, latency_ms: u64) {
        self.llm_requests_total.fetch_add(1, Ordering::Relaxed);
        self.llm_latency_sum_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    /// Record an LLM retry attempt.
    pub fn record_llm_retry(&self) {
        self.llm_retries_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an LLM failure.
    pub fn record_llm_failure(&self) {
        self.llm_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a tool execution completion.
    pub fn record_tool_execution(&self, tool_name: &str, latency_ms: u64, is_error: bool) {
        self.tool_executions_total.fetch_add(1, Ordering::Relaxed);
        self.tool_latency_sum_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        if is_error {
            self.tool_failures_total.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut map) = self.tool_health.lock() {
            let stats = map.entry(tool_name.to_string()).or_default();
            stats.executions += 1;
            stats.total_latency_ms += latency_ms;
            if is_error {
                stats.failures += 1;
            }
        }
    }

    /// Record a tool timeout.
    pub fn record_tool_timeout(&self, tool_name: &str) {
        self.tool_timeouts_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut map) = self.tool_health.lock() {
            let stats = map.entry(tool_name.to_string()).or_default();
            stats.timeouts += 1;
        }
    }

    /// Record a context compaction event.
    pub fn record_compaction(&self, messages_dropped: u64) {
        self.compactions_total.fetch_add(1, Ordering::Relaxed);
        self.compaction_messages_dropped
            .fetch_add(messages_dropped, Ordering::Relaxed);
    }

    /// Record a replan event.
    pub fn record_replan(&self, success: bool) {
        self.replans_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.replan_successes.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Take an immutable snapshot of all current metrics.
    pub fn snapshot(&self) -> AgentMetricsSnapshot {
        let tool_health = self
            .tool_health
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        AgentMetricsSnapshot {
            llm_requests_total: self.llm_requests_total.load(Ordering::Relaxed),
            llm_retries_total: self.llm_retries_total.load(Ordering::Relaxed),
            llm_failures_total: self.llm_failures_total.load(Ordering::Relaxed),
            llm_latency_sum_ms: self.llm_latency_sum_ms.load(Ordering::Relaxed),
            tool_executions_total: self.tool_executions_total.load(Ordering::Relaxed),
            tool_failures_total: self.tool_failures_total.load(Ordering::Relaxed),
            tool_timeouts_total: self.tool_timeouts_total.load(Ordering::Relaxed),
            tool_latency_sum_ms: self.tool_latency_sum_ms.load(Ordering::Relaxed),
            compactions_total: self.compactions_total.load(Ordering::Relaxed),
            compaction_messages_dropped: self.compaction_messages_dropped.load(Ordering::Relaxed),
            replans_total: self.replans_total.load(Ordering::Relaxed),
            replan_successes: self.replan_successes.load(Ordering::Relaxed),
            tool_health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_default_all_zero() {
        let m = AgentMetrics::default();
        let s = m.snapshot();
        assert_eq!(s.llm_requests_total, 0);
        assert_eq!(s.tool_executions_total, 0);
        assert_eq!(s.compactions_total, 0);
    }

    #[test]
    fn record_llm_request_increments() {
        let m = AgentMetrics::default();
        m.record_llm_request(150);
        m.record_llm_request(200);
        let s = m.snapshot();
        assert_eq!(s.llm_requests_total, 2);
        assert_eq!(s.llm_latency_sum_ms, 350);
    }

    #[test]
    fn record_tool_execution_tracks_per_tool() {
        let m = AgentMetrics::default();
        m.record_tool_execution("bash", 100, false);
        m.record_tool_execution("bash", 200, true);
        m.record_tool_execution("read", 50, false);
        let s = m.snapshot();
        assert_eq!(s.tool_executions_total, 3);
        assert_eq!(s.tool_failures_total, 1);
        let bash = &s.tool_health["bash"];
        assert_eq!(bash.executions, 2);
        assert_eq!(bash.failures, 1);
        assert!((bash.success_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn tool_health_stats_success_rate_empty() {
        let stats = ToolHealthStats::default();
        assert!((stats.success_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn record_compaction_increments() {
        let m = AgentMetrics::default();
        m.record_compaction(10);
        m.record_compaction(5);
        let s = m.snapshot();
        assert_eq!(s.compactions_total, 2);
        assert_eq!(s.compaction_messages_dropped, 15);
    }
}
