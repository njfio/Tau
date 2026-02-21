//! OpenAI compatibility and UI telemetry runtime-state helpers.

use super::*;

impl GatewayOpenResponsesServerState {
    pub(super) fn record_openai_compat_request(
        &self,
        surface: GatewayOpenAiCompatSurface,
        stream: bool,
    ) {
        if let Ok(mut runtime) = self.compat_runtime.lock() {
            runtime.total_requests = runtime.total_requests.saturating_add(1);
            if stream {
                runtime.stream_requests = runtime.stream_requests.saturating_add(1);
            }
            match surface {
                GatewayOpenAiCompatSurface::ChatCompletions => {
                    runtime.chat_completions_requests =
                        runtime.chat_completions_requests.saturating_add(1);
                }
                GatewayOpenAiCompatSurface::Completions => {
                    runtime.completions_requests = runtime.completions_requests.saturating_add(1);
                }
                GatewayOpenAiCompatSurface::Models => {
                    runtime.models_requests = runtime.models_requests.saturating_add(1);
                }
            }
        }
    }

    pub(super) fn record_openai_compat_reason(&self, reason_code: &str) {
        if reason_code.trim().is_empty() {
            return;
        }
        if let Ok(mut runtime) = self.compat_runtime.lock() {
            *runtime
                .reason_code_counts
                .entry(reason_code.to_string())
                .or_default() += 1;
            runtime.last_reason_codes.push(reason_code.to_string());
            if runtime.last_reason_codes.len() > 16 {
                let drop_count = runtime.last_reason_codes.len().saturating_sub(16);
                runtime.last_reason_codes.drain(0..drop_count);
            }
        }
    }

    pub(super) fn record_openai_compat_ignored_fields(&self, fields: &[String]) {
        if fields.is_empty() {
            return;
        }
        if let Ok(mut runtime) = self.compat_runtime.lock() {
            for field in fields {
                if field.trim().is_empty() {
                    continue;
                }
                *runtime
                    .ignored_field_counts
                    .entry(field.clone())
                    .or_default() += 1;
            }
        }
    }

    pub(super) fn collect_openai_compat_status_report(&self) -> GatewayOpenAiCompatStatusReport {
        if let Ok(runtime) = self.compat_runtime.lock() {
            return GatewayOpenAiCompatStatusReport {
                total_requests: runtime.total_requests,
                chat_completions_requests: runtime.chat_completions_requests,
                completions_requests: runtime.completions_requests,
                models_requests: runtime.models_requests,
                stream_requests: runtime.stream_requests,
                translation_failures: runtime.translation_failures,
                execution_failures: runtime.execution_failures,
                reason_code_counts: runtime.reason_code_counts.clone(),
                ignored_field_counts: runtime.ignored_field_counts.clone(),
                last_reason_codes: runtime.last_reason_codes.clone(),
            };
        }

        GatewayOpenAiCompatStatusReport::default()
    }

    pub(super) fn increment_openai_compat_translation_failures(&self) {
        if let Ok(mut runtime) = self.compat_runtime.lock() {
            runtime.translation_failures = runtime.translation_failures.saturating_add(1);
        }
    }

    pub(super) fn increment_openai_compat_execution_failures(&self) {
        if let Ok(mut runtime) = self.compat_runtime.lock() {
            runtime.execution_failures = runtime.execution_failures.saturating_add(1);
        }
    }

    pub(super) fn record_ui_telemetry_event(&self, view: &str, action: &str, reason_code: &str) {
        if let Ok(mut runtime) = self.ui_telemetry_runtime.lock() {
            runtime.total_events = runtime.total_events.saturating_add(1);
            runtime.last_event_unix_ms = Some(current_unix_timestamp_ms());

            if !view.trim().is_empty() {
                *runtime
                    .view_counts
                    .entry(view.trim().to_string())
                    .or_default() += 1;
            }
            if !action.trim().is_empty() {
                *runtime
                    .action_counts
                    .entry(action.trim().to_string())
                    .or_default() += 1;
            }
            if !reason_code.trim().is_empty() {
                *runtime
                    .reason_code_counts
                    .entry(reason_code.trim().to_string())
                    .or_default() += 1;
            }
        }
    }

    pub(super) fn collect_ui_telemetry_status_report(&self) -> GatewayUiTelemetryStatusReport {
        if let Ok(runtime) = self.ui_telemetry_runtime.lock() {
            return GatewayUiTelemetryStatusReport {
                total_events: runtime.total_events,
                last_event_unix_ms: runtime.last_event_unix_ms,
                view_counts: runtime.view_counts.clone(),
                action_counts: runtime.action_counts.clone(),
                reason_code_counts: runtime.reason_code_counts.clone(),
            };
        }
        GatewayUiTelemetryStatusReport::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum GatewayOpenAiCompatSurface {
    ChatCompletions,
    Completions,
    Models,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GatewayOpenAiCompatRuntimeState {
    pub(super) total_requests: u64,
    pub(super) chat_completions_requests: u64,
    pub(super) completions_requests: u64,
    pub(super) models_requests: u64,
    pub(super) stream_requests: u64,
    pub(super) translation_failures: u64,
    pub(super) execution_failures: u64,
    pub(super) reason_code_counts: BTreeMap<String, u64>,
    pub(super) ignored_field_counts: BTreeMap<String, u64>,
    pub(super) last_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GatewayUiTelemetryRuntimeState {
    pub(super) total_events: u64,
    pub(super) last_event_unix_ms: Option<u64>,
    pub(super) view_counts: BTreeMap<String, u64>,
    pub(super) action_counts: BTreeMap<String, u64>,
    pub(super) reason_code_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(super) struct GatewayOpenAiCompatStatusReport {
    total_requests: u64,
    chat_completions_requests: u64,
    completions_requests: u64,
    models_requests: u64,
    stream_requests: u64,
    translation_failures: u64,
    execution_failures: u64,
    reason_code_counts: BTreeMap<String, u64>,
    ignored_field_counts: BTreeMap<String, u64>,
    last_reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(super) struct GatewayUiTelemetryStatusReport {
    total_events: u64,
    last_event_unix_ms: Option<u64>,
    view_counts: BTreeMap<String, u64>,
    action_counts: BTreeMap<String, u64>,
    reason_code_counts: BTreeMap<String, u64>,
}
