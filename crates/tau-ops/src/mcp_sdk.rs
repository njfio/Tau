//! B9: MCP SDK documentation stubs.
//!
//! Provides a documentation generator for Tau's MCP tool surface,
//! producing markdown documentation from the tool schemas.

/// Generate markdown documentation for Tau's MCP tool surface.
///
/// This produces a structured reference covering all tool categories
/// that Tau exposes via the Model Context Protocol.
pub fn generate_mcp_tool_docs() -> String {
    let mut docs = String::from("# Tau MCP Tool Reference\n\n");
    docs.push_str("Tau exposes 33+ tools via the Model Context Protocol.\n\n");
    docs.push_str("## Tool Categories\n\n");
    docs.push_str("### File I/O\n- `tau.read` - Read a file\n- `tau.write` - Write a file\n- `tau.edit` - Edit a file\n\n");
    docs.push_str("### Memory\n- `tau.memory_write` - Write to memory\n- `tau.memory_read` - Read from memory\n- `tau.memory_search` - Search memory\n- `tau.memory_tree` - Browse memory tree\n\n");
    docs.push_str("### Sessions\n- `tau.session_list` - List sessions\n- `tau.session_resume` - Resume a session\n- `tau.session_search` - Search sessions\n- `tau.session_stats` - Session statistics\n- `tau.session_export` - Export session\n\n");
    docs.push_str("### Learning\n- `tau.learn_status` - Learning insights\n- `tau.learn_failure_patterns` - Failure patterns\n- `tau.learn_tool_rates` - Tool success rates\n\n");
    docs.push_str("### Training\n- `tau.training_status` - Training pipeline status\n- `tau.training_trigger` - Trigger APO optimization\n\n");
    docs.push_str("### Skills\n- `tau.skills_list` - List skills\n- `tau.skills_search` - Search skills\n- `tau.skills_install` - Install a skill\n- `tau.skills_info` - Skill details\n\n");
    docs.push_str("### Orchestration\n- `tau.agent_spawn` - Spawn sub-agent\n- `tau.agent_status` - Agent status\n- `tau.agent_cancel` - Cancel agent\n\n");
    docs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_mcp_tool_docs_produces_non_empty_markdown() {
        let docs = generate_mcp_tool_docs();
        assert!(!docs.is_empty());
        assert!(docs.starts_with("# Tau MCP Tool Reference"));
    }

    #[test]
    fn generate_mcp_tool_docs_contains_all_sections() {
        let docs = generate_mcp_tool_docs();
        assert!(docs.contains("### File I/O"));
        assert!(docs.contains("### Memory"));
        assert!(docs.contains("### Sessions"));
        assert!(docs.contains("### Learning"));
        assert!(docs.contains("### Training"));
        assert!(docs.contains("### Skills"));
        assert!(docs.contains("### Orchestration"));
    }

    #[test]
    fn generate_mcp_tool_docs_mentions_tool_count() {
        let docs = generate_mcp_tool_docs();
        assert!(docs.contains("33+"));
    }

    #[test]
    fn generate_mcp_tool_docs_contains_tool_names() {
        let docs = generate_mcp_tool_docs();
        assert!(docs.contains("tau.read"));
        assert!(docs.contains("tau.write"));
        assert!(docs.contains("tau.memory_write"));
        assert!(docs.contains("tau.session_list"));
        assert!(docs.contains("tau.learn_status"));
        assert!(docs.contains("tau.training_status"));
        assert!(docs.contains("tau.skills_list"));
        assert!(docs.contains("tau.agent_spawn"));
    }
}
