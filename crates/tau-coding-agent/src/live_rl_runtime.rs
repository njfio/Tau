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
const LIVE_RL_APO_AUTO_TRIGGER_THRESHOLD_ENV: &str = "TAU_LIVE_RL_APO_AUTO_TRIGGER_THRESHOLD";
const LIVE_ROLLOUT_PREFIX: &str = "live-rl-rollout";
const LIVE_LEARNING_OUTCOME_LIMIT: usize = 128;
const LIVE_CALIBRATION_BIN_COUNT: usize = 5;

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
    pub apo_auto_trigger_threshold: usize,
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
            None => true,
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

        let apo_auto_trigger_threshold = parse_positive_usize_env(
            env.get(LIVE_RL_APO_AUTO_TRIGGER_THRESHOLD_ENV)
                .map(String::as_str),
            20,
            LIVE_RL_APO_AUTO_TRIGGER_THRESHOLD_ENV,
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
            apo_auto_trigger_threshold,
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

#[derive(Debug, Clone, PartialEq)]
struct LiveCategorySummary {
    samples: usize,
    mean_reward: f64,
    success_rate: f64,
    calibration_error: f64,
    trend: String,
    difficulty_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct LiveLearningAlert {
    code: String,
    severity: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
struct LiveLearningSummary {
    category_stats: BTreeMap<String, LiveCategorySummary>,
    difficulty_weights: HashMap<String, f64>,
    calibration_curve: Vec<Value>,
    global_calibration_error: f64,
    alerts: Vec<LiveLearningAlert>,
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
    pub(crate) async fn register_if_enabled(
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
        Ok(Some(bridge.snapshot().await))
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
            let mut span = build_final_decision_span(&run, status);
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
            if let Err(error) = self.persist_live_learning_resources().await {
                self.register_failure(format!(
                    "live RL curriculum persistence failed for {}: {error}",
                    run.rollout_id
                ))
                .await;
                return;
            }
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

            let completed = {
                let state = self.inner.state.lock().await;
                state.completed_rollouts
            };
            if completed == self.inner.config.apo_auto_trigger_threshold {
                tracing::info!(
                    completed_rollouts = completed,
                    threshold = self.inner.config.apo_auto_trigger_threshold,
                    "APO auto-trigger threshold reached — full APO optimization pass should be scheduled"
                );
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
        let category = infer_task_category(prompt);

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
            let ask_for_help = should_recommend_help(history.len(), historical_success_rate, trend);
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
        let canonical_category =
            canonicalize_task_category(category).unwrap_or_else(|| "general".to_string());
        let outcomes = self.collect_recent_live_outcomes(limit).await?;
        Ok(outcomes
            .into_iter()
            .filter(|outcome| outcome.category == canonical_category)
            .collect())
    }

    async fn collect_recent_live_outcomes(&self, limit: usize) -> Result<Vec<LiveCategoryOutcome>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

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
            outcomes.push(outcome);
        }

        Ok(outcomes)
    }

    async fn persist_live_learning_resources(&self) -> Result<()> {
        let outcomes = self
            .collect_recent_live_outcomes(LIVE_LEARNING_OUTCOME_LIMIT)
            .await
            .context("failed to collect live outcomes for curriculum persistence")?;
        if outcomes.is_empty() {
            return Ok(());
        }

        let summary = build_live_learning_summary(outcomes.as_slice());
        if summary.category_stats.is_empty() {
            return Ok(());
        }

        let category_stats = summary
            .category_stats
            .into_iter()
            .map(|(category, stats)| {
                (
                    category,
                    json!({
                        "samples": u64::try_from(stats.samples).unwrap_or(0),
                        "mean_reward": stats.mean_reward,
                        "success_rate": stats.success_rate,
                        "calibration_error": stats.calibration_error,
                        "trend": stats.trend,
                        "difficulty_score": stats.difficulty_score,
                    }),
                )
            })
            .collect::<serde_json::Map<String, Value>>();
        let alerts = summary
            .alerts
            .into_iter()
            .map(|alert| {
                json!({
                    "code": alert.code,
                    "severity": alert.severity,
                    "message": alert.message,
                })
            })
            .collect::<Vec<_>>();

        let mut patch = HashMap::new();
        patch.insert(
            "live_curriculum_category_stats".to_string(),
            Value::Object(category_stats),
        );
        patch.insert(
            "live_curriculum_difficulty_weights".to_string(),
            json!(summary.difficulty_weights),
        );
        patch.insert(
            "live_meta_cognition_calibration_curve".to_string(),
            Value::Array(summary.calibration_curve),
        );
        patch.insert(
            "live_meta_cognition_global_calibration_error".to_string(),
            json!(summary.global_calibration_error),
        );
        patch.insert("live_learning_alerts".to_string(), Value::Array(alerts));
        patch.insert(
            "live_learning_summary_updated_unix_ms".to_string(),
            json!(u64::try_from(Utc::now().timestamp_millis().max(0)).unwrap_or(0)),
        );

        self.update_resources_merged(patch)
            .await
            .context("failed to persist live learning curriculum summary")
    }

    async fn load_live_curriculum_difficulty_weights(&self) -> Result<HashMap<String, f64>> {
        let latest = self
            .inner
            .store
            .get_latest_resources()
            .await
            .context("failed to read latest resources for live curriculum weights")?;
        let Some(latest) = latest else {
            return Ok(HashMap::new());
        };

        let Some(weights) = latest
            .resources
            .get("live_curriculum_difficulty_weights")
            .and_then(Value::as_object)
        else {
            return Ok(HashMap::new());
        };

        let mut parsed = HashMap::new();
        for (raw_category, raw_weight) in weights {
            let Some(category) = canonicalize_task_category(raw_category) else {
                continue;
            };
            let Some(weight) = raw_weight.as_f64() else {
                continue;
            };
            if weight.is_finite() {
                parsed.insert(category, weight.clamp(0.0, 1.0));
            }
        }
        Ok(parsed)
    }

    async fn update_resources_merged(&self, patch: HashMap<String, Value>) -> Result<()> {
        let mut merged = self
            .inner
            .store
            .get_latest_resources()
            .await
            .context("failed to read latest resources for merged update")?
            .map(|resources| resources.resources)
            .unwrap_or_default();
        merged.extend(patch);
        self.inner
            .store
            .update_resources(merged)
            .await
            .context("failed to persist merged live RL resources")?;
        Ok(())
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
        let difficulty_weights = self
            .load_live_curriculum_difficulty_weights()
            .await
            .unwrap_or_default();
        let (samples, curriculum_focus_category, curriculum_focus_mean_reward) =
            select_curriculum_samples(
                collected_samples,
                self.inner.config.apo_max_samples,
                &difficulty_weights,
            );

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
            Err(_) => {
                return Ok(LiveApoReport::skipped_with_curriculum(
                    "apo_run_failed",
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
            Err(_) => {
                return Ok(LiveApoReport::skipped_with_curriculum(
                    "apo_significance_failed",
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

        self.update_resources_merged(resources)
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
                .and_then(canonicalize_task_category)
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
            model: "gpt-5.2".to_string(),
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
    canonicalize_task_category(category).unwrap_or_else(|| "general".to_string())
}

fn canonicalize_task_category(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::with_capacity(trimmed.len());
    let mut prior_separator = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            prior_separator = false;
        } else if !prior_separator {
            normalized.push('_');
            prior_separator = true;
        }
    }
    let normalized = normalized.trim_matches('_').to_string();
    if normalized.is_empty() {
        return None;
    }

    let canonical = match normalized.as_str() {
        "debug" | "debugging" | "bugfix" | "bug_fix" => "debugging".to_string(),
        "refactor" | "refactoring" => "refactoring".to_string(),
        "codegen" | "implementation" | "implementing" => "code_generation".to_string(),
        "plan" | "planning" => "planning".to_string(),
        "ops" | "operational" | "operations" => "operations".to_string(),
        "q_a" | "qa" | "question" | "question_answer" | "question_answering" => "qa".to_string(),
        "general" | "misc" | "miscellaneous" => "general".to_string(),
        _ if normalized.starts_with("debug_") => format!("debugging_{}", &normalized[6..]),
        _ => normalized,
    };

    Some(canonical)
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
        .and_then(canonicalize_task_category)
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

fn should_recommend_help(history_len: usize, historical_success_rate: f64, trend: &str) -> bool {
    history_len >= 4 && historical_success_rate < 0.55 && trend != "improving"
}

fn compute_category_difficulty_score(
    mean_reward: f64,
    success_rate: f64,
    calibration_error: f64,
    trend: &str,
) -> f64 {
    let reward_component = 1.0 - normalize_reward_to_quality(mean_reward);
    let success_component = (1.0 - success_rate).clamp(0.0, 1.0);
    let calibration_component = calibration_error.clamp(0.0, 1.0);
    let trend_component = match trend {
        "regressing" => 0.15,
        "plateau" => 0.05,
        "improving" => -0.05,
        _ => 0.0,
    };
    (0.45 * reward_component
        + 0.35 * success_component
        + 0.20 * calibration_component
        + trend_component)
        .clamp(0.0, 1.0)
}

fn build_calibration_curve(outcomes: &[LiveCategoryOutcome], bin_count: usize) -> Vec<Value> {
    if outcomes.is_empty() || bin_count == 0 {
        return Vec::new();
    }
    let mut bins = vec![(0_usize, 0.0_f64, 0_usize); bin_count];
    for outcome in outcomes {
        let probability = outcome.predicted_success_probability.clamp(0.0, 1.0);
        let mut index = (probability * bin_count as f64).floor() as usize;
        if index >= bin_count {
            index = bin_count.saturating_sub(1);
        }
        let entry = &mut bins[index];
        entry.0 = entry.0.saturating_add(1);
        entry.1 += probability;
        entry.2 = entry.2.saturating_add(usize::from(outcome.actual_success));
    }

    let mut curve = Vec::new();
    for (index, (samples, predicted_sum, success_count)) in bins.into_iter().enumerate() {
        if samples == 0 {
            continue;
        }
        let lower_bound = index as f64 / bin_count as f64;
        let upper_bound = (index + 1) as f64 / bin_count as f64;
        let mean_predicted_success = predicted_sum / samples as f64;
        let empirical_success_rate = success_count as f64 / samples as f64;
        curve.push(json!({
            "bin": format!("{lower_bound:.1}-{upper_bound:.1}"),
            "lower_bound": lower_bound,
            "upper_bound": upper_bound,
            "samples": u64::try_from(samples).unwrap_or(0),
            "mean_predicted_success": mean_predicted_success,
            "empirical_success_rate": empirical_success_rate,
            "calibration_gap": (mean_predicted_success - empirical_success_rate).abs(),
        }));
    }
    curve
}

fn build_live_learning_summary(outcomes: &[LiveCategoryOutcome]) -> LiveLearningSummary {
    let mut grouped = BTreeMap::<String, Vec<&LiveCategoryOutcome>>::new();
    for outcome in outcomes {
        grouped
            .entry(outcome.category.clone())
            .or_default()
            .push(outcome);
    }

    let mut category_stats = BTreeMap::new();
    let mut difficulty_weights = HashMap::new();
    let mut alerts = Vec::new();
    for (category, group) in grouped {
        let rewards = group
            .iter()
            .map(|outcome| outcome.reward)
            .collect::<Vec<_>>();
        let mean_reward = mean(rewards.as_slice());
        let success_rate = group
            .iter()
            .filter(|outcome| outcome.actual_success)
            .count() as f64
            / group.len() as f64;
        let calibration_error = mean(
            group
                .iter()
                .map(|outcome| {
                    (outcome.predicted_success_probability
                        - if outcome.actual_success { 1.0 } else { 0.0 })
                    .abs()
                })
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let trend = classify_learning_trend(rewards.as_slice()).to_string();
        let difficulty_score = compute_category_difficulty_score(
            mean_reward,
            success_rate,
            calibration_error,
            trend.as_str(),
        );

        if trend == "regressing" && group.len() >= 4 {
            alerts.push(LiveLearningAlert {
                code: "live_learning_regressing_category".to_string(),
                severity: "warning".to_string(),
                message: format!("{category} trend regressing over recent windows"),
            });
        }

        category_stats.insert(
            category.clone(),
            LiveCategorySummary {
                samples: group.len(),
                mean_reward,
                success_rate,
                calibration_error,
                trend,
                difficulty_score,
            },
        );
        difficulty_weights.insert(category, difficulty_score);
    }

    let global_calibration_error = mean(
        outcomes
            .iter()
            .map(|outcome| {
                (outcome.predicted_success_probability
                    - if outcome.actual_success { 1.0 } else { 0.0 })
                .abs()
            })
            .collect::<Vec<_>>()
            .as_slice(),
    );
    if outcomes.len() >= 8 && global_calibration_error > 0.25 {
        alerts.push(LiveLearningAlert {
            code: "live_learning_poor_calibration".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "global calibration gap {:.3} exceeds threshold 0.250",
                global_calibration_error
            ),
        });
    }
    alerts.truncate(8);

    LiveLearningSummary {
        category_stats,
        difficulty_weights,
        calibration_curve: build_calibration_curve(outcomes, LIVE_CALIBRATION_BIN_COUNT),
        global_calibration_error,
        alerts,
    }
}

fn select_curriculum_samples(
    samples: Vec<LiveApoSample>,
    max_samples: usize,
    difficulty_weights: &HashMap<String, f64>,
) -> (Vec<LiveApoSample>, Option<String>, Option<f64>) {
    if samples.is_empty() || max_samples == 0 {
        return (Vec::new(), None, None);
    }

    let mut grouped = BTreeMap::<String, Vec<LiveApoSample>>::new();
    for sample in samples {
        let category = canonicalize_task_category(sample.category.as_str())
            .unwrap_or_else(|| "general".to_string());
        grouped.entry(category).or_default().push(sample);
    }

    let mut category_rankings = grouped
        .iter()
        .map(|(category, grouped_samples)| {
            let mean_reward = mean(
                grouped_samples
                    .iter()
                    .map(|sample| sample.reward)
                    .collect::<Vec<_>>()
                    .as_slice(),
            );
            let base_difficulty = 1.0 - normalize_reward_to_quality(mean_reward);
            let weight = difficulty_weights.get(category).copied().unwrap_or(1.0);
            let weighted_difficulty = (base_difficulty * weight.max(0.05)).clamp(0.0, 1.0);
            (category.clone(), mean_reward, weighted_difficulty)
        })
        .collect::<Vec<_>>();
    category_rankings.sort_by(|left, right| {
        right
            .2
            .partial_cmp(&left.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.0.cmp(&right.0))
    });
    let curriculum_focus_category = category_rankings.first().map(|entry| entry.0.clone());
    let curriculum_focus_mean_reward = category_rankings.first().map(|entry| entry.1);

    let total = grouped.values().map(Vec::len).sum::<usize>();
    if total <= max_samples {
        let mut selected = Vec::new();
        for (category, _, _) in &category_rankings {
            if let Some(grouped_samples) = grouped.remove(category) {
                selected.extend(grouped_samples);
            }
        }
        return (
            selected,
            curriculum_focus_category,
            curriculum_focus_mean_reward,
        );
    }

    let mut selected = Vec::with_capacity(max_samples);
    for (category, _, _) in &category_rankings {
        if selected.len() == max_samples {
            break;
        }
        if let Some(grouped_samples) = grouped.get_mut(category) {
            if let Some(sample) = grouped_samples.pop() {
                selected.push(sample);
            }
        }
    }
    while selected.len() < max_samples {
        let mut progressed = false;
        for (category, _, weighted_difficulty) in &category_rankings {
            if selected.len() == max_samples {
                break;
            }
            let burst = (weighted_difficulty * 2.0).ceil() as usize;
            let burst = burst.max(1);
            if let Some(grouped_samples) = grouped.get_mut(category) {
                for _ in 0..burst {
                    if selected.len() == max_samples {
                        break;
                    }
                    if let Some(sample) = grouped_samples.pop() {
                        selected.push(sample);
                        progressed = true;
                    } else {
                        break;
                    }
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

fn build_final_decision_span(run: &LiveRlActiveRun, status: RolloutStatus) -> TrainingSpan {
    let reward = compute_live_reward_breakdown(run, status);
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
    span.attributes.insert(
        "session_completed".to_string(),
        json!(status == RolloutStatus::Succeeded),
    );
    span.attributes.insert(
        "input_chars".to_string(),
        json!(run.prompt.as_ref().map_or(0, |p| p.chars().count())),
    );
    span.attributes.insert(
        "output_chars".to_string(),
        json!(run
            .assistant_reply
            .as_ref()
            .map_or(0, |r| r.chars().count())),
    );
    span.attributes.insert("done".to_string(), json!(true));
    span.end_time = Some(Utc::now());
    span
}

#[cfg(test)]
fn compute_live_reward(run: &LiveRlActiveRun) -> f64 {
    compute_live_reward_breakdown(run, RolloutStatus::Succeeded).composite
}

fn compute_live_reward_breakdown(
    run: &LiveRlActiveRun,
    status: RolloutStatus,
) -> RewardInferenceOutput {
    let has_assistant_reply = run
        .assistant_reply
        .as_ref()
        .is_some_and(|reply| !reply.trim().is_empty());
    let session_completed = status == RolloutStatus::Succeeded;
    let input = RewardInferenceInput::new(
        has_assistant_reply,
        session_completed,
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
mod tests;
