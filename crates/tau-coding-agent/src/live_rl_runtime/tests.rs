use super::{
    build_final_decision_span, compute_live_reward, should_recommend_help, LiveRlActiveRun,
    LiveRlRuntimeBridge, LiveRlRuntimeConfig, LiveRlRuntimeGate,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::{BTreeMap, HashMap, VecDeque};
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

#[derive(Clone)]
struct FailingClient {
    message: String,
}

impl FailingClient {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[async_trait]
impl LlmClient for FailingClient {
    async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
        Err(TauAiError::InvalidResponse(self.message.clone()))
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
        format!("{rollout_id}:attempt-live"),
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
        format!("{rollout_id}:attempt-live"),
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

#[allow(clippy::too_many_arguments)]
async fn seed_live_rollout_outcome(
    store: &Arc<dyn TrainingStore + Send + Sync>,
    rollout_id: &str,
    status: RolloutStatus,
    prompt: &str,
    response: &str,
    reward: f64,
    category: &str,
    predicted_success_probability: f64,
    actual_success: bool,
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
        .update_rollout_status(rollout_id, status)
        .await
        .expect("mark seeded rollout final status");

    let mut span = TrainingSpan::new(
        rollout_id,
        format!("{rollout_id}:attempt-live"),
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
    span.attributes.insert(
        "predicted_success_probability".to_string(),
        json!(predicted_success_probability),
    );
    span.attributes
        .insert("actual_success".to_string(), json!(actual_success));
    span.attributes.insert("done".to_string(), json!(true));
    span.end_time = Some(Utc::now());
    store.add_span(span).await.expect("add seeded span");
}

#[test]
fn spec_c04_unit_live_rl_env_defaults_to_enabled() {
    let env = BTreeMap::new();
    let config =
        LiveRlRuntimeConfig::from_env_map(&env, std::path::Path::new(".tau/training/store.sqlite"))
            .expect("config from env");
    assert!(config.enabled);
    assert_eq!(config.update_interval_rollouts, 8);
    assert_eq!(config.max_rollouts_per_update, 64);
    assert_eq!(config.max_failure_streak, 3);
    assert!(config.apo_enabled);
    assert_eq!(config.apo_min_samples, 4);
    assert_eq!(config.apo_max_samples, 32);
    assert!((config.apo_significance_alpha - 0.05).abs() < 1e-12);
    assert_eq!(config.apo_auto_trigger_threshold, 20);
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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
    // session_completed=true (Succeeded), has_assistant_reply=false:
    // completion=0.0 + session_completion=0.0 + efficiency=0.25 + token_efficiency=0.0
    assert_eq!(compute_live_reward(&no_reply), 0.25);

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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
        .expect("get latest resources")
        .expect("live learning resources should still persist");
    assert!(
        !latest.resources.contains_key("system_prompt"),
        "system_prompt should not be adopted without significant improvement"
    );
    assert!(
        !latest.resources.contains_key("system_prompt_version"),
        "system_prompt_version should not be persisted without prompt adoption"
    );
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
            apo_auto_trigger_threshold: 20,
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
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
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
                apo_auto_trigger_threshold: 20,
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
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
        let rollout_ids = seed_live_rollouts(&store, "c10-min4", &[0.20, 0.30, 0.40, 0.50]).await;
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
                apo_auto_trigger_threshold: 20,
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
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
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
                apo_auto_trigger_threshold: 20,
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
        let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
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
                apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
async fn spec_c20_regression_live_apo_missing_runtime_reports_deterministic_reason_code() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    let rollout_ids = seed_live_rollouts(&store, "c20", &[0.4, 0.5, 0.6, 0.7]).await;
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
            apo_auto_trigger_threshold: 20,
        },
    );

    let report = bridge
        .run_live_apo_update(rollout_ids.as_slice())
        .await
        .expect("run APO update");
    assert!(!report.executed, "report={report:?}");
    assert!(!report.adopted);
    assert_eq!(report.sample_count, 0);
    assert_eq!(report.reason_code.as_deref(), Some("apo_missing_runtime"));
}

#[tokio::test]
async fn spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    let rollout_ids = seed_live_rollouts(&store, "c21", &[0.2, 0.3, 0.4, 0.5]).await;
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
            apo_auto_trigger_threshold: 20,
        },
        Arc::new(FailingClient::new("forced APO request failure")),
        "You are Tau.",
    );

    let report = bridge
        .run_live_apo_update(rollout_ids.as_slice())
        .await
        .expect("run APO update");
    assert!(!report.executed, "report={report:?}");
    assert!(!report.adopted);
    assert_eq!(report.sample_count, 4);
    assert_eq!(report.reason_code.as_deref(), Some("apo_run_failed"));
}

