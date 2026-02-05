use std::path::Path;

use async_trait::async_trait;
use pi_agent_core::{Agent, AgentTool, ToolExecutionResult};
use pi_ai::ToolDefinition;
use serde_json::{json, Value};

pub fn register_builtin_tools(agent: &mut Agent) {
    agent.register_tool(ReadTool);
    agent.register_tool(WriteTool);
    agent.register_tool(EditTool);
    agent.register_tool(BashTool);
}

pub struct ReadTool;

#[async_trait]
impl AgentTool for ReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read".to_string(),
            description: "Read a UTF-8 text file from disk".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to read" }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        match tokio::fs::read_to_string(&path).await {
            Ok(content) => ToolExecutionResult::ok(json!({
                "path": path,
                "content": content,
            })),
            Err(error) => ToolExecutionResult::error(json!({
                "path": path,
                "error": error.to_string(),
            })),
        }
    }
}

pub struct WriteTool;

#[async_trait]
impl AgentTool for WriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write".to_string(),
            description: "Write UTF-8 text to disk, creating parent directories if needed"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let content = match required_string(&arguments, "content") {
            Ok(content) => content,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        if let Some(parent) = Path::new(&path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(error) = tokio::fs::create_dir_all(parent).await {
                    return ToolExecutionResult::error(json!({
                        "path": path,
                        "error": format!("failed to create parent directory: {error}"),
                    }));
                }
            }
        }

        match tokio::fs::write(&path, content.as_bytes()).await {
            Ok(()) => ToolExecutionResult::ok(json!({
                "path": path,
                "bytes_written": content.len(),
            })),
            Err(error) => ToolExecutionResult::error(json!({
                "path": path,
                "error": error.to_string(),
            })),
        }
    }
}

pub struct EditTool;

#[async_trait]
impl AgentTool for EditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "edit".to_string(),
            description: "Edit a file by replacing an existing string".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "find": { "type": "string" },
                    "replace": { "type": "string" },
                    "all": { "type": "boolean", "default": false }
                },
                "required": ["path", "find", "replace"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let path = match required_string(&arguments, "path") {
            Ok(path) => path,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let find = match required_string(&arguments, "find") {
            Ok(find) => find,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let replace = match required_string(&arguments, "replace") {
            Ok(replace) => replace,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        if find.is_empty() {
            return ToolExecutionResult::error(json!({
                "path": path,
                "error": "'find' must not be empty",
            }));
        }

        let replace_all = arguments
            .get("all")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let source = match tokio::fs::read_to_string(&path).await {
            Ok(source) => source,
            Err(error) => {
                return ToolExecutionResult::error(json!({
                    "path": path,
                    "error": error.to_string(),
                }))
            }
        };

        let occurrences = source.matches(&find).count();
        if occurrences == 0 {
            return ToolExecutionResult::error(json!({
                "path": path,
                "error": "target string not found",
            }));
        }

        let updated = if replace_all {
            source.replace(&find, &replace)
        } else {
            source.replacen(&find, &replace, 1)
        };

        if let Err(error) = tokio::fs::write(&path, updated.as_bytes()).await {
            return ToolExecutionResult::error(json!({
                "path": path,
                "error": error.to_string(),
            }));
        }

        let replacements = if replace_all { occurrences } else { 1 };
        ToolExecutionResult::ok(json!({
            "path": path,
            "replacements": replacements,
        }))
    }
}

pub struct BashTool;

#[async_trait]
impl AgentTool for BashTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "cwd": { "type": "string" }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, arguments: Value) -> ToolExecutionResult {
        let command = match required_string(&arguments, "command") {
            Ok(command) => command,
            Err(error) => return ToolExecutionResult::error(json!({ "error": error })),
        };

        let cwd = arguments
            .get("cwd")
            .and_then(Value::as_str)
            .map(|value| value.to_string());

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        let mut process = tokio::process::Command::new(shell);
        process.arg("-lc").arg(&command);

        if let Some(cwd) = &cwd {
            process.current_dir(cwd);
        }

        match process.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                ToolExecutionResult::ok(json!({
                    "command": command,
                    "cwd": cwd,
                    "status": output.status.code(),
                    "success": output.status.success(),
                    "stdout": truncate(&stdout, 16_000),
                    "stderr": truncate(&stderr, 16_000),
                }))
            }
            Err(error) => ToolExecutionResult::error(json!({
                "command": command,
                "cwd": cwd,
                "error": error.to_string(),
            })),
        }
    }
}

fn required_string(arguments: &Value, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value.to_string())
        .ok_or_else(|| format!("missing required string argument '{key}'"))
}

fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_string();
    }

    let mut output = value[..limit].to_string();
    output.push_str("\n<output truncated>");
    output
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{AgentTool, BashTool, EditTool, ToolExecutionResult, WriteTool};

    #[tokio::test]
    async fn edit_tool_replaces_single_match() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("test.txt");
        tokio::fs::write(&file, "a a a").await.expect("write file");

        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "path": file,
                "find": "a",
                "replace": "b"
            }))
            .await;

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(temp.path().join("test.txt"))
            .await
            .expect("read file");
        assert_eq!(content, "b a a");
    }

    #[tokio::test]
    async fn edit_tool_replaces_all_matches() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("test.txt");
        tokio::fs::write(&file, "a a a").await.expect("write file");

        let tool = EditTool;
        let result = tool
            .execute(serde_json::json!({
                "path": file,
                "find": "a",
                "replace": "b",
                "all": true
            }))
            .await;

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(temp.path().join("test.txt"))
            .await
            .expect("read file");
        assert_eq!(content, "b b b");
    }

    #[tokio::test]
    async fn write_tool_creates_parent_directory() {
        let temp = tempdir().expect("tempdir");
        let file = temp.path().join("nested/output.txt");

        let tool = WriteTool;
        let result = tool
            .execute(serde_json::json!({
                "path": file,
                "content": "hello"
            }))
            .await;

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(temp.path().join("nested/output.txt"))
            .await
            .expect("read file");
        assert_eq!(content, "hello");
    }

    #[tokio::test]
    async fn bash_tool_runs_command() {
        let tool = BashTool;
        let result = tool
            .execute(serde_json::json!({ "command": "printf 'ok'" }))
            .await;

        assert!(!result.is_error);
        assert_eq!(
            result
                .content
                .get("stdout")
                .and_then(serde_json::Value::as_str),
            Some("ok")
        );
    }

    #[test]
    fn tool_result_text_serializes_json() {
        let result = ToolExecutionResult::ok(serde_json::json!({ "a": 1 }));
        assert!(result.as_text().contains('"'));
    }
}
