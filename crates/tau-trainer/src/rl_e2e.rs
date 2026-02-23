//! Deterministic RL end-to-end harness for operator verification.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use tau_algorithm::{
    compute_gae_batch_from_slices, compute_ppo_update, GaeConfig, PpoConfig, PpoSample,
};
use tau_training_runner::{RolloutExecutionOutcome, RolloutExecutor, TrainingTracer};
use tau_training_store::{InMemoryTrainingStore, RolloutQuery, TrainingStore};
use tau_training_types::{ResourcesUpdate, Reward, Rollout};

use crate::{Trainer, TrainerConfig, TrainingSummary};

/// Runtime configuration for the deterministic RL harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RlE2eHarnessConfig {
    pub run_id: String,
    pub output_dir: PathBuf,
}

impl Default for RlE2eHarnessConfig {
    fn default() -> Self {
        Self {
            run_id: "deterministic".to_string(),
            output_dir: PathBuf::from("artifacts/rl-e2e"),
        }
    }
}

/// Rollout metrics emitted by the harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RlE2eRolloutSummary {
    pub total_rollouts: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub cancelled: usize,
}

/// GAE summary emitted by the harness.
#[derive(Debug, Clone, PartialEq)]
pub struct RlE2eGaeSummary {
    pub advantages_len: usize,
    pub mean_advantage: f64,
    pub mean_return: f64,
    pub normalized: bool,
}

/// PPO summary emitted by the harness.
#[derive(Debug, Clone, PartialEq)]
pub struct RlE2ePpoSummary {
    pub mini_batch_count: usize,
    pub optimizer_step_count: usize,
    pub mean_total_loss: f64,
    pub observed_approx_kl: f64,
    pub early_stop_triggered: bool,
}

/// Check entry attached to harness artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RlE2eCheck {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

/// Deterministic RL e2e artifact.
#[derive(Debug, Clone, PartialEq)]
pub struct RlE2eArtifact {
    pub schema_version: u32,
    pub run_id: String,
    pub rollout_summary: RlE2eRolloutSummary,
    pub gae_summary: RlE2eGaeSummary,
    pub ppo_summary: RlE2ePpoSummary,
    pub checks: Vec<RlE2eCheck>,
    pub pass: bool,
}

impl RlE2eArtifact {
    pub const SCHEMA_VERSION_V1: u32 = 1;

    /// Projects the artifact into JSON for deterministic export.
    pub fn to_json_value(&self) -> Value {
        json!({
            "schema_version": self.schema_version,
            "run_id": self.run_id,
            "rollout_summary": {
                "total_rollouts": self.rollout_summary.total_rollouts,
                "succeeded": self.rollout_summary.succeeded,
                "failed": self.rollout_summary.failed,
                "cancelled": self.rollout_summary.cancelled,
            },
            "gae_summary": {
                "advantages_len": self.gae_summary.advantages_len,
                "mean_advantage": self.gae_summary.mean_advantage,
                "mean_return": self.gae_summary.mean_return,
                "normalized": self.gae_summary.normalized,
            },
            "ppo_summary": {
                "mini_batch_count": self.ppo_summary.mini_batch_count,
                "optimizer_step_count": self.ppo_summary.optimizer_step_count,
                "mean_total_loss": self.ppo_summary.mean_total_loss,
                "observed_approx_kl": self.ppo_summary.observed_approx_kl,
                "early_stop_triggered": self.ppo_summary.early_stop_triggered,
            },
            "checks": self.checks.iter().map(|check| {
                json!({
                    "id": check.id,
                    "passed": check.passed,
                    "detail": check.detail,
                })
            }).collect::<Vec<_>>(),
            "pass": self.pass,
        })
    }
}

/// File-export summary for persisted harness artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RlE2eArtifactExportSummary {
    pub path: PathBuf,
    pub bytes_written: usize,
}