#[tokio::test]
async fn spec_c22_regression_live_apo_significance_failure_reports_deterministic_reason_code() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    let rollout_ids = seed_live_rollouts(&store, "c22", &[-0.9, -0.8, -0.7, -0.6]).await;
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
            apo_significance_alpha: 0.02,
            apo_auto_trigger_threshold: 20,
        },
        Arc::new(ScriptedClient::new(vec![
            "{\"score\":0.10}",
            "Improve deterministic checks.",
            "You are Tau. Prefer deterministic checks and concise plans.",
            "{\"score\":0.90}",
        ])),
        "You are Tau.",
    );

    let report = bridge
        .run_live_apo_update(rollout_ids.as_slice())
        .await
        .expect("run APO update");
    assert!(!report.executed);
    assert!(!report.adopted);
    assert_eq!(report.sample_count, 4);
    assert_eq!(
        report.reason_code.as_deref(),
        Some("apo_significance_failed")
    );
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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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
            apo_auto_trigger_threshold: 20,
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

#[tokio::test]
async fn spec_c15_regression_collect_recent_category_outcomes_filters_succeeded_and_caps_limit() {
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
            apo_auto_trigger_threshold: 20,
        },
    );

    for index in 0..40 {
        let rollout_id = format!("live-rl-rollout-c15-{index:04}");
        let reward = if index < 20 { 0.4 } else { -0.4 };
        let actual_success = reward > 0.0;
        seed_live_rollout_outcome(
            &store,
            rollout_id.as_str(),
            RolloutStatus::Succeeded,
            "debug parser",
            "debug output",
            reward,
            "debugging",
            0.75,
            actual_success,
        )
        .await;
    }
    seed_live_rollout_outcome(
        &store,
        "live-rl-rollout-c15-failed",
        RolloutStatus::Failed,
        "debug parser",
        "failed run",
        -1.0,
        "debugging",
        0.99,
        false,
    )
    .await;

    let outcomes = bridge
        .collect_recent_category_outcomes("debugging", 32)
        .await
        .expect("collect category outcomes");
    assert_eq!(outcomes.len(), 32);
    assert!(outcomes
        .iter()
        .all(|outcome| outcome.category == "debugging"));
    assert!(
        outcomes
            .iter()
            .all(|outcome| (outcome.reward - (-1.0_f64)).abs() > 1e-12),
        "failed rollouts must not be included"
    );
}

