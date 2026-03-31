//! Status bar showing model, tokens, cost, and agent state.

/// Status bar state.
pub struct StatusBar {
    pub model: String,
    pub profile: String,
    pub transport: TransportDisplay,
    pub active_skills: Vec<String>,
    pub active_mission_id: Option<String>,
    pub total_tokens: u64,
    pub total_cost_cents: f64,
    pub total_messages: u64,
    pub circuit_breaker_state: CircuitBreakerDisplay,
    pub agent_state: AgentStateDisplay,
}

/// Transport display state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportDisplay {
    Gateway,
}

impl TransportDisplay {
    pub fn label(&self) -> &'static str {
        match self {
            TransportDisplay::Gateway => "transport=gateway",
        }
    }
}

/// Circuit breaker display state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerDisplay {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreakerDisplay {
    pub fn label(&self) -> &'static str {
        match self {
            CircuitBreakerDisplay::Closed => "OK",
            CircuitBreakerDisplay::Open => "OPEN",
            CircuitBreakerDisplay::HalfOpen => "HALF",
        }
    }
}

/// Agent operational state display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStateDisplay {
    Idle,
    Thinking,
    ToolExec,
    Streaming,
    Error,
}

impl AgentStateDisplay {
    pub fn label(&self) -> &'static str {
        match self {
            AgentStateDisplay::Idle => "IDLE",
            AgentStateDisplay::Thinking => "THINKING",
            AgentStateDisplay::ToolExec => "TOOL",
            AgentStateDisplay::Streaming => "STREAM",
            AgentStateDisplay::Error => "ERROR",
        }
    }
}

impl StatusBar {
    pub fn new(model: String, profile: String) -> Self {
        Self {
            model,
            profile,
            transport: TransportDisplay::Gateway,
            active_skills: Vec::new(),
            active_mission_id: None,
            total_tokens: 0,
            total_cost_cents: 0.0,
            total_messages: 0,
            circuit_breaker_state: CircuitBreakerDisplay::Closed,
            agent_state: AgentStateDisplay::Idle,
        }
    }

    pub fn format_cost(&self) -> String {
        if self.total_cost_cents < 100.0 {
            format!("{:.1}c", self.total_cost_cents)
        } else {
            format!("${:.2}", self.total_cost_cents / 100.0)
        }
    }

    pub fn format_tokens(&self) -> String {
        if self.total_tokens < 1000 {
            format!("{}", self.total_tokens)
        } else if self.total_tokens < 1_000_000 {
            format!("{:.1}k", self.total_tokens as f64 / 1000.0)
        } else {
            format!("{:.1}M", self.total_tokens as f64 / 1_000_000.0)
        }
    }

    pub fn format_active_skills(&self) -> Option<String> {
        if self.active_skills.is_empty() {
            return None;
        }

        let joined = self.active_skills.join(",");
        if joined.chars().count() <= 32 {
            return Some(joined);
        }

        let truncated = joined.chars().take(31).collect::<String>();
        Some(format!("{truncated}..."))
    }

    pub fn format_active_mission(&self) -> Option<String> {
        let mission_id = self
            .active_mission_id
            .as_deref()
            .map(str::trim)
            .filter(|mission_id| !mission_id.is_empty())?;
        if mission_id.chars().count() <= 28 {
            return Some(mission_id.to_string());
        }

        let truncated = mission_id.chars().take(27).collect::<String>();
        Some(format!("{truncated}..."))
    }
}
