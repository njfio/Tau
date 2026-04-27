//! Structured verifier bundles and first back-pressure adapters for gateway missions.

use super::*;

const ACTION_TOOL_EVIDENCE_RETRY_EXHAUSTED_MESSAGE: &str =
    "gateway action request exhausted action retries without satisfying verifier requirements";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum GatewayMissionVerifierStatus {
    Passed,
    Continue,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct GatewayMissionVerifierRecord {
    pub(super) kind: String,
    pub(super) status: GatewayMissionVerifierStatus,
    pub(super) reason_code: String,
    pub(super) message: String,
    #[serde(default)]
    pub(super) details: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct GatewayMissionVerifierBundle {
    pub(super) overall: GatewayMissionVerifierRecord,
    pub(super) records: Vec<GatewayMissionVerifierRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GatewayVerifierToolTrace {
    pub(super) tool_name: String,
    pub(super) arguments: Value,
    pub(super) success: bool,
}

impl GatewayMissionVerifierBundle {
    pub(super) fn from_records(records: Vec<GatewayMissionVerifierRecord>) -> Self {
        let overall = overall_gateway_verifier_record(&records);
        Self { overall, records }
    }
}

pub(super) fn build_gateway_verifier_bundle(
    requires_tool_evidence: bool,
    requires_mutation_evidence: bool,
    requires_validation_evidence: bool,
    traces: &[GatewayVerifierToolTrace],
    retry_exhausted: bool,
) -> GatewayMissionVerifierBundle {
    let mut records = Vec::new();
    let evidence_traces = traces
        .iter()
        .filter(|trace| trace.tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME)
        .cloned()
        .collect::<Vec<_>>();

    if !requires_tool_evidence {
        records.push(gateway_verifier_record(
            "response_completion",
            GatewayMissionVerifierStatus::Passed,
            "conversation_completed",
            "non-mutating request completed without requiring tool evidence",
            [("observed_count", json!(0))],
        ));
        return GatewayMissionVerifierBundle::from_records(records);
    }

    let tool_execution_count = evidence_traces.len();
    records.push(if tool_execution_count > 0 {
        gateway_verifier_record(
            "action_tool_evidence",
            GatewayMissionVerifierStatus::Passed,
            "tool_execution_observed",
            format!(
                "observed {tool_execution_count} tool execution event(s) during mission execution"
            ),
            [("observed_count", json!(tool_execution_count))],
        )
    } else if retry_exhausted {
        gateway_verifier_record(
            "action_tool_evidence",
            GatewayMissionVerifierStatus::Failed,
            "tool_evidence_missing_exhausted",
            ACTION_TOOL_EVIDENCE_RETRY_EXHAUSTED_MESSAGE,
            [
                ("observed_count", json!(0)),
                ("retry_exhausted", json!(true)),
            ],
        )
    } else {
        gateway_verifier_record(
            "action_tool_evidence",
            GatewayMissionVerifierStatus::Continue,
            "tool_evidence_missing_continue",
            "no tool execution evidence observed yet; continue the outer loop",
            [
                ("observed_count", json!(0)),
                ("retry_exhausted", json!(false)),
            ],
        )
    });

    if requires_mutation_evidence {
        let mutation_count = evidence_traces
            .iter()
            .filter(|trace| trace.success && gateway_trace_is_mutating(trace))
            .count();
        records.push(if mutation_count > 0 {
            gateway_verifier_record(
                "workspace_mutation_evidence",
                GatewayMissionVerifierStatus::Passed,
                "mutation_evidence_observed",
                format!(
                    "observed {mutation_count} successful workspace-mutating tool execution(s)"
                ),
                [("observed_count", json!(mutation_count))],
            )
        } else if retry_exhausted {
            gateway_verifier_record(
                "workspace_mutation_evidence",
                GatewayMissionVerifierStatus::Failed,
                "mutation_evidence_missing_exhausted",
                "workspace-changing work was requested but no successful mutating evidence was observed before retries exhausted",
                [
                    ("observed_count", json!(0)),
                    ("retry_exhausted", json!(true)),
                ],
            )
        } else {
            gateway_verifier_record(
                "workspace_mutation_evidence",
                GatewayMissionVerifierStatus::Continue,
                "mutation_evidence_missing_continue",
                "workspace-changing work was requested but no successful mutating evidence has been observed yet",
                [
                    ("observed_count", json!(0)),
                    ("retry_exhausted", json!(false)),
                ],
            )
        });
    }

    if requires_validation_evidence {
        let validation_count = evidence_traces
            .iter()
            .filter(|trace| trace.success && gateway_trace_is_validation(trace))
            .count();
        records.push(if validation_count > 0 {
            gateway_verifier_record(
                "validation_evidence",
                GatewayMissionVerifierStatus::Passed,
                "validation_evidence_observed",
                format!(
                    "observed {validation_count} successful validation-oriented tool execution(s)"
                ),
                [("observed_count", json!(validation_count))],
            )
        } else if retry_exhausted {
            gateway_verifier_record(
                "validation_evidence",
                GatewayMissionVerifierStatus::Failed,
                "validation_evidence_missing_exhausted",
                "validation was requested but no successful validation evidence was observed before retries exhausted",
                [
                    ("observed_count", json!(0)),
                    ("retry_exhausted", json!(true)),
                ],
            )
        } else {
            gateway_verifier_record(
                "validation_evidence",
                GatewayMissionVerifierStatus::Continue,
                "validation_evidence_missing_continue",
                "validation was requested but no successful validation evidence has been observed yet",
                [
                    ("observed_count", json!(0)),
                    ("retry_exhausted", json!(false)),
                ],
            )
        });
    }

    GatewayMissionVerifierBundle::from_records(records)
}

pub(super) fn build_gateway_runtime_failure_verifier_bundle(
    reason_code: &str,
    message: &str,
) -> GatewayMissionVerifierBundle {
    GatewayMissionVerifierBundle::from_records(vec![gateway_verifier_record(
        "gateway_runtime",
        GatewayMissionVerifierStatus::Failed,
        reason_code,
        message,
        [("retry_exhausted", json!(false))],
    )])
}

pub(super) fn build_gateway_read_only_saturation_verifier_bundle(
    read_only_count: usize,
    threshold: usize,
) -> GatewayMissionVerifierBundle {
    GatewayMissionVerifierBundle::from_records(vec![
        gateway_verifier_record(
            "action_tool_evidence",
            GatewayMissionVerifierStatus::Passed,
            "tool_execution_observed",
            format!("observed {read_only_count} read-only tool execution event(s)"),
            [("observed_count", json!(read_only_count))],
        ),
        gateway_verifier_record(
            "workspace_mutation_evidence",
            GatewayMissionVerifierStatus::Continue,
            "read_only_saturation_continue",
            "read-only exploration saturated before any successful workspace mutation; retry with a workspace-mutating tool next",
            [
                ("observed_count", json!(0)),
                ("read_only_count", json!(read_only_count)),
                ("saturation_threshold", json!(threshold)),
                ("retry_exhausted", json!(false)),
            ],
        ),
    ])
}

pub(super) fn build_gateway_retry_feedback(bundle: &GatewayMissionVerifierBundle) -> String {
    let unresolved = bundle
        .records
        .iter()
        .filter(|record| record.status != GatewayMissionVerifierStatus::Passed)
        .map(|record| format!("- {}", record.message))
        .collect::<Vec<_>>();
    if unresolved.is_empty() {
        "The previous attempt is not complete yet. Continue the same task and satisfy the active verifier requirements before stopping.".to_string()
    } else {
        format!(
            "The previous attempt is not complete yet. Continue the same task and satisfy these verifier requirements before stopping:\n{}",
            unresolved.join("\n")
        )
    }
}

pub(super) fn overall_gateway_verifier_record(
    records: &[GatewayMissionVerifierRecord],
) -> GatewayMissionVerifierRecord {
    if let Some(record) = records
        .iter()
        .find(|record| record.status == GatewayMissionVerifierStatus::Failed)
    {
        return record.clone();
    }
    if let Some(record) = records
        .iter()
        .find(|record| record.status == GatewayMissionVerifierStatus::Continue)
    {
        return record.clone();
    }
    records.last().cloned().unwrap_or_else(|| {
        gateway_verifier_record(
            "response_completion",
            GatewayMissionVerifierStatus::Passed,
            "conversation_completed",
            "request completed without active verifiers",
            [("observed_count", json!(0))],
        )
    })
}

fn gateway_verifier_record(
    kind: &str,
    status: GatewayMissionVerifierStatus,
    reason_code: &str,
    message: impl Into<String>,
    details: impl IntoIterator<Item = (&'static str, Value)>,
) -> GatewayMissionVerifierRecord {
    GatewayMissionVerifierRecord {
        kind: kind.to_string(),
        status,
        reason_code: reason_code.to_string(),
        message: message.into(),
        details: details
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    }
}

pub(super) fn gateway_trace_is_mutating(trace: &GatewayVerifierToolTrace) -> bool {
    let tool_name = trace.tool_name.trim().to_ascii_lowercase();
    if matches!(
        tool_name.as_str(),
        "write" | "edit" | "delete" | "move" | "rename" | "copy" | "branch" | "undo" | "redo"
    ) {
        return true;
    }
    if tool_name.contains("write")
        || tool_name.contains("edit")
        || tool_name.contains("patch")
        || tool_name.contains("delete")
        || tool_name.contains("rename")
    {
        return true;
    }
    if tool_name == "bash" {
        return gateway_bash_command_contains_any(
            &trace.arguments,
            &[
                "mkdir",
                "touch",
                "mv ",
                "cp ",
                "rm ",
                "cargo new",
                "npm init",
                "pnpm create",
                "sed -i",
                "tee ",
                ">",
            ],
        );
    }
    false
}

pub(super) fn gateway_trace_is_validation(trace: &GatewayVerifierToolTrace) -> bool {
    let tool_name = trace.tool_name.trim().to_ascii_lowercase();
    if tool_name == "bash" {
        return gateway_bash_command_contains_any(
            &trace.arguments,
            &[
                "cargo test",
                "cargo check",
                "cargo build",
                "npm test",
                "pnpm test",
                "pnpm lint",
                "pnpm build",
                "npm run test",
                "pytest",
                "playwright",
                "vitest",
                "jest",
                "lint",
                "validate",
                "verify",
                "check",
                "build",
            ],
        );
    }
    tool_name.contains("test")
        || tool_name.contains("lint")
        || tool_name.contains("playwright")
        || tool_name.contains("verify")
        || tool_name.contains("validate")
        || tool_name.contains("build")
        || tool_name.contains("check")
}

fn gateway_bash_command_contains_any(arguments: &Value, needles: &[&str]) -> bool {
    let command = arguments
        .get("command")
        .or_else(|| arguments.get("cmd"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    needles.iter().any(|needle| command.contains(needle))
}
