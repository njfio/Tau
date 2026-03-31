//! Explicit mission completion/checkpoint tooling for gateway Ralph-loop missions.

use std::future::Future;
use std::pin::Pin;

use super::*;
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_ai::ToolDefinition;

pub(super) const GATEWAY_COMPLETE_TASK_TOOL_NAME: &str = "complete_task";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum GatewayMissionCompletionStatus {
    Success,
    Partial,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct GatewayMissionCompletionSignalRecord {
    pub(super) status: GatewayMissionCompletionStatus,
    pub(super) summary: String,
    #[serde(default)]
    pub(super) next_step: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct GatewayCompleteTaskTool;

impl AgentTool for GatewayCompleteTaskTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: GATEWAY_COMPLETE_TASK_TOOL_NAME.to_string(),
            description:
                "Marks the current gateway mission as complete, checkpointed, or explicitly blocked"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Summary of what was accomplished or why the mission is blocked"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["success", "partial", "blocked"],
                        "description": "Use success when the mission objective is satisfied, partial when checkpointing partial progress, and blocked when the mission cannot continue without outside help"
                    },
                    "next_step": {
                        "type": "string",
                        "description": "Optional next step to resume from after a partial checkpoint"
                    }
                },
                "required": ["summary"]
            }),
        }
    }

    fn execute<'life0, 'async_trait>(
        &'life0 self,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = ToolExecutionResult> + Send + 'async_trait>>
    where
        Self: 'async_trait,
        'life0: 'async_trait,
    {
        Box::pin(async move {
            let summary = arguments
                .get("summary")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|summary| !summary.is_empty())
                .unwrap_or("mission completion requested")
                .to_string();
            let status = arguments
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("success")
                .trim()
                .to_ascii_lowercase();
            let next_step = arguments
                .get("next_step")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|next_step| !next_step.is_empty())
                .map(ToOwned::to_owned);
            ToolExecutionResult::ok(json!({
                "status": status,
                "summary": summary,
                "next_step": next_step,
                "should_stop": true
            }))
        })
    }
}

pub(super) fn register_gateway_completion_tool(agent: &mut Agent) {
    if agent.has_tool(GATEWAY_COMPLETE_TASK_TOOL_NAME) {
        return;
    }
    agent.register_tool(GatewayCompleteTaskTool);
}

pub(super) fn extract_gateway_completion_signal(
    traces: &[GatewayVerifierToolTrace],
) -> Option<GatewayMissionCompletionSignalRecord> {
    traces.iter().rev().find_map(|trace| {
        if !trace.success || trace.tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME {
            return None;
        }
        let status = match trace
            .arguments
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("success")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "partial" => GatewayMissionCompletionStatus::Partial,
            "blocked" => GatewayMissionCompletionStatus::Blocked,
            _ => GatewayMissionCompletionStatus::Success,
        };
        let summary = trace
            .arguments
            .get("summary")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
            .unwrap_or("mission completion requested")
            .to_string();
        let next_step = trace
            .arguments
            .get("next_step")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|next_step| !next_step.is_empty())
            .map(ToOwned::to_owned);
        Some(GatewayMissionCompletionSignalRecord {
            status,
            summary,
            next_step,
        })
    })
}

pub(super) fn render_gateway_completion_guidance() -> &'static str {
    "## Mission Completion\nWhen the mission objective is actually satisfied, call `complete_task` with `status: \"success\"` and a concise summary. If you need to checkpoint partial progress, call `complete_task` with `status: \"partial\"` and include `next_step`. If you are truly blocked by a concrete external dependency, call `complete_task` with `status: \"blocked\"` and explain why."
}