#[tokio::test]
async fn spec_c16_regression_meta_cognition_history_fields_and_help_thresholds() {
    {
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
                apo_auto_trigger_threshold: 20,
            },
        );

        let rewards = [0.8, 0.7, 0.6, -0.4, -0.5, -0.6];
        for (index, reward) in rewards.into_iter().enumerate() {
            let rollout_id = format!("live-rl-rollout-c16-a-{index:04}");
            let actual_success = reward > 0.0;
            seed_live_rollout_outcome(
                &store,
                rollout_id.as_str(),
                RolloutStatus::Succeeded,
                "debug parser mismatch",
                "debug output",
                reward,
                "debugging",
                0.9,
                actual_success,
            )
            .await;
        }

        let run = LiveRlActiveRun {
            rollout_id: "live-rl-rollout-0000009999".to_string(),
            attempt_id: "live-rl-rollout-0000009999:attempt-live".to_string(),
            prompt: Some("debug parser mismatch".to_string()),
            assistant_reply: Some("patched parser mismatch".to_string()),
            turns: 2,
            tool_errors: 0,
            safety_blocked: false,
        };
        let mut span = build_final_decision_span(&run, RolloutStatus::Succeeded);
        bridge
            .enrich_final_decision_span(&mut span)
            .await
            .expect("enrich span");
        let attrs = &span.attributes;

        assert_eq!(
            attrs
                .get("historical_category_samples")
                .and_then(serde_json::Value::as_u64),
            Some(6)
        );
        assert_eq!(
            attrs
                .get("learning_trend")
                .and_then(serde_json::Value::as_str),
            Some("regressing")
        );
        let success_rate = attrs
            .get("historical_success_rate")
            .and_then(serde_json::Value::as_f64)
            .expect("historical success rate");
        assert!((success_rate - 0.5).abs() < 1e-12);
        let calibration_error = attrs
            .get("historical_calibration_error")
            .and_then(serde_json::Value::as_f64)
            .expect("historical calibration error");
        assert!((calibration_error - 0.5).abs() < 1e-12);
        assert_eq!(
            attrs
                .get("ask_for_help_recommended")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    {
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
                apo_auto_trigger_threshold: 20,
            },
        );

        for index in 0..20 {
            let rollout_id = format!("live-rl-rollout-c16-b-{index:04}");
            let actual_success = index < 11;
            let reward = if actual_success { 0.8 } else { -0.8 };
            seed_live_rollout_outcome(
                &store,
                rollout_id.as_str(),
                RolloutStatus::Succeeded,
                "debug parser mismatch",
                "debug output",
                reward,
                "debugging",
                0.8,
                actual_success,
            )
            .await;
        }

        let run = LiveRlActiveRun {
            rollout_id: "live-rl-rollout-0000009998".to_string(),
            attempt_id: "live-rl-rollout-0000009998:attempt-live".to_string(),
            prompt: Some("debug parser mismatch".to_string()),
            assistant_reply: Some("patched parser mismatch".to_string()),
            turns: 2,
            tool_errors: 0,
            safety_blocked: false,
        };
        let mut span = build_final_decision_span(&run, RolloutStatus::Succeeded);
        bridge
            .enrich_final_decision_span(&mut span)
            .await
            .expect("enrich span");
        let attrs = &span.attributes;
        assert_eq!(
            attrs
                .get("historical_success_rate")
                .and_then(serde_json::Value::as_f64),
            Some(0.55)
        );
        assert_eq!(
            attrs
                .get("ask_for_help_recommended")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
    }
}

#[test]
fn spec_c17_unit_should_recommend_help_requires_all_conditions() {
    assert!(should_recommend_help(4, 0.50, "regressing"));
    assert!(!should_recommend_help(3, 0.50, "regressing"));
    assert!(!should_recommend_help(4, 0.60, "regressing"));
    assert!(!should_recommend_help(4, 0.50, "improving"));
}