/// Runs the deterministic RL harness and returns an in-memory artifact.
pub async fn run_deterministic_rl_e2e_harness(
    config: &RlE2eHarnessConfig,
) -> Result<RlE2eArtifact> {
    validate_output_dir_path(&config.output_dir)?;

    let store: Arc<dyn TrainingStore> = Arc::new(InMemoryTrainingStore::new());
    let trainer = Trainer::new(
        store.clone(),
        TrainerConfig {
            worker_count: 2,
            poll_interval: Duration::from_millis(10),
            heartbeat_interval: Duration::from_millis(20),
            completion_poll_interval: Duration::from_millis(10),
            completion_timeout: Duration::from_secs(10),
        },
    );

    let dataset = deterministic_rollout_inputs();
    let summary = trainer
        .fit(
            Arc::new(DeterministicHarnessExecutor),
            Some(dataset),
            Option::<Vec<Value>>::None,
        )
        .await
        .context("run deterministic trainer fit")?;

    let mut rollouts = store
        .query_rollouts(RolloutQuery::default())
        .await
        .context("query deterministic harness rollouts")?;
    rollouts.sort_by(|left, right| left.rollout_id.cmp(&right.rollout_id));

    let (rewards, values, dones) = derive_reward_and_value_sequences(&rollouts)?;
    let gae = compute_gae_batch_from_slices(
        &GaeConfig::default(),
        format!("{}-gae-batch", config.run_id),
        format!("{}-trajectory", config.run_id),
        &rewards,
        &values,
        &dones,
        0.0,
    )
    .context("compute deterministic GAE summary")?;

    let ppo_samples = build_ppo_samples(&gae.advantages, &gae.returns, &gae.value_targets)?;
    let ppo = compute_ppo_update(
        &PpoConfig {
            mini_batch_size: 2,
            gradient_accumulation_steps: 1,
            epochs: 2,
            max_kl: Some(0.5),
            ..PpoConfig::default()
        },
        &ppo_samples,
    )
    .context("compute deterministic PPO summary")?;

    let rollout_summary = summary_to_rollout_summary(summary);
    let gae_summary = RlE2eGaeSummary {
        advantages_len: gae.advantages.len(),
        mean_advantage: mean(&gae.advantages),
        mean_return: mean(&gae.returns),
        normalized: gae.normalized,
    };
    let ppo_summary = RlE2ePpoSummary {
        mini_batch_count: ppo.mini_batch_count,
        optimizer_step_count: ppo.optimizer_step_count,
        mean_total_loss: ppo.mean_loss.total_loss,
        observed_approx_kl: ppo.observed_approx_kl,
        early_stop_triggered: ppo.early_stop_triggered,
    };

    let checks = vec![
        RlE2eCheck {
            id: "rollout_completion".to_string(),
            passed: rollout_summary.failed == 0 && rollout_summary.cancelled == 0,
            detail: format!(
                "failed={} cancelled={}",
                rollout_summary.failed, rollout_summary.cancelled
            ),
        },
        RlE2eCheck {
            id: "gae_numeric".to_string(),
            passed: gae_summary.mean_advantage.is_finite() && gae_summary.mean_return.is_finite(),
            detail: format!(
                "mean_advantage={:.6} mean_return={:.6}",
                gae_summary.mean_advantage, gae_summary.mean_return
            ),
        },
        RlE2eCheck {
            id: "ppo_numeric".to_string(),
            passed: ppo_summary.mean_total_loss.is_finite()
                && ppo_summary.observed_approx_kl.is_finite(),
            detail: format!(
                "mean_total_loss={:.6} observed_approx_kl={:.6}",
                ppo_summary.mean_total_loss, ppo_summary.observed_approx_kl
            ),
        },
    ];
    let pass = checks.iter().all(|entry| entry.passed);

    Ok(RlE2eArtifact {
        schema_version: RlE2eArtifact::SCHEMA_VERSION_V1,
        run_id: config.run_id.clone(),
        rollout_summary,
        gae_summary,
        ppo_summary,
        checks,
        pass,
    })
}

/// Exports an RL harness artifact to disk.
pub fn export_rl_e2e_harness_artifact(
    artifact: &RlE2eArtifact,
    output_dir: &Path,
) -> Result<RlE2eArtifactExportSummary> {
    validate_output_dir_path(output_dir)?;
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create RL e2e output directory {}",
            output_dir.display()
        )
    })?;
    let path = output_dir.join(rl_e2e_harness_filename(&artifact.run_id));
    let payload = serde_json::to_string_pretty(&artifact.to_json_value())
        .context("serialize RL e2e artifact payload")?;
    fs::write(&path, payload.as_bytes())
        .with_context(|| format!("failed to write RL e2e artifact {}", path.display()))?;
    Ok(RlE2eArtifactExportSummary {
        path,
        bytes_written: payload.len(),
    })
}

