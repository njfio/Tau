use super::*;
use async_trait::async_trait;
use serde_json::Value;
use tau_agent_core::{AgentTool, ToolExecutionResult};
use tau_ai::ToolDefinition;

#[derive(Clone, Copy)]
struct FixtureInventoryTool {
    name: &'static str,
    description: &'static str,
}

#[async_trait]
impl AgentTool for FixtureInventoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.to_string(),
            description: self.description.to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
            }),
        }
    }

    async fn execute(&self, _arguments: Value) -> ToolExecutionResult {
        ToolExecutionResult::ok(json!({"ok": true}))
    }
}

#[derive(Clone, Default)]
pub(super) struct FixtureGatewayToolRegistrar;

impl GatewayToolRegistrar for FixtureGatewayToolRegistrar {
    fn register(&self, agent: &mut Agent) {
        agent.register_tool(FixtureInventoryTool {
            name: "memory_search",
            description: "Searches memory entries.",
        });
        agent.register_tool(FixtureInventoryTool {
            name: "bash",
            description: "Runs shell commands.",
        });
    }
}

#[derive(Clone, Default)]
pub(super) struct FixtureGatewayMcpToolRegistrar;

impl GatewayToolRegistrar for FixtureGatewayMcpToolRegistrar {
    fn register(&self, agent: &mut Agent) {
        agent.register_tool(FixtureInventoryTool {
            name: "memory_search",
            description: "Searches memory entries.",
        });
        agent.register_tool(FixtureInventoryTool {
            name: "bash",
            description: "Runs shell commands.",
        });
        agent.register_tool(FixtureInventoryTool {
            name: "mcp.demo.echo",
            description: "Echoes payloads through MCP demo runtime.",
        });
    }
}