#[tokio::test]
async fn spec_c18_regression_live_curriculum_aggregates_persisted_to_resources() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    seed_live_rollout_outcome(
        &store,
        "live-rl-rollout-c18-0000",
        RolloutStatus::Succeeded,
        "debug rust borrow checker mismatch",
        "fixed borrow checker mismatch",
        0.6,
        "Debugging/Rust",
        0.9,
        true,
    )
    .await;
    seed_live_rollout_outcome(
        &store,
        "live-rl-rollout-c18-0001",
        RolloutStatus::Succeeded,
        "debug rust trait bound regression",
        "found trait bound root cause",
        -0.4,
        "debugging-rust",
        0.8,
        false,
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
            apo_auto_trigger_threshold: 20,
        },
    );

    bridge.handle_event(AgentEvent::AgentStart).await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::user("debug rust panic in parser pipeline"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::assistant_text("patched parser pipeline panic"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
        .await;

    let resources = store
        .get_latest_resources()
        .await
        .expect("read resources")
        .expect("resources should be persisted");
    let category_stats = resources
        .resources
        .get("live_curriculum_category_stats")
        .and_then(serde_json::Value::as_object)
        .expect("live curriculum category stats");
    let debugging_rust = category_stats
        .get("debugging_rust")
        .and_then(serde_json::Value::as_object)
        .expect("canonical debugging_rust category");
    assert!(
        debugging_rust
            .get("samples")
            .and_then(serde_json::Value::as_u64)
            .expect("category samples")
            >= 2
    );
    assert!(debugging_rust
        .get("difficulty_score")
        .and_then(serde_json::Value::as_f64)
        .is_some());
    assert!(debugging_rust
        .get("trend")
        .and_then(serde_json::Value::as_str)
        .is_some());
}

#[tokio::test]
async fn spec_c19_regression_live_apo_progressive_difficulty_prioritizes_harder_category() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    seed_live_rollout_with_category(
        &store,
        "live-rl-rollout-c19-0000",
        "debug intermittent crash",
        "debug fix 1",
        -0.2,
        "debugging",
    )
    .await;
    seed_live_rollout_with_category(
        &store,
        "live-rl-rollout-c19-0001",
        "debug intermittent crash",
        "debug fix 2",
        -0.1,
        "debugging",
    )
    .await;
    seed_live_rollout_with_category(
        &store,
        "live-rl-rollout-c19-0002",
        "summarize docs update",
        "summary 1",
        -0.9,
        "qa",
    )
    .await;
    seed_live_rollout_with_category(
        &store,
        "live-rl-rollout-c19-0003",
        "summarize docs update",
        "summary 2",
        -0.8,
        "qa",
    )
    .await;

    store
        .update_resources(HashMap::from([(
            "live_curriculum_difficulty_weights".to_string(),
            json!({"debugging": 0.95, "qa": 0.10}),
        )]))
        .await
        .expect("seed curriculum weights");

    let rollout_ids = vec![
        "live-rl-rollout-c19-0000".to_string(),
        "live-rl-rollout-c19-0001".to_string(),
        "live-rl-rollout-c19-0002".to_string(),
        "live-rl-rollout-c19-0003".to_string(),
    ];

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
            apo_max_samples: 2,
            apo_significance_alpha: 0.05,
            apo_auto_trigger_threshold: 20,
        },
        Arc::new(ScriptedClient::new(vec![
            "{\"score\":0.20}",
            "Prioritize deterministic debug narrowing and validation.",
            "You are Tau. Focus on deterministic debug narrowing before broad edits.",
            "{\"score\":0.99}",
        ])),
        "You are Tau.",
    );

    let report = bridge
        .run_live_apo_update(rollout_ids.as_slice())
        .await
        .expect("run APO update");
    assert_eq!(
        report.curriculum_focus_category.as_deref(),
        Some("debugging")
    );
}

#[tokio::test]
async fn spec_c20_regression_live_calibration_curve_and_alerts_are_persisted() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    for index in 0..10 {
        let rollout_id = format!("live-rl-rollout-c20-{index:04}");
        seed_live_rollout_outcome(
            &store,
            rollout_id.as_str(),
            RolloutStatus::Succeeded,
            "debug parser crash",
            "attempted parser fix",
            if index < 4 { 0.5 } else { -0.6 },
            "debugging",
            0.95,
            index < 4,
        )
        .await;
    }

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
            apo_auto_trigger_threshold: 20,
        },
    );

    bridge.handle_event(AgentEvent::AgentStart).await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::user("debug parser crash after dependency update"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::assistant_text("narrowed parser crash to decoding branch"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
        .await;

    let resources = store
        .get_latest_resources()
        .await
        .expect("read resources")
        .expect("resources should be persisted");
    let calibration_curve = resources
        .resources
        .get("live_meta_cognition_calibration_curve")
        .and_then(serde_json::Value::as_array)
        .expect("calibration curve");
    assert!(!calibration_curve.is_empty());
    let first_bin = calibration_curve
        .first()
        .and_then(serde_json::Value::as_object)
        .expect("calibration bin object");
    assert!(first_bin
        .get("samples")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|samples| samples > 0));
    assert!(first_bin
        .get("mean_predicted_success")
        .and_then(serde_json::Value::as_f64)
        .is_some());
    assert!(first_bin
        .get("empirical_success_rate")
        .and_then(serde_json::Value::as_f64)
        .is_some());

    let alerts = resources
        .resources
        .get("live_learning_alerts")
        .and_then(serde_json::Value::as_array)
        .expect("live learning alerts");
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|entry| {
        entry
            .get("code")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|code| code == "live_learning_regressing_category")
    }));
}

