//! Skill execution runtime for dispatching tool calls, hooks, and commands.
//!
//! Delegates to process execution (`std::process::Command`) or WASM sandbox
//! based on the skill's declared `SkillRuntime`.

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use crate::{Skill, SkillHook, SkillRuntime};

/// Context passed to lifecycle hook dispatch.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// The hook-specific payload (JSON object).
    pub payload: Value,
}

/// Dispatch a tool call to a skill's runtime.
///
/// Returns the tool result as a JSON value.
///
/// # Errors
///
/// Returns an error if the skill has no runtime configured, the tool name is
/// not found in the skill's tool definitions, or execution fails.
pub fn dispatch_skill_tool(
    skill: &Skill,
    tool_name: &str,
    params: Value,
) -> Result<Value> {
    let runtime = skill
        .runtime
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no runtime configured", skill.name))?;
    let entrypoint = skill
        .entrypoint
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no entrypoint configured", skill.name))?;

    // Verify the tool exists in the skill's tool definitions.
    let tools = skill.tools.as_ref().ok_or_else(|| {
        anyhow!(
            "skill '{}' declares no tools but tool '{}' was requested",
            skill.name,
            tool_name
        )
    })?;
    if !tools.iter().any(|t| t.name == tool_name) {
        bail!(
            "skill '{}' does not declare tool '{}'",
            skill.name,
            tool_name
        );
    }

    let request = serde_json::json!({
        "kind": "tool-call",
        "tool": {
            "name": tool_name,
            "arguments": params,
        },
        "skill_name": skill.name,
    });
    let request_json =
        serde_json::to_string(&request).map_err(|e| anyhow!("failed to serialize tool request: {e}"))?;

    execute_skill_runtime(runtime, entrypoint, &request_json)
}

/// Dispatch a lifecycle hook to a skill's runtime.
///
/// # Errors
///
/// Returns an error if the skill has no runtime configured, the hook is not
/// declared by the skill, or execution fails.
pub fn dispatch_skill_hook(
    skill: &Skill,
    hook: SkillHook,
    context: &HookContext,
) -> Result<()> {
    let runtime = skill
        .runtime
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no runtime configured", skill.name))?;
    let entrypoint = skill
        .entrypoint
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no entrypoint configured", skill.name))?;

    // Verify the hook is declared.
    let hooks = skill.hooks.as_ref().ok_or_else(|| {
        anyhow!(
            "skill '{}' declares no hooks but hook '{}' was requested",
            skill.name,
            hook.as_str()
        )
    })?;
    if !hooks.contains(&hook) {
        bail!(
            "skill '{}' does not declare hook '{}'",
            skill.name,
            hook.as_str()
        );
    }

    let request = serde_json::json!({
        "kind": "hook",
        "hook": hook.as_str(),
        "payload": context.payload,
        "skill_name": skill.name,
    });
    let request_json =
        serde_json::to_string(&request).map_err(|e| anyhow!("failed to serialize hook request: {e}"))?;

    let _response = execute_skill_runtime(runtime, entrypoint, &request_json)?;
    Ok(())
}

/// Dispatch a command invocation to a skill's runtime.
///
/// Returns the command output as a string.
///
/// # Errors
///
/// Returns an error if the skill has no runtime configured, the command is not
/// found in the skill's command definitions, or execution fails.
pub fn dispatch_skill_command(
    skill: &Skill,
    command_name: &str,
    args: &[String],
) -> Result<String> {
    let runtime = skill
        .runtime
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no runtime configured", skill.name))?;
    let entrypoint = skill
        .entrypoint
        .as_ref()
        .ok_or_else(|| anyhow!("skill '{}' has no entrypoint configured", skill.name))?;

    // Verify the command exists in the skill's command definitions.
    let commands = skill.commands.as_ref().ok_or_else(|| {
        anyhow!(
            "skill '{}' declares no commands but command '{}' was requested",
            skill.name,
            command_name
        )
    })?;
    if !commands.iter().any(|c| c.name == command_name) {
        bail!(
            "skill '{}' does not declare command '{}'",
            skill.name,
            command_name
        );
    }

    let request = serde_json::json!({
        "kind": "command-call",
        "command": {
            "name": command_name,
            "args": args,
        },
        "skill_name": skill.name,
    });
    let request_json = serde_json::to_string(&request)
        .map_err(|e| anyhow!("failed to serialize command request: {e}"))?;

    let response = execute_skill_runtime(runtime, entrypoint, &request_json)?;

    // Extract "output" field from response, or return the raw JSON.
    let output = response
        .get("output")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string(&response).unwrap_or_default());

    Ok(output)
}