/// Resolves the deterministic artifact filename for an RL harness run id.
pub fn rl_e2e_harness_filename(run_id: &str) -> PathBuf {
    let normalized = run_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    PathBuf::from(format!("rl-e2e-harness-v1-{normalized}.json"))
}

fn deterministic_rollout_inputs() -> Vec<Value> {
    vec![
        json!({"prompt": "math-1", "reward": 1.0, "value_estimate": 0.4}),
        json!({"prompt": "math-2", "reward": 0.9, "value_estimate": 0.35}),
        json!({"prompt": "safety-1", "reward": 0.8, "value_estimate": 0.30}),
        json!({"prompt": "tool-use-1", "reward": 0.95, "value_estimate": 0.45}),
        json!({"prompt": "planning-1", "reward": 0.85, "value_estimate": 0.40}),
        json!({"prompt": "memory-1", "reward": 0.92, "value_estimate": 0.38}),
    ]
}

fn derive_reward_and_value_sequences(
    rollouts: &[Rollout],
) -> Result<(Vec<f64>, Vec<f64>, Vec<bool>)> {
    if rollouts.is_empty() {
        bail!("rollout set for RL e2e harness must not be empty");
    }

    let mut rewards = Vec::with_capacity(rollouts.len());
    let mut values = Vec::with_capacity(rollouts.len());
    let mut dones = Vec::with_capacity(rollouts.len());
    for (index, rollout) in rollouts.iter().enumerate() {
        rewards.push(
            required_f64_field(&rollout.input, "reward")
                .with_context(|| format!("read reward from rollout {}", rollout.rollout_id))?,
        );
        values.push(
            required_f64_field(&rollout.input, "value_estimate").with_context(|| {
                format!("read value_estimate from rollout {}", rollout.rollout_id)
            })?,
        );
        dones.push(index + 1 == rollouts.len());
    }

    Ok((rewards, values, dones))
}

fn required_f64_field(payload: &Value, key: &str) -> Result<f64> {
    let value = payload
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("payload field `{key}` must be a numeric value"))?;
    if !value.is_finite() {
        bail!("payload field `{key}` must be finite");
    }
    Ok(value)
}

fn build_ppo_samples(
    advantages: &[f64],
    returns: &[f64],
    values: &[f64],
) -> Result<Vec<PpoSample>> {
    if advantages.is_empty() {
        bail!("advantages must not be empty");
    }
    if returns.len() != advantages.len() || values.len() != advantages.len() {
        bail!(
            "ppo sample construction length mismatch: advantages={} returns={} values={}",
            advantages.len(),
            returns.len(),
            values.len()
        );
    }

    Ok(advantages
        .iter()
        .enumerate()
        .map(|(index, advantage)| {
            let base = index as f64 * 0.05;
            PpoSample {
                old_logprob: -0.9 + base,
                new_logprob: -0.885 + base,
                advantage: *advantage,
                return_value: returns[index],
                value_prediction: values[index],
                entropy: 0.06 + (index as f64 * 0.002),
            }
        })
        .collect())
}

fn summary_to_rollout_summary(summary: TrainingSummary) -> RlE2eRolloutSummary {
    RlE2eRolloutSummary {
        total_rollouts: summary.total_rollouts,
        succeeded: summary.succeeded,
        failed: summary.failed,
        cancelled: summary.cancelled,
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn validate_output_dir_path(path: &Path) -> Result<()> {
    if path.exists() && !path.is_dir() {
        bail!(
            "output_dir '{}' must point to a directory path",
            path.display()
        );
    }
    Ok(())
}

struct DeterministicHarnessExecutor;

#[async_trait]
impl RolloutExecutor for DeterministicHarnessExecutor {
    async fn execute(
        &self,
        rollout: &Rollout,
        _resources: Option<&ResourcesUpdate>,
        _tracer: Arc<TrainingTracer>,
    ) -> Result<RolloutExecutionOutcome> {
        let prompt = rollout
            .input
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let reward = required_f64_field(&rollout.input, "reward")?;

        Ok(RolloutExecutionOutcome {
            output: json!({
                "assistant_text": format!("ok:{prompt}"),
                "reward": reward,
            }),
            rewards: vec![Reward::new("deterministic_reward", reward)],
        })
    }
}
