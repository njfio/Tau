//! Reward inference contracts and deterministic trace-based scoring.

/// Immutable signals used to infer reward from an observed trace/run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardInferenceInput {
    pub has_assistant_reply: bool,
    pub session_completed: bool,
    pub tool_errors: u32,
    pub safety_blocked: bool,
    pub turns: u32,
    pub input_chars: usize,
    pub output_chars: usize,
}

impl RewardInferenceInput {
    /// Creates an inference input with explicit runtime signals.
    pub fn new(
        has_assistant_reply: bool,
        session_completed: bool,
        tool_errors: u32,
        safety_blocked: bool,
        turns: u32,
        input_chars: usize,
        output_chars: usize,
    ) -> Self {
        Self {
            has_assistant_reply,
            session_completed,
            tool_errors,
            safety_blocked,
            turns,
            input_chars,
            output_chars,
        }
    }
}

/// Deterministic reward inference result with component visibility.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RewardInferenceOutput {
    pub composite: f64,
    pub completion: f64,
    pub session_completion: f64,
    pub reliability: f64,
    pub safety: f64,
    pub efficiency: f64,
    pub token_efficiency: f64,
    pub confidence: f64,
}

impl RewardInferenceOutput {
    fn new(
        composite: f64,
        completion: f64,
        session_completion: f64,
        reliability: f64,
        safety: f64,
        efficiency: f64,
        token_efficiency: f64,
        confidence: f64,
    ) -> Self {
        Self {
            composite,
            completion,
            session_completion,
            reliability,
            safety,
            efficiency,
            token_efficiency,
            confidence,
        }
    }
}

/// Contract for reward inference strategies.
pub trait RewardInference: Send + Sync {
    fn infer(&self, input: &RewardInferenceInput) -> RewardInferenceOutput;
}

/// Trace-based deterministic reward inference strategy.
#[derive(Debug, Clone, Default)]
pub struct TraceBasedRewardInference;

impl RewardInference for TraceBasedRewardInference {
    fn infer(&self, input: &RewardInferenceInput) -> RewardInferenceOutput {
        let completion = if input.has_assistant_reply { 0.5 } else { 0.0 };
        let session_completion = if input.session_completed { 0.0 } else { 0.0 };
        let reliability = -0.25 * f64::from(input.tool_errors.min(2));
        let efficiency = if input.turns <= 2 {
            0.5
        } else if input.turns <= 4 {
            0.25
        } else {
            0.0
        };
        let token_efficiency = 0.0;
        let safety = if input.safety_blocked { -1.0 } else { 0.0 };

        let confidence = (0.5_f64
            + if input.has_assistant_reply {
                0.25_f64
            } else {
                0.0_f64
            }
            + if input.turns > 0 { 0.25_f64 } else { 0.0_f64 })
        .clamp(0.0, 1.0);

        if input.safety_blocked {
            return RewardInferenceOutput::new(
                -1.0,
                completion,
                session_completion,
                reliability,
                safety,
                efficiency,
                token_efficiency,
                confidence,
            );
        }

        let composite =
            (completion + session_completion + reliability + efficiency + token_efficiency)
                .clamp(-1.0, 1.0);
        RewardInferenceOutput::new(
            composite,
            completion,
            session_completion,
            reliability,
            safety,
            efficiency,
            token_efficiency,
            confidence,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RewardInference, RewardInferenceInput, RewardInferenceOutput, TraceBasedRewardInference,
    };

    #[test]
    fn spec_c01_unit_trace_based_reward_inference_computes_components() {
        let input = RewardInferenceInput::new(true, true, 0, false, 1, 32, 48);
        let output = TraceBasedRewardInference.infer(&input);

        assert_eq!(
            output,
            RewardInferenceOutput {
                composite: 1.0,
                completion: 0.5,
                session_completion: 0.0,
                reliability: 0.0,
                safety: 0.0,
                efficiency: 0.5,
                token_efficiency: 0.0,
                confidence: 1.0,
            }
        );
    }

    #[test]
    fn spec_c02_regression_trace_based_reward_inference_safety_hard_gate() {
        let input = RewardInferenceInput::new(true, true, 0, true, 1, 32, 48);
        let output = TraceBasedRewardInference.infer(&input);

        assert_eq!(output.composite, -1.0);
        assert_eq!(output.safety, -1.0);
    }

    #[test]
    fn spec_c03_unit_trace_based_reward_inference_token_efficiency_signal() {
        let efficient = RewardInferenceInput::new(true, true, 0, false, 1, 24, 24);
        let inefficient = RewardInferenceInput::new(true, true, 0, false, 1, 24, 96);

        let efficient_output = TraceBasedRewardInference.infer(&efficient);
        let inefficient_output = TraceBasedRewardInference.infer(&inefficient);

        assert!(efficient_output.token_efficiency > 0.0);
        assert!(inefficient_output.token_efficiency < efficient_output.token_efficiency);
    }

    #[test]
    fn spec_c04_regression_trace_based_reward_inference_session_not_completed_penalty() {
        let completed = RewardInferenceInput::new(true, true, 0, false, 1, 24, 24);
        let not_completed = RewardInferenceInput::new(true, false, 0, false, 1, 24, 24);

        let completed_output = TraceBasedRewardInference.infer(&completed);
        let not_completed_output = TraceBasedRewardInference.infer(&not_completed);

        assert!(not_completed_output.session_completion < 0.0);
        assert!(not_completed_output.composite < completed_output.composite);
    }
}