/// Execute a skill's runtime with the given request payload.
///
/// For `Process` runtime: spawns the entrypoint as a process, writes request
/// JSON to stdin, reads response JSON from stdout.
///
/// For `Wasm` runtime: currently returns an error indicating WASM support is
/// not yet integrated at the skill level. When integrated, this will delegate
/// to the existing `tau-runtime` WASM sandbox.
fn execute_skill_runtime(
    runtime: &SkillRuntime,
    entrypoint: &str,
    request_json: &str,
) -> Result<Value> {
    match runtime {
        SkillRuntime::Process => execute_process_runtime(entrypoint, request_json),
        SkillRuntime::Wasm => {
            bail!(
                "WASM skill runtime is not yet supported; \
                 use process runtime or wait for WASM integration"
            )
        }
    }
}

/// Execute a process-based skill runtime.
fn execute_process_runtime(entrypoint: &str, request_json: &str) -> Result<Value> {
    let mut child = Command::new(entrypoint)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("failed to spawn skill process '{}': {}", entrypoint, e))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open skill process stdin"))?;
        stdin
            .write_all(request_json.as_bytes())
            .map_err(|e| anyhow!("failed to write to skill process stdin: {e}"))?;
        stdin
            .write_all(b"\n")
            .map_err(|e| anyhow!("failed to write newline to skill process stdin: {e}"))?;
        stdin
            .flush()
            .map_err(|e| anyhow!("failed to flush skill process stdin: {e}"))?;
    }
    // Drop stdin to signal EOF.
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| anyhow!("failed to wait for skill process: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "skill process '{}' exited with status {}: {}",
            entrypoint,
            output.status,
            stderr.trim()
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow!("skill process output is not valid UTF-8: {e}"))?;
    let response: Value = serde_json::from_str(stdout.trim())
        .map_err(|e| anyhow!("skill process response is not valid JSON: {e}"))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::{
        Skill, SkillCommandDefinition, SkillHook, SkillPermission, SkillRuntime,
        SkillToolDefinition,
    };

    /// Helper to build a minimal Skill with no runtime (for backward compat tests).
    fn minimal_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: String::new(),
            content: String::new(),
            path: PathBuf::from(format!("{name}.md")),
            base_dir: PathBuf::from("."),
            tools: None,
            commands: None,
            hooks: None,
            runtime: None,
            entrypoint: None,
            permissions: None,
        }
    }

    /// Helper to build a Skill with runtime and tools configured.
    fn skill_with_tools(name: &str, tools: Vec<SkillToolDefinition>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("{name} skill"),
            content: String::new(),
            path: PathBuf::from(format!("{name}.md")),
            base_dir: PathBuf::from("."),
            tools: Some(tools),
            commands: None,
            hooks: None,
            runtime: Some(SkillRuntime::Process),
            entrypoint: Some("/bin/echo".to_string()),
            permissions: Some(vec![SkillPermission::RunCommands]),
        }
    }

    // ---- Test 1: Skill struct deserialization with new optional fields ----

    #[test]
    fn test_skill_tool_definition_serialization_roundtrip() {
        let tool = SkillToolDefinition {
            name: "greet".to_string(),
            description: "Greet a user".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }),
            handler: Some("greet_handler".to_string()),
        };

        let serialized = serde_json::to_string(&tool).expect("serialize");
        let deserialized: SkillToolDefinition =
            serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn test_skill_tool_definition_without_handler_roundtrip() {
        let tool = SkillToolDefinition {
            name: "ping".to_string(),
            description: "Ping check".to_string(),
            parameters: json!({"type": "object"}),
            handler: None,
        };

        let serialized = serde_json::to_string(&tool).expect("serialize");
        let deserialized: SkillToolDefinition =
            serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn test_skill_command_definition_serialization_roundtrip() {
        let command = SkillCommandDefinition {
            name: "deploy".to_string(),
            description: "Deploy to environment".to_string(),
            template: Some("deploy {{env}}".to_string()),
            arguments: json!({"env": "staging"}),
        };

        let serialized = serde_json::to_string(&command).expect("serialize");
        let deserialized: SkillCommandDefinition =
            serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(command, deserialized);
    }

    #[test]
    fn test_skill_hook_serialization_roundtrip() {
        let hooks = vec![
            SkillHook::RunStart,
            SkillHook::RunEnd,
            SkillHook::PreToolCall,
            SkillHook::PostToolCall,
            SkillHook::MessageTransform,
            SkillHook::PolicyOverride,
        ];

        for hook in &hooks {
            let serialized = serde_json::to_string(hook).expect("serialize");
            let deserialized: SkillHook =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(*hook, deserialized);
        }
    }

    #[test]
    fn test_skill_runtime_serialization_roundtrip() {
        let process = SkillRuntime::Process;
        let wasm = SkillRuntime::Wasm;

        assert_eq!(
            serde_json::from_str::<SkillRuntime>(
                &serde_json::to_string(&process).expect("serialize")
            )
            .expect("deserialize"),
            process
        );
        assert_eq!(
            serde_json::from_str::<SkillRuntime>(
                &serde_json::to_string(&wasm).expect("serialize")
            )
            .expect("deserialize"),
            wasm
        );
    }

    #[test]
    fn test_skill_permission_serialization_roundtrip() {
        let permissions = vec![
            SkillPermission::ReadFiles,
            SkillPermission::WriteFiles,
            SkillPermission::RunCommands,
            SkillPermission::Network,
        ];

        for perm in &permissions {
            let serialized = serde_json::to_string(perm).expect("serialize");
            let deserialized: SkillPermission =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(*perm, deserialized);
        }
    }

    // ---- Test 2: Backward compatibility — existing skills without new fields ----

    #[test]
    fn test_existing_skills_without_new_fields_still_work() {
        let skill = minimal_skill("legacy");
        assert!(skill.tools.is_none());
        assert!(skill.commands.is_none());
        assert!(skill.hooks.is_none());
        assert!(skill.runtime.is_none());
        assert!(skill.entrypoint.is_none());
        assert!(skill.permissions.is_none());
        // The skill should still be usable for prompt augmentation.
        assert_eq!(skill.name, "legacy");
    }

    // ---- Test 3: dispatch_skill_tool returns error for skills without runtime ----

    #[test]
    fn test_dispatch_skill_tool_errors_without_runtime() {
        let skill = minimal_skill("no-runtime");
        let error = dispatch_skill_tool(&skill, "some-tool", json!({}))
            .expect_err("should fail without runtime");
        assert!(
            error.to_string().contains("has no runtime configured"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_dispatch_skill_tool_errors_without_entrypoint() {
        let mut skill = minimal_skill("no-entrypoint");
        skill.runtime = Some(SkillRuntime::Process);
        // No entrypoint set.
        let error = dispatch_skill_tool(&skill, "some-tool", json!({}))
            .expect_err("should fail without entrypoint");
        assert!(
            error.to_string().contains("has no entrypoint configured"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_dispatch_skill_tool_errors_when_no_tools_declared() {
        let mut skill = minimal_skill("no-tools");
        skill.runtime = Some(SkillRuntime::Process);
        skill.entrypoint = Some("/bin/true".to_string());
        // tools is None
        let error = dispatch_skill_tool(&skill, "missing", json!({}))
            .expect_err("should fail with no tools");
        assert!(
            error.to_string().contains("declares no tools"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_dispatch_skill_tool_errors_when_tool_not_found() {
        let skill = skill_with_tools(
            "has-tools",
            vec![SkillToolDefinition {
                name: "greet".to_string(),
                description: "Greet".to_string(),
                parameters: json!({"type": "object"}),
                handler: None,
            }],
        );
        let error = dispatch_skill_tool(&skill, "unknown-tool", json!({}))
            .expect_err("should fail with unknown tool");
        assert!(
            error.to_string().contains("does not declare tool 'unknown-tool'"),
            "unexpected error: {error}"
        );
    }

    // ---- Test 4: dispatch_skill_hook errors without runtime ----

    #[test]
    fn test_dispatch_skill_hook_errors_without_runtime() {
        let skill = minimal_skill("no-runtime");
        let context = HookContext {
            payload: json!({}),
        };
        let error = dispatch_skill_hook(&skill, SkillHook::RunStart, &context)
            .expect_err("should fail without runtime");
        assert!(
            error.to_string().contains("has no runtime configured"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_dispatch_skill_hook_errors_when_hook_not_declared() {
        let mut skill = minimal_skill("partial-hooks");
        skill.runtime = Some(SkillRuntime::Process);
        skill.entrypoint = Some("/bin/true".to_string());
        skill.hooks = Some(vec![SkillHook::RunStart]);
        let context = HookContext {
            payload: json!({}),
        };
        let error = dispatch_skill_hook(&skill, SkillHook::RunEnd, &context)
            .expect_err("should fail with undeclared hook");
        assert!(
            error.to_string().contains("does not declare hook 'run-end'"),
            "unexpected error: {error}"
        );
    }

    // ---- Test 5: dispatch_skill_command errors without runtime ----

    #[test]
    fn test_dispatch_skill_command_errors_without_runtime() {
        let skill = minimal_skill("no-runtime");
        let error = dispatch_skill_command(&skill, "deploy", &[])
            .expect_err("should fail without runtime");
        assert!(
            error.to_string().contains("has no runtime configured"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_dispatch_skill_command_errors_when_command_not_found() {
        let mut skill = minimal_skill("has-commands");
        skill.runtime = Some(SkillRuntime::Process);
        skill.entrypoint = Some("/bin/true".to_string());
        skill.commands = Some(vec![SkillCommandDefinition {
            name: "deploy".to_string(),
            description: "Deploy".to_string(),
            template: None,
            arguments: json!({}),
        }]);
        let error = dispatch_skill_command(&skill, "rollback", &[])
            .expect_err("should fail with unknown command");
        assert!(
            error.to_string().contains("does not declare command 'rollback'"),
            "unexpected error: {error}"
        );
    }

    // ---- Test 6: WASM runtime returns not-yet-supported error ----

    #[test]
    fn test_wasm_runtime_returns_not_supported_error() {
        let error = execute_skill_runtime(&SkillRuntime::Wasm, "module.wasm", "{}")
            .expect_err("WASM should not be supported yet");
        assert!(
            error.to_string().contains("WASM skill runtime is not yet supported"),
            "unexpected error: {error}"
        );
    }

    // ---- Test 7: SkillHook::as_str ----

    #[test]
    fn test_skill_hook_as_str_values() {
        assert_eq!(SkillHook::RunStart.as_str(), "run-start");
        assert_eq!(SkillHook::RunEnd.as_str(), "run-end");
        assert_eq!(SkillHook::PreToolCall.as_str(), "pre-tool-call");
        assert_eq!(SkillHook::PostToolCall.as_str(), "post-tool-call");
        assert_eq!(SkillHook::MessageTransform.as_str(), "message-transform");
        assert_eq!(SkillHook::PolicyOverride.as_str(), "policy-override");
    }

    // ---- Test 8: SkillRuntime::as_str ----

    #[test]
    fn test_skill_runtime_as_str_values() {
        assert_eq!(SkillRuntime::Process.as_str(), "process");
        assert_eq!(SkillRuntime::Wasm.as_str(), "wasm");
    }

    // ---- Test 9: SkillPermission::as_str ----

    #[test]
    fn test_skill_permission_as_str_values() {
        assert_eq!(SkillPermission::ReadFiles.as_str(), "read-files");
        assert_eq!(SkillPermission::WriteFiles.as_str(), "write-files");
        assert_eq!(SkillPermission::RunCommands.as_str(), "run-commands");
        assert_eq!(SkillPermission::Network.as_str(), "network");
    }
}