#[test]
fn unit_reward_inference_input_construction_from_span_data() {
    use tau_algorithm::{RewardInference, RewardInferenceInput, TraceBasedRewardInference};

    let input = RewardInferenceInput::new(
        true,  // has_assistant_reply
        true,  // session_completed
        1,     // tool_errors
        false, // safety_blocked
        3,     // turns
        100,   // input_chars
        200,   // output_chars
    );
    let output = TraceBasedRewardInference.infer(&input);
    assert!(output.composite.is_finite());
    assert!(output.composite >= -1.0 && output.composite <= 1.0);
    assert!(
        output.reliability < 0.0,
        "tool errors should reduce reliability"
    );
    assert_eq!(output.completion, 0.5, "assistant reply present");

    // Verify construction with session not completed
    let input_not_completed = RewardInferenceInput::new(true, false, 0, false, 2, 50, 60);
    let output_not_completed = TraceBasedRewardInference.infer(&input_not_completed);
    assert!(output_not_completed.session_completion < 0.0);

    // Verify build_final_decision_span populates all expected fields
    let run = LiveRlActiveRun {
        rollout_id: "test-rollout-001".to_string(),
        attempt_id: "test-rollout-001:attempt-live".to_string(),
        prompt: Some("implement feature X".to_string()),
        assistant_reply: Some("Done implementing feature X.".to_string()),
        turns: 2,
        tool_errors: 0,
        safety_blocked: false,
    };
    let span = build_final_decision_span(&run, RolloutStatus::Succeeded);
    assert_eq!(span.attributes["session_completed"], json!(true));
    assert!(span.attributes["input_chars"].as_u64().unwrap() > 0);
    assert!(span.attributes["output_chars"].as_u64().unwrap() > 0);
    assert_eq!(span.attributes["reward"], json!(1.0));
}

#[tokio::test]
async fn unit_apo_trigger_threshold_fires_at_n_not_before() {
    let store: Arc<dyn TrainingStore + Send + Sync> = Arc::new(InMemoryTrainingStore::new());
    let threshold = 3_usize;
    let bridge = LiveRlRuntimeBridge::for_tests(
        store.clone(),
        LiveRlRuntimeConfig {
            enabled: true,
            store_path: ".tau/training/store.sqlite".into(),
            update_interval_rollouts: 100, // high so optimizer doesn't run
            max_rollouts_per_update: 32,
            max_failure_streak: 10,
            apo_enabled: false,
            apo_min_samples: 4,
            apo_max_samples: 32,
            apo_significance_alpha: 0.05,
            apo_auto_trigger_threshold: threshold,
        },
    );

    // Complete rollouts up to threshold
    for i in 0..threshold {
        bridge.handle_event(AgentEvent::AgentStart).await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::user(format!("request {i}")),
            })
            .await;
        bridge
            .handle_event(AgentEvent::MessageAdded {
                message: Message::assistant_text(format!("response {i}")),
            })
            .await;
        bridge
            .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
            .await;

        let snapshot = bridge.snapshot().await;
        assert_eq!(snapshot.completed_rollouts, i + 1);
    }

    // After threshold rollouts, verify completed count matches threshold
    let snapshot = bridge.snapshot().await;
    assert_eq!(snapshot.completed_rollouts, threshold);

    // One more rollout — past threshold (trigger should have fired once at ==threshold)
    bridge.handle_event(AgentEvent::AgentStart).await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::user("extra request"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::MessageAdded {
            message: Message::assistant_text("extra response"),
        })
        .await;
    bridge
        .handle_event(AgentEvent::AgentEnd { new_messages: 2 })
        .await;

    let snapshot = bridge.snapshot().await;
    assert_eq!(snapshot.completed_rollouts, threshold + 1);
}
