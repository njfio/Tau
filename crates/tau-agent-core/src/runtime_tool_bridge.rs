//! Tool-bridge helpers for argument validation and tool execution.

use std::{sync::Arc, time::Duration};

use jsonschema::validator_for;
use serde_json::{json, Value};
use tau_ai::{ToolCall, ToolDefinition};

use crate::{AgentTool, CooperativeCancellationToken, ToolExecutionResult};

pub(crate) async fn execute_tool_call_inner(
    call: ToolCall,
    registered: Option<(ToolDefinition, Arc<dyn AgentTool>)>,
    tool_timeout: Option<Duration>,
    cancellation_token: Option<CooperativeCancellationToken>,
) -> ToolExecutionResult {
    let ToolCall {
        id: _,
        name: call_name,
        arguments: raw_arguments,
    } = call;

    if cancellation_token
        .as_ref()
        .map(CooperativeCancellationToken::is_cancelled)
        .unwrap_or(false)
    {
        return ToolExecutionResult::error(json!({
            "error": format!("tool '{}' cancelled before execution", call_name)
        }));
    }

    if let Some((definition, tool)) = registered {
        let arguments = normalize_tool_arguments(raw_arguments);
        if let Err(error) = validate_tool_arguments(&definition, &arguments) {
            return ToolExecutionResult::error(json!({ "error": error }));
        }

        let tool_name = definition.name.clone();
        let execution = async move {
            if let Some(timeout) = tool_timeout {
                match tokio::time::timeout(timeout, tool.execute(arguments)).await {
                    Ok(result) => result,
                    Err(_) => ToolExecutionResult::error(json!({
                        "error": format!(
                            "tool '{}' timed out after {}ms",
                            tool_name,
                            timeout.as_millis()
                        )
                    })),
                }
            } else {
                tool.execute(arguments).await
            }
        };

        if let Some(token) = cancellation_token {
            tokio::select! {
                _ = token.cancelled() => ToolExecutionResult::error(json!({
                    "error": format!("tool '{}' cancelled", definition.name)
                })),
                result = execution => result,
            }
        } else {
            execution.await
        }
    } else {
        ToolExecutionResult::error(json!({
            "error": format!("Tool '{}' is not registered", call_name)
        }))
    }
}

fn normalize_tool_arguments(arguments: Value) -> Value {
    match arguments {
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                serde_json::from_str(trimmed).unwrap_or(Value::String(raw))
            } else {
                Value::String(raw)
            }
        }
        other => other,
    }
}

pub(crate) fn validate_tool_arguments(
    definition: &ToolDefinition,
    arguments: &Value,
) -> Result<(), String> {
    let validator = validator_for(&definition.parameters)
        .map_err(|error| format!("invalid JSON schema for '{}': {error}", definition.name))?;

    let mut errors = validator.iter_errors(arguments);
    if let Some(first) = errors.next() {
        return Err(format!(
            "invalid arguments for '{}': {}",
            definition.name, first
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use serde_json::{json, Value};
    use tau_ai::{ToolCall, ToolDefinition};

    use super::execute_tool_call_inner;
    use crate::{AgentTool, ToolExecutionResult};

    struct CaptureTool {
        captured: Arc<Mutex<Option<Value>>>,
    }

    #[async_trait]
    impl AgentTool for CaptureTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "write".to_string(),
                description: "Write content to a file".to_string(),
                parameters: json!({
                    "type": "object",
                    "required": ["path", "content"],
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    }
                }),
            }
        }

        async fn execute(&self, arguments: Value) -> ToolExecutionResult {
            *self
                .captured
                .lock()
                .expect("capture mutex should not be poisoned") = Some(arguments.clone());
            ToolExecutionResult::ok(arguments)
        }
    }

    #[tokio::test]
    async fn regression_execute_tool_call_inner_parses_stringified_json_arguments() {
        let captured = Arc::new(Mutex::new(None));
        let tool = Arc::new(CaptureTool {
            captured: captured.clone(),
        });
        let definition = tool.definition();
        let call = ToolCall {
            id: "call_write_1".to_string(),
            name: "write".to_string(),
            arguments: Value::String("{\"path\":\"README.md\",\"content\":\"hello\"}".to_string()),
        };

        let result = execute_tool_call_inner(call, Some((definition, tool)), None, None).await;

        assert!(
            !result.is_error,
            "stringified JSON arguments should be normalized before schema validation: {}",
            result.as_text()
        );
        let captured_arguments = captured
            .lock()
            .expect("capture mutex should not be poisoned")
            .clone()
            .expect("tool should execute with parsed arguments");
        assert_eq!(captured_arguments["path"], "README.md");
        assert_eq!(captured_arguments["content"], "hello");
    }
}
