//! Live RL runtime bridge for wiring agent decision traces into rollout updates.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tau_agent_core::{Agent, AgentEvent};
use tau_ai::{ChatRequest, LlmClient, Message, MessageRole, ModelRef};
use tau_algorithm::{
    collect_trajectory_batch, compute_gae_batch_from_slices, compute_ppo_update, Algorithm,
    AlgorithmContext, ApoAlgorithm, ApoConfig, GaeConfig, PpoConfig, PpoSample, PromptEvaluator,
    PromptExample, RewardInference, RewardInferenceInput, RewardInferenceOutput,
    TraceBasedRewardInference,
};
use tau_trainer::benchmark_significance::compare_policy_improvement;
use tau_training_store::{
    Attempt, AttemptStatus, DequeuedRollout, ResourcesUpdate, Rollout, RolloutQuery, RolloutStatus,
    SqliteTrainingStore, StoreResult, TrainingSpan, TrainingStore, WorkerState,
};
use tokio::sync::Mutex;

const LIVE_RL_ENABLED_ENV: &str = "TAU_LIVE_RL_ENABLED";
const LIVE_RL_STORE_PATH_ENV: &str = "TAU_LIVE_RL_STORE_SQLITE";
const LIVE_RL_UPDATE_INTERVAL_ENV: &str = "TAU_LIVE_RL_UPDATE_INTERVAL";
const LIVE_RL_MAX_ROLLOUTS_ENV: &str = "TAU_LIVE_RL_MAX_ROLLOUTS_PER_UPDATE";
const LIVE_RL_MAX_FAILURE_STREAK_ENV: &str = "TAU_LIVE_RL_MAX_FAILURE_STREAK";
const LIVE_RL_APO_ENABLED_ENV: &str = "TAU_LIVE_RL_APO_ENABLED";
const LIVE_RL_APO_MIN_SAMPLES_ENV: &str = "TAU_LIVE_RL_APO_MIN_SAMPLES";
const LIVE_RL_APO_MAX_SAMPLES_ENV: &str = "TAU_LIVE_RL_APO_MAX_SAMPLES";
const LIVE_RL_APO_SIGNIFICANCE_ALPHA_ENV: &str = "TAU_LIVE_RL_APO_SIGNIFICANCE_ALPHA";
const LIVE_ROLLOUT_PREFIX: &str = "live-rl-rollout";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveRlRuntimeGate {
    Pass,
    Hold,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveRlOptimizerReport {
    pub executed: bool,
    pub trajectories: usize,
    pub samples: usize,
    pub mean_total_loss: Option<f64>,
    pub observed_approx_kl: Option<f64>,
    pub early_stop_triggered: bool,
    pub apo: Option<LiveApoReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveApoReport {
    pub executed: bool,
    pub adopted: bool,
    pub sample_count: usize,
    pub curriculum_focus_category: Option<String>,
    pub curriculum_focus_mean_reward: Option<f64>,
    pub baseline_mean_reward: Option<f64>,
    pub candidate_mean_reward: Option<f64>,
    pub best_prompt_version: Option<String>,
    pub best_prompt_score: Option<f64>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveRlRuntimeSnapshot {
    pub enabled: bool,
    pub store_path: PathBuf,
    pub gate: LiveRlRuntimeGate,
    pub completed_rollouts: usize,
    pub consecutive_failures: usize,
    pub last_error: Option<String>,
    pub last_optimizer_report: Option<LiveRlOptimizerReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveRlRuntimeConfig {
    pub enabled: bool,
    pub store_path: PathBuf,
    pub update_interval_rollouts: usize,
    pub max_rollouts_per_update: usize,
    pub max_failure_streak: usize,
    pub apo_enabled: bool,
    pub apo_min_samples: usize,
    pub apo_max_samples: usize,
    pub apo_significance_alpha: f64,
}

impl LiveRlRuntimeConfig {
    pub(crate) fn from_env_map(
        env: &BTreeMap<String, String>,
        default_store_path: &Path,
    ) -> Result<Self> {
        let enabled = match env.get(LIVE_RL_ENABLED_ENV) {
            Some(raw) => parse_bool_env(raw).ok_or_else(|| {
                anyhow!("{LIVE_RL_ENABLED_ENV} must be one of 1,true,yes,on,0,false,no,off")
            })?,
            None => false,
        };

        let store_path = env
            .get(LIVE_RL_STORE_PATH_ENV)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| default_store_path.to_path_buf());

        let update_interval_rollouts = parse_positive_usize_env(
            env.get(LIVE_RL_UPDATE_INTERVAL_ENV).map(String::as_str),
            8,
            LIVE_RL_UPDATE_INTERVAL_ENV,
        )?;
        let max_rollouts_per_update = parse_positive_usize_env(
            env.get(LIVE_RL_MAX_ROLLOUTS_ENV).map(String::as_str),
            64,
            LIVE_RL_MAX_ROLLOUTS_ENV,
        )?;
        let max_failure_streak = parse_positive_usize_env(
            env.get(LIVE_RL_MAX_FAILURE_STREAK_ENV).map(String::as_str),
            3,
            LIVE_RL_MAX_FAILURE_STREAK_ENV,
        )?;
        let apo_enabled = match env.get(LIVE_RL_APO_ENABLED_ENV) {
            Some(raw) => parse_bool_env(raw).ok_or_else(|| {
                anyhow!("{LIVE_RL_APO_ENABLED_ENV} must be one of 1,true,yes,on,0,false,no,off")
            })?,
            None => true,
        };
        let apo_min_samples = parse_positive_usize_env(
            env.get(LIVE_RL_APO_MIN_SAMPLES_ENV).map(String::as_str),
            4,
            LIVE_RL_APO_MIN_SAMPLES_ENV,
        )?;
        let apo_max_samples = parse_positive_usize_env(
            env.get(LIVE_RL_APO_MAX_SAMPLES_ENV).map(String::as_str),
            32,
            LIVE_RL_APO_MAX_SAMPLES_ENV,
        )?;
        if apo_min_samples > apo_max_samples {
            return Err(anyhow!(
                "{LIVE_RL_APO_MIN_SAMPLES_ENV} cannot be greater than {LIVE_RL_APO_MAX_SAMPLES_ENV}"
            ));
        }
        let apo_significance_alpha = parse_significance_alpha_env(
            env.get(LIVE_RL_APO_SIGNIFICANCE_ALPHA_ENV)
                .map(String::as_str),
            0.05,
            LIVE_RL_APO_SIGNIFICANCE_ALPHA_ENV,
        )?;

        Ok(Self {
            enabled,
            store_path,
            update_interval_rollouts,
            max_rollouts_per_update,
            max_failure_streak,
            apo_enabled,
            apo_min_samples,
            apo_max_samples,
            apo_significance_alpha,
        })
    }
}

#[derive(Clone)]
pub(crate) struct LiveRlRuntimeBridge {
    inner: Arc<LiveRlRuntimeBridgeInner>,
}

struct LiveRlRuntimeBridgeInner {
    store: Arc<dyn TrainingStore + Send + Sync>,
    config: LiveRlRuntimeConfig,
    apo_runtime: Option<LiveApoRuntime>,
    state: Mutex<LiveRlRuntimeState>,
}

#[derive(Debug)]
struct LiveRlRuntimeState {
    gate: LiveRlRuntimeGate,
    next_rollout_sequence: u64,
    completed_rollouts: usize,
    consecutive_failures: usize,
    last_error: Option<String>,
    last_optimizer_report: Option<LiveRlOptimizerReport>,
    active_run: Option<LiveRlActiveRun>,
}

#[derive(Debug, Clone)]
struct LiveRlActiveRun {
    rollout_id: String,
    attempt_id: String,
    prompt: Option<String>,
    assistant_reply: Option<String>,
    turns: u32,
    tool_errors: u32,
    safety_blocked: bool,
}

#[derive(Clone)]
struct LiveApoRuntime {
    client: Arc<dyn LlmClient>,
    model: String,
    seed_system_prompt: String,
}

#[derive(Debug, Clone, PartialEq)]
struct LiveApoSample {
    prompt: String,
    response: String,
    reward: f64,
    category: String,
}

#[derive(Debug, Clone, PartialEq)]
struct LiveCategoryOutcome {
    category: String,
    reward: f64,
    predicted_success_probability: f64,
    actual_success: bool,
}

/// Proxy around the training store that suppresses resource persistence side effects.
///
/// APO currently writes resources during `run()`. Live RL applies significance gating after
/// evaluation, so this adapter prevents writes during candidate evaluation and only allows
/// explicit persistence once a significant improvement is proven.
#[derive(Clone)]
struct NoResourceWriteStore {
    inner: Arc<dyn TrainingStore + Send + Sync>,
}

impl NoResourceWriteStore {
    fn new(inner: Arc<dyn TrainingStore + Send + Sync>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl TrainingStore for NoResourceWriteStore {
    async fn enqueue_rollout(&self, rollout: Rollout) -> StoreResult<()> {
        self.inner.enqueue_rollout(rollout).await
    }

    async fn dequeue_rollout(&self, worker_id: &str) -> StoreResult<Option<DequeuedRollout>> {
        self.inner.dequeue_rollout(worker_id).await
    }

    async fn update_rollout_status(
        &self,
        rollout_id: &str,
        status: RolloutStatus,
    ) -> StoreResult<()> {
        self.inner.update_rollout_status(rollout_id, status).await
    }

    async fn cancel_rollout(&self, rollout_id: &str) -> StoreResult<()> {
        self.inner.cancel_rollout(rollout_id).await
    }

    async fn add_span(&self, span: TrainingSpan) -> StoreResult<()> {
        self.inner.add_span(span).await
    }

    async fn query_spans(
        &self,
        rollout_id: &str,
        attempt_id: Option<&str>,
    ) -> StoreResult<Vec<TrainingSpan>> {
        self.inner.query_spans(rollout_id, attempt_id).await
    }

    async fn get_next_span_sequence_id(
        &self,
        rollout_id: &str,
        attempt_id: &str,
    ) -> StoreResult<u64> {
        self.inner
            .get_next_span_sequence_id(rollout_id, attempt_id)
            .await
    }

    async fn update_resources(
        &self,
        resources: HashMap<String, Value>,
    ) -> StoreResult<ResourcesUpdate> {
        let version = self
            .inner
            .get_latest_resources()
            .await?
            .map(|current| current.version.saturating_add(1))
            .unwrap_or(1);
        Ok(ResourcesUpdate {
            resources_id: format!("live-apo-shadow-{version}"),
            version,
            resources,
            created_time: Utc::now(),
            is_latest: false,
        })
    }

    async fn get_latest_resources(&self) -> StoreResult<Option<ResourcesUpdate>> {
        self.inner.get_latest_resources().await
    }

    async fn get_resources_by_id(
        &self,
        resources_id: &str,
    ) -> StoreResult<Option<ResourcesUpdate>> {
        self.inner.get_resources_by_id(resources_id).await
    }

    async fn query_rollouts(&self, query: RolloutQuery) -> StoreResult<Vec<Rollout>> {
        self.inner.query_rollouts(query).await
    }

    async fn wait_for_rollouts(
        &self,
        statuses: &[RolloutStatus],
        timeout: Duration,
    ) -> StoreResult<Vec<Rollout>> {
        self.inner.wait_for_rollouts(statuses, timeout).await
    }

    async fn register_worker(&self, worker_id: &str) -> StoreResult<WorkerState> {
        self.inner.register_worker(worker_id).await
    }

    async fn update_worker_heartbeat(
        &self,
        worker_id: &str,
        active_rollout_id: Option<String>,
        active_attempt_id: Option<String>,
    ) -> StoreResult<()> {
        self.inner
            .update_worker_heartbeat(worker_id, active_rollout_id, active_attempt_id)
            .await
    }

    async fn reassign_timed_out_rollouts(
        &self,
        heartbeat_timeout: Duration,
    ) -> StoreResult<Vec<String>> {
        self.inner
            .reassign_timed_out_rollouts(heartbeat_timeout)
            .await
    }

    async fn query_workers(&self) -> StoreResult<Vec<WorkerState>> {
        self.inner.query_workers().await
    }

    async fn update_attempt_status(
        &self,
        attempt_id: &str,
        status: AttemptStatus,
        error_message: Option<String>,
    ) -> StoreResult<()> {
        self.inner
            .update_attempt_status(attempt_id, status, error_message)
            .await
    }

    async fn get_attempt(&self, attempt_id: &str) -> StoreResult<Option<Attempt>> {
        self.inner.get_attempt(attempt_id).await
    }
}

impl LiveRlRuntimeBridge {
    pub(crate) fn register_if_enabled(
        agent: &mut Agent,
        default_store_path: &Path,
        client: Arc<dyn LlmClient>,
        model_ref: &ModelRef,
        seed_system_prompt: &str,
    ) -> Result<Option<LiveRlRuntimeSnapshot>> {
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        let config = LiveRlRuntimeConfig::from_env_map(&env, default_store_path)
            .context("failed to resolve live RL runtime config")?;
        if !config.enabled {
            return Ok(None);
        }

        let sqlite_store = Arc::new(
            SqliteTrainingStore::new(config.store_path.as_path()).with_context(|| {
                format!(
                    "failed to initialize live RL training store at {}",
                    config.store_path.display()
                )
            })?,
        );

        let apo_runtime = config.apo_enabled.then_some(LiveApoRuntime {
            client,
            model: model_ref.model.clone(),
            seed_system_prompt: seed_system_prompt.to_string(),
        });
        let bridge = Self::new(sqlite_store, config, apo_runtime);
        bridge.register(agent);
        Ok(Some(bridge.snapshot_blocking()))
    }

    fn new(
        store: Arc<dyn TrainingStore + Send + Sync>,
        config: LiveRlRuntimeConfig,
        apo_runtime: Option<LiveApoRuntime>,
    ) -> Self {
        Self {
            inner: Arc::new(LiveRlRuntimeBridgeInner {
                store,
                config,
                apo_runtime,
                state: Mutex::new(LiveRlRuntimeState {
                    gate: LiveRlRuntimeGate::Pass,
                    next_rollout_sequence: 0,
                    completed_rollouts: 0,
                    consecutive_failures: 0,
                    last_error: None,
                    last_optimizer_report: None,
                    active_run: None,
                }),
            }),
        }
    }

    fn register(&self, agent: &mut Agent) {
        let bridge = self.clone();
        agent.subscribe_async(move |event| {
            let bridge = bridge.clone();
            async move {
                bridge.handle_event(event).await;
            }
        });
    }

    pub(crate) async fn handle_event(&self, event: AgentEvent) {
        if !self.inner.config.enabled {
            return;
        }

        match event {
            AgentEvent::AgentStart => {
                self.handle_agent_start().await;
            }
            AgentEvent::AgentEnd { .. } => {
                self.handle_agent_end().await;
            }
            AgentEvent::MessageAdded { message } => {
                self.handle_message_event(message.role, message.text_content())
                    .await;
            }
            AgentEvent::ToolExecutionEnd { result, .. } => {
                if result.is_error {
                    let mut state = self.inner.state.lock().await;
                    if let Some(run) = state.active_run.as_mut() {
                        run.tool_errors = run.tool_errors.saturating_add(1);
                    }
                }
            }
            AgentEvent::TurnEnd { .. } => {
                let mut state = self.inner.state.lock().await;
                if let Some(run) = state.active_run.as_mut() {
                    run.turns = run.turns.saturating_add(1);
                }
            }
            AgentEvent::SafetyPolicyApplied { blocked, .. } => {
                if blocked {
                    let mut state = self.inner.state.lock().await;
                    if let Some(run) = state.active_run.as_mut() {
                        run.safety_blocked = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn snapshot_blocking(&self) -> LiveRlRuntimeSnapshot {
        let state = self.inner.state.blocking_lock();
        LiveRlRuntimeSnapshot {
            enabled: self.inner.config.enabled,
            store_path: self.inner.config.store_path.clone(),
            gate: state.gate,
            completed_rollouts: state.completed_rollouts,
            consecutive_failures: state.consecutive_failures,
            last_error: state.last_error.clone(),
            last_optimizer_report: state.last_optimizer_report.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) async fn snapshot(&self) -> LiveRlRuntimeSnapshot {
        let state = self.inner.state.lock().await;
        LiveRlRuntimeSnapshot {
            enabled: self.inner.config.enabled,
            store_path: self.inner.config.store_path.clone(),
            gate: state.gate,
            completed_rollouts: state.completed_rollouts,
            consecutive_failures: state.consecutive_failures,
            last_error: state.last_error.clone(),
            last_optimizer_report: state.last_optimizer_report.clone(),
        }
    }

    async fn handle_agent_start(&self) {
        let stale_run = {
            let mut state = self.inner.state.lock().await;
            let stale = state.active_run.take();
            if state.gate == LiveRlRuntimeGate::Hold {
                return;
            }
            state.next_rollout_sequence = state.next_rollout_sequence.saturating_add(1);
            let rollout_id = format!("{LIVE_ROLLOUT_PREFIX}-{:010}", state.next_rollout_sequence);
            let attempt_id = format!("{rollout_id}:attempt-live");
            state.active_run = Some(LiveRlActiveRun {
                rollout_id: rollout_id.clone(),
                attempt_id,
                prompt: None,
                assistant_reply: None,
                turns: 0,
                tool_errors: 0,
                safety_blocked: false,
            });
            stale
        };

        if let Some(stale) = stale_run {
            self.finalize_run(stale, RolloutStatus::Cancelled).await;
        }

        let active_rollout_id = {
            let state = self.inner.state.lock().await;
            state
                .active_run
                .as_ref()
                .map(|run| run.rollout_id.clone())
                .unwrap_or_default()
        };

        if active_rollout_id.is_empty() {
            return;
        }

        if let Err(error) = self.create_rollout(active_rollout_id.as_str()).await {
            self.clear_active_run(active_rollout_id.as_str()).await;
            self.register_failure(format!(
                "live RL rollout init failed for {active_rollout_id}: {error}"
            ))
            .await;
        }
    }

    async fn handle_agent_end(&self) {
        let active = {
            let mut state = self.inner.state.lock().await;
            state.active_run.take()
        };
        let Some(active) = active else {
            return;
        };
        self.finalize_run(active, RolloutStatus::Succeeded).await;
    }

    async fn handle_message_event(&self, role: MessageRole, text: String) {
        let normalized = text.trim();
        if normalized.is_empty() {
            return;
        }
        let mut state = self.inner.state.lock().await;
        let Some(run) = state.active_run.as_mut() else {
            return;
        };
        match role {
            MessageRole::User => {
                if run.prompt.is_none() {
                    run.prompt = Some(normalized.to_string());
                }
            }
            MessageRole::Assistant => {
                run.assistant_reply = Some(normalized.to_string());
            }
            _ => {}
        }
    }

    async fn create_rollout(&self, rollout_id: &str) -> Result<()> {
        let mut rollout = Rollout::new(
            rollout_id.to_string(),
            json!({
                "source": "live_rl_runtime",
                "kind": "live_agent_decision",
            }),
            None,
        );
        rollout
            .metadata
            .insert("source".to_string(), json!("live_rl_runtime"));
        self.inner
            .store
            .enqueue_rollout(rollout)
            .await
            .with_context(|| format!("failed to enqueue live rollout '{rollout_id}'"))?;
        self.inner
            .store
            .update_rollout_status(rollout_id, RolloutStatus::Running)
            .await
            .with_context(|| format!("failed to mark live rollout '{rollout_id}' running"))?;
        Ok(())
    }

    async fn finalize_run(&self, run: LiveRlActiveRun, status: RolloutStatus) {
        if status == RolloutStatus::Succeeded {
            let mut span = build_final_decision_span(&run);
            if let Err(error) = self.enrich_final_decision_span(&mut span).await {
                span.attributes.insert(
                    "meta_cognition_enrichment_error".to_string(),
                    json!(error.to_string()),
                );
            }
            if let Err(error) = self.inner.store.add_span(span).await {
                self.register_failure(format!(
                    "live RL span persistence failed for {}: {error}",
                    run.rollout_id
                ))
                .await;
                return;
            }
        }

        if let Err(error) = self
            .inner
            .store
            .update_rollout_status(run.rollout_id.as_str(), status)
            .await
        {
            self.register_failure(format!(
                "live RL rollout status update failed for {}: {error}",
                run.rollout_id
            ))
            .await;
            return;
        }

        if status == RolloutStatus::Succeeded {
            let should_run_update = {
                let mut state = self.inner.state.lock().await;
                state.completed_rollouts = state.completed_rollouts.saturating_add(1);
                state.consecutive_failures = 0;
                state.last_error = None;
                let due =
                    state.completed_rollouts % self.inner.config.update_interval_rollouts == 0;
                if !due {
                    state.last_optimizer_report = None;
                }
                due
            };

            if should_run_update {
                if let Err(error) = self.run_optimizer_update().await {
                    self.register_failure(format!("live RL optimizer update failed: {error}"))
                        .await;
                }
            }
        }
    }

    async fn enrich_final_decision_span(&self, span: &mut TrainingSpan) -> Result<()> {
        let prompt = span
            .attributes
            .get("prompt")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default();
        let category = span
            .attributes
            .get("task_category")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| infer_task_category(prompt));

        span.attributes
            .insert("task_category".to_string(), json!(category.clone()));

        let history = self
            .collect_recent_category_outcomes(category.as_str(), 32)
            .await?;
        let trend = classify_learning_trend(
            history
                .iter()
                .map(|outcome| outcome.reward)
                .collect::<Vec<_>>()
                .as_slice(),
        );
        span.attributes
            .insert("learning_trend".to_string(), json!(trend));
        span.attributes.insert(
            "historical_category_samples".to_string(),
            json!(u64::try_from(history.len()).unwrap_or(0)),
        );

        if !history.is_empty() {
            let historical_success_rate = history
                .iter()
                .filter(|outcome| outcome.actual_success)
                .count() as f64
                / history.len() as f64;
            span.attributes.insert(
                "historical_success_rate".to_string(),
                json!(historical_success_rate),
            );
            let historical_calibration_error = mean(
                history
                    .iter()
                    .map(|outcome| {
                        (outcome.predicted_success_probability
                            - if outcome.actual_success { 1.0 } else { 0.0 })
                        .abs()
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            span.attributes.insert(
                "historical_calibration_error".to_string(),
                json!(historical_calibration_error),
            );
            let ask_for_help =
                history.len() >= 4 && historical_success_rate < 0.55 && trend != "improving";
            span.attributes
                .insert("ask_for_help_recommended".to_string(), json!(ask_for_help));
        }

        Ok(())
    }

    async fn collect_recent_category_outcomes(
        &self,
        category: &str,
        limit: usize,
    ) -> Result<Vec<LiveCategoryOutcome>> {
        let rollouts = self
            .inner
            .store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .context("failed to query succeeded rollouts for category outcomes")?;

        let mut rollout_ids = rollouts
            .into_iter()
            .filter(|rollout| rollout.rollout_id.starts_with(LIVE_ROLLOUT_PREFIX))
            .map(|rollout| rollout.rollout_id)
            .collect::<Vec<_>>();
        rollout_ids.sort();
        if rollout_ids.len() > limit {
            let start = rollout_ids.len() - limit;
            rollout_ids = rollout_ids[start..].to_vec();
        }

        let mut outcomes = Vec::new();
        for rollout_id in rollout_ids {
            let spans = self
                .inner
                .store
                .query_spans(rollout_id.as_str(), None)
                .await
                .with_context(|| format!("failed to query spans for rollout '{rollout_id}'"))?;
            let decision = spans
                .iter()
                .filter(|span| span.name == "live.agent.decision")
                .max_by_key(|span| span.sequence_id);
            let Some(decision) = decision else {
                continue;
            };
            let Some(outcome) = parse_live_category_outcome(decision) else {
                continue;
            };
            if outcome.category == category {
                outcomes.push(outcome);
            }
        }

        Ok(outcomes)
    }

    async fn clear_active_run(&self, rollout_id: &str) {
        let mut state = self.inner.state.lock().await;
        if state
            .active_run
            .as_ref()
            .is_some_and(|run| run.rollout_id == rollout_id)
        {
            state.active_run = None;
        }
    }

    async fn run_optimizer_update(&self) -> Result<()> {
        let rollout_ids = self.collect_live_rollout_ids_for_update().await?;
        if rollout_ids.is_empty() {
            self.set_optimizer_report(LiveRlOptimizerReport {
                executed: false,
                trajectories: 0,
                samples: 0,
                mean_total_loss: None,
                observed_approx_kl: None,
                early_stop_triggered: false,
                apo: None,
            })
            .await;
            return Ok(());
        }

        let trajectory_batch =
            collect_trajectory_batch(self.inner.store.as_ref(), &rollout_ids, None)
                .await
                .context("failed to collect live trajectories")?;
        if trajectory_batch.trajectories.is_empty() {
            self.set_optimizer_report(LiveRlOptimizerReport {
                executed: false,
                trajectories: 0,
                samples: 0,
                mean_total_loss: None,
                observed_approx_kl: None,
                early_stop_triggered: false,
                apo: None,
            })
            .await;
            return Ok(());
        }

        let mut samples = Vec::new();
        let gae_config = GaeConfig::default();
        let ppo_config = PpoConfig::default();

        for trajectory in &trajectory_batch.trajectories {
            if trajectory.steps.is_empty() {
                continue;
            }

            let rewards = trajectory
                .steps
                .iter()
                .map(|step| step.reward)
                .collect::<Vec<_>>();
            let values = trajectory
                .steps
                .iter()
                .map(|step| step.value_estimate.unwrap_or(0.0))
                .collect::<Vec<_>>();
            let dones = trajectory
                .steps
                .iter()
                .map(|step| step.done)
                .collect::<Vec<_>>();

            let gae_batch = compute_gae_batch_from_slices(
                &gae_config,
                format!("live-gae-{}", trajectory.trajectory_id),
                trajectory.trajectory_id.clone(),
                &rewards,
                &values,
                &dones,
                0.0,
            )
            .with_context(|| {
                format!(
                    "failed to compute GAE batch for live trajectory '{}'",
                    trajectory.trajectory_id
                )
            })?;

            for (index, step) in trajectory.steps.iter().enumerate() {
                let logprob = step.logprob.unwrap_or(0.0);
                samples.push(PpoSample {
                    old_logprob: logprob,
                    new_logprob: logprob,
                    advantage: gae_batch.advantages[index],
                    return_value: gae_batch.returns[index],
                    value_prediction: values[index],
                    entropy: step
                        .metadata
                        .get("entropy")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
                });
            }
        }

        if samples.is_empty() {
            self.set_optimizer_report(LiveRlOptimizerReport {
                executed: false,
                trajectories: trajectory_batch.trajectories.len(),
                samples: 0,
                mean_total_loss: None,
                observed_approx_kl: None,
                early_stop_triggered: false,
                apo: None,
            })
            .await;
            return Ok(());
        }

        let update = compute_ppo_update(&ppo_config, &samples)
            .context("failed PPO update for live RL runtime")?;
        let apo = if self.inner.config.apo_enabled {
            Some(
                self.run_live_apo_update(rollout_ids.as_slice())
                    .await
                    .unwrap_or_else(|error| {
                        LiveApoReport::skipped(format!("apo_runtime_error:{error}"), 0)
                    }),
            )
        } else {
            None
        };
        self.set_optimizer_report(LiveRlOptimizerReport {
            executed: true,
            trajectories: trajectory_batch.trajectories.len(),
            samples: samples.len(),
            mean_total_loss: Some(update.mean_loss.total_loss),
            observed_approx_kl: Some(update.observed_approx_kl),
            early_stop_triggered: update.early_stop_triggered,
            apo,
        })
        .await;

        Ok(())
    }

    async fn run_live_apo_update(&self, rollout_ids: &[String]) -> Result<LiveApoReport> {
        let Some(apo_runtime) = self.inner.apo_runtime.clone() else {
            return Ok(LiveApoReport::skipped("apo_missing_runtime", 0));
        };

        let collected_samples = self.collect_live_apo_samples(rollout_ids).await?;
        let (samples, curriculum_focus_category, curriculum_focus_mean_reward) =
            select_curriculum_samples(collected_samples, self.inner.config.apo_max_samples);

        if samples.len() < self.inner.config.apo_min_samples || samples.len() < 2 {
            return Ok(LiveApoReport::skipped_with_curriculum(
                "apo_insufficient_samples",
                samples.len(),
                curriculum_focus_category,
                curriculum_focus_mean_reward,
            ));
        }

        let train_examples = samples
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                PromptExample::new(
                    format!("sample_{}: {}", index + 1, sample.prompt),
                    format!(
                        "reward={:.4}; assistant_response={}",
                        sample.reward, sample.response
                    ),
                )
            })
            .collect::<Vec<_>>();
        let validation_examples = train_examples.clone();
        let seed_prompt = self
            .resolve_live_seed_prompt(apo_runtime.seed_system_prompt.as_str())
            .await?;

        let evaluator = Arc::new(LiveApoPromptEvaluator::new(
            apo_runtime.client.clone(),
            apo_runtime.model.clone(),
        ));
        let algorithm = ApoAlgorithm::new(
            apo_runtime.client.clone(),
            apo_runtime.client.clone(),
            evaluator,
            ApoConfig {
                rounds: 1,
                beam_width: 1,
                candidates_per_parent: 1,
                gradient_model: apo_runtime.model.clone(),
                edit_model: apo_runtime.model.clone(),
                temperature: Some(0.0),
                max_tokens: Some(256),
            },
        );
        let algorithm_store: Arc<dyn TrainingStore> =
            Arc::new(NoResourceWriteStore::new(self.inner.store.clone()));
        let summary = match algorithm
            .run(AlgorithmContext::new(
                algorithm_store,
                seed_prompt,
                train_examples,
                validation_examples,
            ))
            .await
        {
            Ok(summary) => summary,
            Err(error) => {
                return Ok(LiveApoReport::skipped_with_curriculum(
                    format!("apo_run_failed:{error}"),
                    samples.len(),
                    curriculum_focus_category,
                    curriculum_focus_mean_reward,
                ));
            }
        };

        let Some(best_prompt) = summary.best_prompt else {
            return Ok(LiveApoReport::skipped_with_curriculum(
                "apo_no_best_prompt",
                samples.len(),
                curriculum_focus_category,
                curriculum_focus_mean_reward,
            ));
        };
        let best_prompt_version = best_prompt.version.clone();
        let best_prompt_text = best_prompt.prompt.clone();
        let best_prompt_score = best_prompt.score.unwrap_or(0.0).clamp(0.0, 1.0);

        let baseline_scores = samples
            .iter()
            .map(|sample| normalize_reward_to_quality(sample.reward))
            .collect::<Vec<_>>();
        let baseline_mean = mean(baseline_scores.as_slice());
        let delta = best_prompt_score - baseline_mean;
        let candidate_scores = baseline_scores
            .iter()
            .map(|score| (*score + delta).clamp(0.0, 1.0))
            .collect::<Vec<_>>();
        let candidate_mean = mean(candidate_scores.as_slice());

        let significance = match compare_policy_improvement(
            baseline_scores.as_slice(),
            candidate_scores.as_slice(),
            self.inner.config.apo_significance_alpha,
        ) {
            Ok(report) => report,
            Err(error) => {
                return Ok(LiveApoReport::skipped_with_curriculum(
                    format!("apo_significance_failed:{error}"),
                    samples.len(),
                    curriculum_focus_category,
                    curriculum_focus_mean_reward,
                ));
            }
        };

        if !significance.is_significant_improvement || significance.mean_delta <= 0.0 {
            return Ok(LiveApoReport {
                executed: true,
                adopted: false,
                sample_count: samples.len(),
                curriculum_focus_category,
                curriculum_focus_mean_reward,
                baseline_mean_reward: Some(baseline_mean),
                candidate_mean_reward: Some(candidate_mean),
                best_prompt_version: Some(best_prompt_version),
                best_prompt_score: Some(best_prompt_score),
                reason_code: Some("apo_no_significant_improvement".to_string()),
            });
        }

        let mut resources = HashMap::new();
        resources.insert("system_prompt".to_string(), json!(best_prompt_text));
        resources.insert(
            "system_prompt_version".to_string(),
            json!(best_prompt_version.clone()),
        );
        resources.insert("algorithm".to_string(), json!("apo_live_runtime"));
        resources.insert("score".to_string(), json!(best_prompt_score));
        resources.insert(
            "apo_significance_alpha".to_string(),
            json!(self.inner.config.apo_significance_alpha),
        );
        resources.insert(
            "apo_significance_delta_ci_low".to_string(),
            json!(significance.delta_ci_low),
        );
        resources.insert(
            "apo_significance_delta_ci_high".to_string(),
            json!(significance.delta_ci_high),
        );
        resources.insert(
            "apo_samples".to_string(),
            json!(u64::try_from(samples.len()).unwrap_or(0)),
        );
        if let Some(category) = curriculum_focus_category.as_ref() {
            resources.insert("apo_curriculum_focus_category".to_string(), json!(category));
        }
        if let Some(mean_reward) = curriculum_focus_mean_reward {
            resources.insert(
                "apo_curriculum_focus_mean_reward".to_string(),
                json!(mean_reward),
            );
        }

        self.inner
            .store
            .update_resources(resources)
            .await
            .context("failed to persist live APO prompt adoption")?;

        Ok(LiveApoReport {
            executed: true,
            adopted: true,
            sample_count: samples.len(),
            curriculum_focus_category,
            curriculum_focus_mean_reward,
            baseline_mean_reward: Some(baseline_mean),
            candidate_mean_reward: Some(candidate_mean),
            best_prompt_version: Some(best_prompt_version),
            best_prompt_score: Some(best_prompt_score),
            reason_code: Some("apo_adopted".to_string()),
        })
    }

    async fn resolve_live_seed_prompt(&self, fallback: &str) -> Result<String> {
        let latest = self
            .inner
            .store
            .get_latest_resources()
            .await
            .context("failed to read latest resources for APO seed prompt")?;
        if let Some(resources) = latest {
            if let Some(system_prompt) = resources
                .resources
                .get("system_prompt")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|prompt| !prompt.is_empty())
            {
                return Ok(system_prompt.to_string());
            }
        }
        Ok(fallback.to_string())
    }

    async fn collect_live_apo_samples(&self, rollout_ids: &[String]) -> Result<Vec<LiveApoSample>> {
        let mut samples = Vec::new();
        for rollout_id in rollout_ids {
            let spans = self
                .inner
                .store
                .query_spans(rollout_id, None)
                .await
                .with_context(|| format!("failed to query spans for APO rollout '{rollout_id}'"))?;
            let decision = spans
                .into_iter()
                .filter(|span| span.name == "live.agent.decision")
                .max_by_key(|span| span.sequence_id);
            let Some(span) = decision else {
                continue;
            };

            let prompt = span
                .attributes
                .get("prompt")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            let response = span
                .attributes
                .get("assistant_text")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            let reward = span
                .attributes
                .get("reward")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            if prompt.is_empty() || response.is_empty() || !reward.is_finite() {
                continue;
            }
            let category = span
                .attributes
                .get("task_category")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .unwrap_or_else(|| infer_task_category(prompt));

            samples.push(LiveApoSample {
                prompt: prompt.to_string(),
                response: response.to_string(),
                reward,
                category,
            });
        }
        Ok(samples)
    }

    async fn collect_live_rollout_ids_for_update(&self) -> Result<Vec<String>> {
        let rollouts = self
            .inner
            .store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .context("failed to query succeeded live rollouts")?;

        let mut rollout_ids = rollouts
            .into_iter()
            .filter(|rollout| rollout.rollout_id.starts_with(LIVE_ROLLOUT_PREFIX))
            .map(|rollout| rollout.rollout_id)
            .collect::<Vec<_>>();

        rollout_ids.sort();
        if rollout_ids.len() > self.inner.config.max_rollouts_per_update {
            let start = rollout_ids.len() - self.inner.config.max_rollouts_per_update;
            rollout_ids = rollout_ids[start..].to_vec();
        }

        Ok(rollout_ids)
    }

    async fn set_optimizer_report(&self, report: LiveRlOptimizerReport) {
        let mut state = self.inner.state.lock().await;
        state.last_optimizer_report = Some(report);
    }

    async fn register_failure(&self, message: String) {
        let mut state = self.inner.state.lock().await;
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        state.last_error = Some(message);
        if state.consecutive_failures >= self.inner.config.max_failure_streak {
            state.gate = LiveRlRuntimeGate::Hold;
        }
    }

    #[cfg(test)]
    pub(crate) fn for_tests(
        store: Arc<dyn TrainingStore + Send + Sync>,
        config: LiveRlRuntimeConfig,
    ) -> Self {
        Self::new(store, config, None)
    }

    #[cfg(test)]
    pub(crate) fn for_tests_with_apo(
        store: Arc<dyn TrainingStore + Send + Sync>,
        config: LiveRlRuntimeConfig,
        client: Arc<dyn LlmClient>,
        seed_system_prompt: &str,
    ) -> Self {
        let apo_runtime = config.apo_enabled.then_some(LiveApoRuntime {
            client,
            model: "gpt-4o-mini".to_string(),
            seed_system_prompt: seed_system_prompt.to_string(),
        });
        Self::new(store, config, apo_runtime)
    }

    #[cfg(test)]
    pub(crate) async fn record_failure_for_tests(&self, message: &str) {
        self.register_failure(message.to_string()).await;
    }
}

impl LiveApoReport {
    fn skipped(reason_code: impl Into<String>, sample_count: usize) -> Self {
        Self {
            executed: false,
            adopted: false,
            sample_count,
            curriculum_focus_category: None,
            curriculum_focus_mean_reward: None,
            baseline_mean_reward: None,
            candidate_mean_reward: None,
            best_prompt_version: None,
            best_prompt_score: None,
            reason_code: Some(reason_code.into()),
        }
    }

    fn skipped_with_curriculum(
        reason_code: impl Into<String>,
        sample_count: usize,
        curriculum_focus_category: Option<String>,
        curriculum_focus_mean_reward: Option<f64>,
    ) -> Self {
        Self {
            executed: false,
            adopted: false,
            sample_count,
            curriculum_focus_category,
            curriculum_focus_mean_reward,
            baseline_mean_reward: None,
            candidate_mean_reward: None,
            best_prompt_version: None,
            best_prompt_score: None,
            reason_code: Some(reason_code.into()),
        }
    }
}

struct LiveApoPromptEvaluator {
    client: Arc<dyn LlmClient>,
    model: String,
}

impl LiveApoPromptEvaluator {
    fn new(client: Arc<dyn LlmClient>, model: String) -> Self {
        Self { client, model }
    }
}

#[async_trait]
impl PromptEvaluator for LiveApoPromptEvaluator {
    async fn score_prompt(&self, prompt: &str, dataset: &[PromptExample]) -> Result<f64> {
        let rendered_examples = if dataset.is_empty() {
            "(no examples)".to_string()
        } else {
            dataset
                .iter()
                .take(8)
                .enumerate()
                .map(|(index, example)| {
                    format!(
                        "{}. input={} expected={}",
                        index + 1,
                        example.input,
                        example.expected
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message::system(
                    "Score agent system prompts for expected task quality. Return JSON: {\"score\": <0..1>} only.",
                ),
                Message::user(format!(
                    "Prompt:\n{prompt}\n\nExamples:\n{rendered_examples}\n\nReturn JSON score."
                )),
            ],
            tools: Vec::new(),
            tool_choice: None,
            json_mode: true,
            max_tokens: Some(64),
            temperature: Some(0.0),
            prompt_cache: Default::default(),
        };

        let llm_score = self
            .client
            .complete(request)
            .await
            .ok()
            .and_then(|response| parse_score_from_text(&response.message.text_content()));
        Ok(llm_score.unwrap_or_else(|| fallback_prompt_score(prompt, dataset)))
    }
}

fn parse_score_from_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(score) = value.get("score").and_then(Value::as_f64) {
            return Some(score.clamp(0.0, 1.0));
        }
    }

    if let Ok(score) = trimmed.parse::<f64>() {
        return Some(score.clamp(0.0, 1.0));
    }

    for token in trimmed.split(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-')) {
        if token.is_empty() || token == "-" || token == "." {
            continue;
        }
        if let Ok(score) = token.parse::<f64>() {
            return Some(score.clamp(0.0, 1.0));
        }
    }

    None
}

fn fallback_prompt_score(prompt: &str, dataset: &[PromptExample]) -> f64 {
    let normalized_prompt = prompt.to_ascii_lowercase();
    let keyword_hits = [
        "verify",
        "deterministic",
        "concise",
        "safe",
        "error",
        "tool",
        "plan",
    ]
    .iter()
    .filter(|keyword| normalized_prompt.contains(**keyword))
    .count() as f64;
    let keyword_score = (keyword_hits / 7.0).clamp(0.0, 1.0);

    let length_score = ((prompt.chars().count() as f64) / 300.0).clamp(0.0, 1.0);
    let dataset_score = if dataset.is_empty() {
        0.0
    } else {
        (dataset.len() as f64 / 8.0).clamp(0.0, 1.0)
    };

    (0.3 + 0.4 * keyword_score + 0.2 * length_score + 0.1 * dataset_score).clamp(0.0, 1.0)
}

fn normalize_reward_to_quality(reward: f64) -> f64 {
    ((reward + 1.0) / 2.0).clamp(0.0, 1.0)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn infer_task_category(prompt: &str) -> String {
    let normalized = prompt.to_ascii_lowercase();
    let category = if [
        "debug", "fix", "error", "panic", "trace", "failure", "flaky",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
    {
        "debugging"
    } else if ["refactor", "cleanup", "rename", "extract"]
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        "refactoring"
    } else if ["implement", "build", "create", "feature", "write code"]
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        "code_generation"
    } else if ["plan", "roadmap", "milestone", "spec", "tasks"]
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        "planning"
    } else if ["deploy", "release", "incident", "runbook", "ops"]
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        "operations"
    } else if ["why", "what", "summarize", "explain", "status", "question"]
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        "qa"
    } else {
        "general"
    };
    category.to_string()
}

fn parse_live_category_outcome(span: &TrainingSpan) -> Option<LiveCategoryOutcome> {
    let reward = span
        .attributes
        .get("reward")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())?;
    let prompt = span
        .attributes
        .get("prompt")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    let category = span
        .attributes
        .get("task_category")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| infer_task_category(prompt));
    let predicted_success_probability = span
        .attributes
        .get("predicted_success_probability")
        .and_then(Value::as_f64)
        .or_else(|| {
            span.attributes
                .get("reward_confidence")
                .and_then(Value::as_f64)
        })
        .unwrap_or_else(|| normalize_reward_to_quality(reward))
        .clamp(0.0, 1.0);
    let actual_success = span
        .attributes
        .get("actual_success")
        .and_then(Value::as_bool)
        .unwrap_or(reward > 0.0);

    Some(LiveCategoryOutcome {
        category,
        reward,
        predicted_success_probability,
        actual_success,
    })
}

fn classify_learning_trend(rewards: &[f64]) -> &'static str {
    if rewards.len() < 4 {
        return "insufficient_data";
    }
    let window = (rewards.len() / 2).max(2);
    if rewards.len() <= window {
        return "insufficient_data";
    }
    let split = rewards.len() - window;
    let prior_mean = mean(&rewards[..split]);
    let recent_mean = mean(&rewards[split..]);
    let delta = recent_mean - prior_mean;
    if delta <= -0.15 {
        "regressing"
    } else if delta >= 0.10 {
        "improving"
    } else {
        "plateau"
    }
}

fn select_curriculum_samples(
    samples: Vec<LiveApoSample>,
    max_samples: usize,
) -> (Vec<LiveApoSample>, Option<String>, Option<f64>) {
    if samples.is_empty() || max_samples == 0 {
        return (Vec::new(), None, None);
    }

    let mut grouped = BTreeMap::<String, Vec<LiveApoSample>>::new();
    for sample in samples {
        grouped
            .entry(sample.category.clone())
            .or_default()
            .push(sample);
    }

    let mut category_means = grouped
        .iter()
        .map(|(category, grouped_samples)| {
            let mean_reward = mean(
                grouped_samples
                    .iter()
                    .map(|sample| sample.reward)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            (category.clone(), mean_reward)
        })
        .collect::<Vec<_>>();
    category_means.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    let curriculum_focus_category = category_means.first().map(|entry| entry.0.clone());
    let curriculum_focus_mean_reward = category_means.first().map(|entry| entry.1);

    let total = grouped.values().map(Vec::len).sum::<usize>();
    if total <= max_samples {
        let mut selected = Vec::new();
        for grouped_samples in grouped.into_values() {
            selected.extend(grouped_samples);
        }
        return (
            selected,
            curriculum_focus_category,
            curriculum_focus_mean_reward,
        );
    }

    let mut selected = Vec::with_capacity(max_samples);
    while selected.len() < max_samples {
        let mut progressed = false;
        for (category, _) in &category_means {
            if selected.len() == max_samples {
                break;
            }
            if let Some(grouped_samples) = grouped.get_mut(category) {
                if let Some(sample) = grouped_samples.pop() {
                    selected.push(sample);
                    progressed = true;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    selected.reverse();

    (
        selected,
        curriculum_focus_category,
        curriculum_focus_mean_reward,
    )
}

fn build_final_decision_span(run: &LiveRlActiveRun) -> TrainingSpan {
    let reward = compute_live_reward_breakdown(run);
    let prompt = run.prompt.clone().unwrap_or_default();
    let task_category = infer_task_category(prompt.as_str());
    let predicted_success_probability = reward.confidence.clamp(0.0, 1.0);
    let actual_success = reward.composite > 0.0;
    let confidence_calibration_error =
        (predicted_success_probability - if actual_success { 1.0 } else { 0.0 }).abs();
    let mut span = TrainingSpan::new(
        run.rollout_id.as_str(),
        run.attempt_id.as_str(),
        1,
        format!("trace:{}", run.rollout_id),
        format!("span:{}:1", run.rollout_id),
        None,
        "live.agent.decision",
    );
    span.attributes.insert("prompt".to_string(), json!(prompt));
    span.attributes.insert(
        "assistant_text".to_string(),
        json!(run.assistant_reply.clone().unwrap_or_default()),
    );
    span.attributes
        .insert("task_category".to_string(), json!(task_category));
    span.attributes
        .insert("reward".to_string(), json!(reward.composite));
    span.attributes
        .insert("reward_completion".to_string(), json!(reward.completion));
    span.attributes.insert(
        "reward_session_completion".to_string(),
        json!(reward.session_completion),
    );
    span.attributes
        .insert("reward_reliability".to_string(), json!(reward.reliability));
    span.attributes
        .insert("reward_safety".to_string(), json!(reward.safety));
    span.attributes
        .insert("reward_efficiency".to_string(), json!(reward.efficiency));
    span.attributes.insert(
        "reward_token_efficiency".to_string(),
        json!(reward.token_efficiency),
    );
    span.attributes
        .insert("reward_confidence".to_string(), json!(reward.confidence));
    span.attributes.insert(
        "predicted_success_probability".to_string(),
        json!(predicted_success_probability),
    );
    span.attributes
        .insert("actual_success".to_string(), json!(actual_success));
    span.attributes.insert(
        "confidence_calibration_error".to_string(),
        json!(confidence_calibration_error),
    );
    span.attributes
        .insert("ask_for_help_recommended".to_string(), json!(false));
    span.attributes
        .insert("learning_trend".to_string(), json!("insufficient_data"));
    span.attributes
        .insert("turns".to_string(), json!(run.turns));
    span.attributes
        .insert("tool_errors".to_string(), json!(run.tool_errors));
    span.attributes
        .insert("safety_blocked".to_string(), json!(run.safety_blocked));
    span.attributes.insert("done".to_string(), json!(true));
    span.end_time = Some(Utc::now());
    span
}

#[cfg(test)]
fn compute_live_reward(run: &LiveRlActiveRun) -> f64 {
    compute_live_reward_breakdown(run).composite
}

fn compute_live_reward_breakdown(run: &LiveRlActiveRun) -> RewardInferenceOutput {
    let has_assistant_reply = run
        .assistant_reply
        .as_ref()
        .is_some_and(|reply| !reply.trim().is_empty());
    let input = RewardInferenceInput::new(
        has_assistant_reply,
        has_assistant_reply,
        run.tool_errors,
        run.safety_blocked,
        run.turns,
        run.prompt
            .as_ref()
            .map_or(0, |prompt| prompt.chars().count()),
        run.assistant_reply
            .as_ref()
            .map_or(0, |reply| reply.chars().count()),
    );
    TraceBasedRewardInference.infer(&input)
}

fn parse_bool_env(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_positive_usize_env(raw: Option<&str>, default: usize, key: &str) -> Result<usize> {
    let Some(raw) = raw else {
        return Ok(default);
    };
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Ok(default);
    }
    let value = normalized
        .parse::<usize>()
        .with_context(|| format!("{key} must be a positive integer"))?;
    if value == 0 {
        return Err(anyhow!("{key} must be greater than 0"));
    }
    Ok(value)
}

fn parse_significance_alpha_env(raw: Option<&str>, default: f64, key: &str) -> Result<f64> {
    let Some(raw) = raw else {
        return Ok(default);
    };
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Ok(default);
    }
    let value = normalized
        .parse::<f64>()
        .with_context(|| format!("{key} must be a floating-point alpha value"))?;
    if !value.is_finite() {
        return Err(anyhow!("{key} must be finite"));
    }
    let supported = [0.10_f64, 0.05_f64, 0.01_f64];
    if supported
        .iter()
        .any(|candidate| (value - candidate).abs() < 1e-12)
    {
        Ok(value)
    } else {
        Err(anyhow!(
            "{key} must be one of 0.10, 0.05, 0.01 (supported by significance engine)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_live_reward, LiveRlActiveRun, LiveRlRuntimeBridge, LiveRlRuntimeConfig,
        LiveRlRuntimeGate,
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::collections::VecDeque;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tau_agent_core::AgentEvent;
    use tau_ai::{ChatRequest, ChatResponse, ChatUsage, LlmClient, Message, TauAiError};
    use tau_training_store::{
        InMemoryTrainingStore, Rollout, RolloutQuery, RolloutStatus, TrainingSpan, TrainingStore,
    };

    #[derive(Clone)]
    struct ScriptedClient {
        outputs: Arc<Mutex<VecDeque<String>>>,
    }

    impl ScriptedClient {
        fn new(lines: Vec<&str>) -> Self {
            Self {
                outputs: Arc::new(Mutex::new(
                    lines
                        .into_iter()
                        .map(ToString::to_string)
                        .collect::<VecDeque<_>>(),
                )),
            }
        }
    }

    #[async_trait]
    impl LlmClient for ScriptedClient {
        async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
            let mut outputs = self.outputs.lock().expect("scripted client mutex poisoned");
            let text = outputs
                .pop_front()
                .unwrap_or_else(|| "fallback output".to_string());
            Ok(ChatResponse {
                message: Message::assistant_text(text),
                finish_reason: Some("stop".to_string()),
                usage: ChatUsage::default(),
            })
        }
    }

    async fn seed_live_rollout(
        store: &Arc<dyn TrainingStore + Send + Sync>,
        rollout_id: &str,
        reward: f64,
    ) {
        let mut rollout = Rollout::new(rollout_id.to_string(), json!({"source":"seed"}), None);
        rollout
            .metadata
            .insert("source".to_string(), json!("seeded_test"));
        store
            .enqueue_rollout(rollout)
            .await
            .expect("enqueue seeded rollout");
        store
            .update_rollout_status(rollout_id, RolloutStatus::Running)
            .await
            .expect("mark seeded rollout running");
        store
            .update_rollout_status(rollout_id, RolloutStatus::Succeeded)
            .await
            .expect("mark seeded rollout succeeded");

        let mut span = TrainingSpan::new(
            rollout_id,
            &format!("{rollout_id}:attempt-live"),
            1,
            format!("trace:{rollout_id}"),
            format!("span:{rollout_id}:1"),
            None,
            "live.agent.decision",
        );
        span.attributes
            .insert("prompt".to_string(), json!("seeded prompt"));
        span.attributes
            .insert("assistant_text".to_string(), json!("seeded response"));
        span.attributes.insert("reward".to_string(), json!(reward));
        span.attributes.insert("done".to_string(), json!(true));
        span.end_time = Some(Utc::now());
        store.add_span(span).await.expect("add seeded span");
    }

    async fn seed_live_rollouts(
        store: &Arc<dyn TrainingStore + Send + Sync>,
        id_prefix: &str,
        rewards: &[f64],
    ) -> Vec<String> {
        let mut rollout_ids = Vec::with_capacity(rewards.len());
        for (index, reward) in rewards.iter().copied().enumerate() {
            let rollout_id = format!("live-rl-rollout-{id_prefix}-{index:04}");
            seed_live_rollout(store, rollout_id.as_str(), reward).await;
            rollout_ids.push(rollout_id);
        }
        rollout_ids
    }

    async fn seed_live_rollout_with_category(
        store: &Arc<dyn TrainingStore + Send + Sync>,
        rollout_id: &str,
        prompt: &str,
        response: &str,
        reward: f64,
        category: &str,
    ) {
        let mut rollout = Rollout::new(rollout_id.to_string(), json!({"source":"seed"}), None);
        rollout
            .metadata
            .insert("source".to_string(), json!("seeded_test"));
        store
            .enqueue_rollout(rollout)
            .await
            .expect("enqueue seeded rollout");
        store
            .update_rollout_status(rollout_id, RolloutStatus::Running)
            .await
            .expect("mark seeded rollout running");
        store
            .update_rollout_status(rollout_id, RolloutStatus::Succeeded)
            .await
            .expect("mark seeded rollout succeeded");

        let mut span = TrainingSpan::new(
            rollout_id,
            &format!("{rollout_id}:attempt-live"),
            1,
            format!("trace:{rollout_id}"),
            format!("span:{rollout_id}:1"),
            None,
            "live.agent.decision",
        );
        span.attributes.insert("prompt".to_string(), json!(prompt));
        span.attributes
            .insert("assistant_text".to_string(), json!(response));
        span.attributes.insert("reward".to_string(), json!(reward));
        span.attributes
            .insert("task_category".to_string(), json!(category));
        span.attributes.insert("done".to_string(), json!(true));
        span.end_time = Some(Utc::now());
        store.add_span(span).await.expect("add seeded span");
    }

    #[test]
    fn spec_c04_unit_live_rl_env_defaults_to_disabled() {
        let env = BTreeMap::new();
        let config = LiveRlRuntimeConfig::from_env_map(
            &env,
            std::path::Path::new(".tau/training/store.sqlite"),
        )
        .expect("config from env");
        assert!(!config.enabled);
        assert_eq!(config.update_interval_rollouts, 8);
        assert_eq!(config.max_rollouts_per_update, 64);
        assert_eq!(config.max_failure_streak, 3);
        assert!(config.apo_enabled);
        assert_eq!(config.apo_min_samples, 4);
        assert_eq!(config.apo_max_samples, 32);
        assert!((config.apo_significance_alpha - 0.05).abs() < 1e-12);
    }

    #[tokio::test]
    async fn spec_c01_functional_live_events_persist_rollout_and_span() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let bridge = LiveRlRuntimeBridge::for_tests(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 8,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("summarize latest deploy status"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("Deploy completed with no failures."),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let rollouts = store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .expect("query succeeded rollouts");
        assert_eq!(rollouts.len(), 1);
        assert!(rollouts[0].rollout_id.starts_with("live-rl-rollout"));

        let spans = store
            .query_spans(rollouts[0].rollout_id.as_str(), None)
            .await
            .expect("query spans");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "live.agent.decision");
        assert_eq!(spans[0].attributes["reward"], serde_json::json!(1.0));
    }

    #[tokio::test]
    async fn spec_c02_functional_optimizer_runs_on_update_interval() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let bridge = LiveRlRuntimeBridge::for_tests(
            store,
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("draft release notes"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("Release notes drafted."),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let snapshot = bridge.snapshot().await;
        let report = snapshot
            .last_optimizer_report
            .expect("optimizer report should be present");
        assert!(report.executed);
        assert!(report.samples > 0);
        assert_eq!(snapshot.completed_rollouts, 1);
    }

    #[tokio::test]
    async fn spec_c03_regression_failure_streak_holds_live_gate() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let bridge = LiveRlRuntimeBridge::for_tests(
            store,
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 4,
                max_rollouts_per_update: 32,
                max_failure_streak: 1,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.record_failure_for_tests("forced failure").await;
        let snapshot = bridge.snapshot().await;
        assert_eq!(snapshot.gate, LiveRlRuntimeGate::Hold);
        assert_eq!(snapshot.consecutive_failures, 1);
        assert_eq!(snapshot.last_error.as_deref(), Some("forced failure"));
    }

    #[test]
    fn spec_c05_unit_live_reward_breakdown_scores_deterministically() {
        let run = LiveRlActiveRun {
            rollout_id: "live-rl-rollout-1".to_string(),
            attempt_id: "live-rl-rollout-1:attempt-1".to_string(),
            prompt: Some("plan".to_string()),
            assistant_reply: Some("done".to_string()),
            turns: 1,
            tool_errors: 0,
            safety_blocked: false,
        };
        assert_eq!(compute_live_reward(&run), 1.0);

        let noisy = LiveRlActiveRun {
            tool_errors: 2,
            ..run.clone()
        };
        assert_eq!(compute_live_reward(&noisy), 0.75);

        let no_reply = LiveRlActiveRun {
            assistant_reply: None,
            turns: 4,
            ..run.clone()
        };
        assert_eq!(compute_live_reward(&no_reply), 0.0);

        let blocked = LiveRlActiveRun {
            safety_blocked: true,
            ..run
        };
        assert_eq!(compute_live_reward(&blocked), -1.0);
    }

    #[tokio::test]
    async fn spec_c06_functional_live_rollout_span_persists_reward_breakdown() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let bridge = LiveRlRuntimeBridge::for_tests(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 8,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("summarize latest deploy status"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("Deploy completed with no failures."),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let rollouts = store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .expect("query succeeded rollouts");
        assert_eq!(rollouts.len(), 1);

        let spans = store
            .query_spans(rollouts[0].rollout_id.as_str(), None)
            .await
            .expect("query spans");
        assert_eq!(spans.len(), 1);
        let attrs = &spans[0].attributes;
        assert!(attrs.contains_key("reward_completion"));
        assert!(attrs.contains_key("reward_reliability"));
        assert!(attrs.contains_key("reward_safety"));
        assert!(attrs.contains_key("reward_efficiency"));
        assert!(attrs.contains_key("reward_confidence"));
        assert!(attrs.contains_key("reward_session_completion"));
        assert!(attrs.contains_key("reward_token_efficiency"));
    }

    #[tokio::test]
    async fn spec_c07_functional_live_optimizer_runs_apo_and_persists_prompt_resources() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        seed_live_rollout(&store, "live-rl-rollout-0000000101", -0.90).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000102", -0.85).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000103", -0.80).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000104", -0.75).await;

        let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: true,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
            Arc::new(ScriptedClient::new(vec![
                "{\"score\":0.12}",
                "Add deterministic verification and concise plan-first structure.",
                "You are Tau. Verify outcomes, be concise, and include deterministic checks.",
                "{\"score\":0.95}",
            ])),
            "You are Tau.",
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("status"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("ok"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let snapshot = bridge.snapshot().await;
        let report = snapshot
            .last_optimizer_report
            .expect("optimizer report should be present");
        let apo = report.apo.expect("apo report should be present");
        assert!(apo.executed);
        assert!(apo.adopted);
        assert_eq!(apo.reason_code.as_deref(), Some("apo_adopted"));

        let latest = store
            .get_latest_resources()
            .await
            .expect("get latest resources")
            .expect("resources should exist after adoption");
        let persisted_prompt = latest
            .resources
            .get("system_prompt")
            .and_then(serde_json::Value::as_str)
            .expect("system_prompt should be persisted");
        assert!(persisted_prompt.contains("deterministic checks"));
    }

    #[tokio::test]
    async fn spec_c08_regression_live_apo_skips_adoption_without_significant_improvement() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        seed_live_rollout(&store, "live-rl-rollout-0000000011", 0.52).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000012", 0.54).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000013", 0.53).await;
        seed_live_rollout(&store, "live-rl-rollout-0000000014", 0.55).await;

        let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: true,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
            Arc::new(ScriptedClient::new(vec![
                "{\"score\":0.53}",
                "Tiny wording refinement.",
                "You are Tau.",
                "{\"score\":0.53}",
            ])),
            "You are Tau.",
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("status"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("ok"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let snapshot = bridge.snapshot().await;
        let report = snapshot
            .last_optimizer_report
            .expect("optimizer report should be present");
        let apo = report.apo.expect("apo report should be present");
        assert!(apo.executed);
        assert!(!apo.adopted);
        assert_eq!(
            apo.reason_code.as_deref(),
            Some("apo_no_significant_improvement")
        );

        let latest = store
            .get_latest_resources()
            .await
            .expect("get latest resources");
        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn spec_c09_regression_live_apo_caps_samples_to_max_window() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let rollout_ids = seed_live_rollouts(
            &store,
            "c09",
            &[-1.0, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3],
        )
        .await;

        let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
            store,
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: true,
                apo_min_samples: 2,
                apo_max_samples: 4,
                apo_significance_alpha: 0.05,
            },
            Arc::new(ScriptedClient::new(vec![
                "{\"score\":0.40}",
                "Make the prompt concise and checklist-oriented.",
                "You are Tau. Keep outputs concise with deterministic checks.",
                "{\"score\":0.70}",
            ])),
            "You are Tau.",
        );

        let report = bridge
            .run_live_apo_update(rollout_ids.as_slice())
            .await
            .expect("run APO update");
        assert!(report.executed);
        assert_eq!(report.sample_count, 4);
    }

    #[tokio::test]
    async fn spec_c10_regression_live_apo_sample_thresholds_respect_min_and_hard_floor() {
        {
            let store: Arc<dyn TrainingStore + Send + Sync> =
                Arc::new(InMemoryTrainingStore::new());
            let rollout_ids = seed_live_rollouts(&store, "c10-min3", &[0.20, 0.30, 0.40]).await;
            let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
                store,
                LiveRlRuntimeConfig {
                    enabled: true,
                    store_path: ".tau/training/store.sqlite".into(),
                    update_interval_rollouts: 1,
                    max_rollouts_per_update: 32,
                    max_failure_streak: 3,
                    apo_enabled: true,
                    apo_min_samples: 4,
                    apo_max_samples: 32,
                    apo_significance_alpha: 0.05,
                },
                Arc::new(ScriptedClient::new(vec![
                    "{\"score\":0.50}",
                    "n/a",
                    "n/a",
                    "{\"score\":0.50}",
                ])),
                "You are Tau.",
            );
            let report = bridge
                .run_live_apo_update(rollout_ids.as_slice())
                .await
                .expect("run APO update");
            assert!(!report.executed);
            assert_eq!(report.sample_count, 3);
            assert_eq!(
                report.reason_code.as_deref(),
                Some("apo_insufficient_samples")
            );
        }

        {
            let store: Arc<dyn TrainingStore + Send + Sync> =
                Arc::new(InMemoryTrainingStore::new());
            let rollout_ids =
                seed_live_rollouts(&store, "c10-min4", &[0.20, 0.30, 0.40, 0.50]).await;
            let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
                store,
                LiveRlRuntimeConfig {
                    enabled: true,
                    store_path: ".tau/training/store.sqlite".into(),
                    update_interval_rollouts: 1,
                    max_rollouts_per_update: 32,
                    max_failure_streak: 3,
                    apo_enabled: true,
                    apo_min_samples: 4,
                    apo_max_samples: 32,
                    apo_significance_alpha: 0.05,
                },
                Arc::new(ScriptedClient::new(vec![
                    "{\"score\":0.45}",
                    "Minor wording update.",
                    "You are Tau. Verify and summarize clearly.",
                    "{\"score\":0.46}",
                ])),
                "You are Tau.",
            );
            let report = bridge
                .run_live_apo_update(rollout_ids.as_slice())
                .await
                .expect("run APO update");
            assert_eq!(report.sample_count, 4);
            assert_ne!(
                report.reason_code.as_deref(),
                Some("apo_insufficient_samples")
            );
        }

        {
            let store: Arc<dyn TrainingStore + Send + Sync> =
                Arc::new(InMemoryTrainingStore::new());
            let rollout_ids = seed_live_rollouts(&store, "c10-hard1", &[0.40]).await;
            let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
                store,
                LiveRlRuntimeConfig {
                    enabled: true,
                    store_path: ".tau/training/store.sqlite".into(),
                    update_interval_rollouts: 1,
                    max_rollouts_per_update: 32,
                    max_failure_streak: 3,
                    apo_enabled: true,
                    apo_min_samples: 1,
                    apo_max_samples: 32,
                    apo_significance_alpha: 0.05,
                },
                Arc::new(ScriptedClient::new(vec![
                    "{\"score\":0.50}",
                    "n/a",
                    "n/a",
                    "{\"score\":0.51}",
                ])),
                "You are Tau.",
            );
            let report = bridge
                .run_live_apo_update(rollout_ids.as_slice())
                .await
                .expect("run APO update");
            assert!(!report.executed);
            assert_eq!(report.sample_count, 1);
            assert_eq!(
                report.reason_code.as_deref(),
                Some("apo_insufficient_samples")
            );
        }

        {
            let store: Arc<dyn TrainingStore + Send + Sync> =
                Arc::new(InMemoryTrainingStore::new());
            let rollout_ids = seed_live_rollouts(&store, "c10-hard2", &[0.40, 0.50]).await;
            let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
                store,
                LiveRlRuntimeConfig {
                    enabled: true,
                    store_path: ".tau/training/store.sqlite".into(),
                    update_interval_rollouts: 1,
                    max_rollouts_per_update: 32,
                    max_failure_streak: 3,
                    apo_enabled: true,
                    apo_min_samples: 1,
                    apo_max_samples: 32,
                    apo_significance_alpha: 0.05,
                },
                Arc::new(ScriptedClient::new(vec![
                    "{\"score\":0.45}",
                    "Tiny wording refinement.",
                    "You are Tau. Keep deterministic checks.",
                    "{\"score\":0.46}",
                ])),
                "You are Tau.",
            );
            let report = bridge
                .run_live_apo_update(rollout_ids.as_slice())
                .await
                .expect("run APO update");
            assert_eq!(report.sample_count, 2);
            assert_ne!(
                report.reason_code.as_deref(),
                Some("apo_insufficient_samples")
            );
        }
    }

    #[tokio::test]
    async fn spec_c11_regression_live_apo_rejects_non_significant_positive_delta() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let rollout_ids = seed_live_rollouts(&store, "c11", &[-1.0, -1.0, 1.0, 1.0]).await;

        let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: true,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
            Arc::new(ScriptedClient::new(vec![
                "{\"score\":0.50}",
                "Add a small wording tweak.",
                "You are Tau. Prefer deterministic checks and concise plans.",
                "{\"score\":0.60}",
            ])),
            "You are Tau.",
        );

        let report = bridge
            .run_live_apo_update(rollout_ids.as_slice())
            .await
            .expect("run APO update");
        assert!(report.executed);
        assert!(!report.adopted);
        assert_eq!(
            report.reason_code.as_deref(),
            Some("apo_no_significant_improvement")
        );
        assert!(
            report.candidate_mean_reward.expect("candidate mean reward")
                > report.baseline_mean_reward.expect("baseline mean reward")
        );

        let latest = store
            .get_latest_resources()
            .await
            .expect("read latest resources");
        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn spec_c12_functional_live_span_persists_meta_cognition_fields() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let bridge = LiveRlRuntimeBridge::for_tests(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 8,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("debug why this parser test fails intermittently"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("I found and fixed the flaky parser branch."),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let rollouts = store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .expect("query succeeded rollouts");
        assert_eq!(rollouts.len(), 1);

        let spans = store
            .query_spans(rollouts[0].rollout_id.as_str(), None)
            .await
            .expect("query spans");
        assert_eq!(spans.len(), 1);
        let attrs = &spans[0].attributes;
        assert!(attrs.contains_key("task_category"));
        assert!(attrs.contains_key("predicted_success_probability"));
        assert!(attrs.contains_key("actual_success"));
        assert!(attrs.contains_key("confidence_calibration_error"));
        assert!(attrs.contains_key("ask_for_help_recommended"));
        assert!(attrs.contains_key("learning_trend"));
    }

    #[tokio::test]
    async fn spec_c13_regression_live_apo_curriculum_prioritizes_weak_categories() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0000",
            "debug failing auth flow",
            "debug fix 1",
            -1.0,
            "debugging",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0001",
            "debug memory corruption",
            "debug fix 2",
            -0.9,
            "debugging",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0002",
            "summarize status report",
            "status output",
            -0.4,
            "qa",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0003",
            "summarize status report",
            "status output",
            -0.3,
            "qa",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0004",
            "summarize status report",
            "status output",
            -0.2,
            "qa",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c13-0005",
            "summarize status report",
            "status output",
            -0.1,
            "qa",
        )
        .await;
        let rollout_ids = vec![
            "live-rl-rollout-c13-0000".to_string(),
            "live-rl-rollout-c13-0001".to_string(),
            "live-rl-rollout-c13-0002".to_string(),
            "live-rl-rollout-c13-0003".to_string(),
            "live-rl-rollout-c13-0004".to_string(),
            "live-rl-rollout-c13-0005".to_string(),
        ];

        let bridge = LiveRlRuntimeBridge::for_tests_with_apo(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 1,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: true,
                apo_min_samples: 2,
                apo_max_samples: 4,
                apo_significance_alpha: 0.05,
            },
            Arc::new(ScriptedClient::new(vec![
                "{\"score\":0.10}",
                "Strengthen deterministic debug guidance and verification steps.",
                "You are Tau. Prioritize deterministic debugging checks and concise verification.",
                "{\"score\":0.98}",
            ])),
            "You are Tau.",
        );

        let report = bridge
            .run_live_apo_update(rollout_ids.as_slice())
            .await
            .expect("run APO update");
        assert!(report.executed);
        assert!(report.adopted);

        let resources = store
            .get_latest_resources()
            .await
            .expect("read resources")
            .expect("resources should be persisted");
        assert_eq!(
            resources
                .resources
                .get("apo_curriculum_focus_category")
                .and_then(serde_json::Value::as_str),
            Some("debugging")
        );
    }

    #[tokio::test]
    async fn spec_c14_regression_live_span_learning_trend_regressing_when_recent_reward_drops() {
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c14-0000",
            "debug parser",
            "baseline success",
            0.8,
            "debugging",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c14-0001",
            "debug parser",
            "baseline success",
            0.7,
            "debugging",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c14-0002",
            "debug parser",
            "recent failure",
            -0.4,
            "debugging",
        )
        .await;
        seed_live_rollout_with_category(
            &store,
            "live-rl-rollout-c14-0003",
            "debug parser",
            "recent failure",
            -0.5,
            "debugging",
        )
        .await;

        let bridge = LiveRlRuntimeBridge::for_tests(
            store.clone(),
            LiveRlRuntimeConfig {
                enabled: true,
                store_path: ".tau/training/store.sqlite".into(),
                update_interval_rollouts: 8,
                max_rollouts_per_update: 32,
                max_failure_streak: 3,
                apo_enabled: false,
                apo_min_samples: 4,
                apo_max_samples: 32,
                apo_significance_alpha: 0.05,
            },
        );

        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user("debug parser mismatch in integration tests"),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text("I found the mismatch and patched it."),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let rollouts = store
            .query_rollouts(RolloutQuery {
                statuses: Some(vec![RolloutStatus::Succeeded]),
                ..RolloutQuery::default()
            })
            .await
            .expect("query succeeded rollouts");
        let latest_rollout_id = rollouts
            .iter()
            .map(|rollout| rollout.rollout_id.as_str())
            .filter(|rollout_id| rollout_id.starts_with("live-rl-rollout-000"))
            .max()
            .expect("at least one runtime rollout");
        let spans = store
            .query_spans(latest_rollout_id, None)
            .await
            .expect("query spans");
        let latest = spans
            .iter()
            .max_by_key(|span| span.sequence_id)
            .expect("latest span");
        assert_eq!(
            latest
                .attributes
                .get("learning_trend")
                .and_then(serde_json::Value::as_str),
            Some("regressing")
        );
    }
}
