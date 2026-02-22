//! Reward inference contracts and deterministic trace-based scoring.

/// Immutable signals used to infer reward from an observed trace/run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardInferenceInput {
    pub has_assistant_reply: bool,
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
        tool_errors: u32,
        safety_blocked: bool,
        turns: u32,
        input_chars: usize,
        output_chars: usize,
    ) -> Self {
        Self {
            has_assistant_reply,
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
    pub reliability: f64,
    pub safety: f64,
    pub efficiency: f64,
    pub confidence: f64,
}

impl RewardInferenceOutput {
    fn new(
        composite: f64,
        completion: f64,
        reliability: f64,
        safety: f64,
        efficiency: f64,
        confidence: f64,
    ) -> Self {
        Self {
            composite,
            completion,
            reliability,
            safety,
            efficiency,
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
        let reliability = -0.25 * f64::from(input.tool_errors.min(2));
        let efficiency = if input.turns <= 2 {
            0.5
        } else if input.turns <= 4 {
            0.25
        } else {
            0.0
        };
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
                reliability,
                safety,
                efficiency,
                confidence,
            );
        }

        let composite = (completion + reliability + efficiency).clamp(-1.0, 1.0);
        RewardInferenceOutput::new(
            composite,
            completion,
            reliability,
            safety,
            efficiency,
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
        let input = RewardInferenceInput::new(true, 0, false, 1, 32, 48);
        let output = TraceBasedRewardInference.infer(&input);

        assert_eq!(
            output,
            RewardInferenceOutput {
                composite: 1.0,
                completion: 0.5,
                reliability: 0.0,
                safety: 0.0,
                efficiency: 0.5,
                confidence: 1.0,
            }
        );
    }

    #[test]
    fn spec_c02_regression_trace_based_reward_inference_safety_hard_gate() {
        let input = RewardInferenceInput::new(true, 0, true, 1, 32, 48);
        let output = TraceBasedRewardInference.infer(&input);

        assert_eq!(output.composite, -1.0);
        assert_eq!(output.safety, -1.0);
    }
}
