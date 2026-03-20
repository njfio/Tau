use super::status::AgentStateDisplay;
use super::tools::{ToolEntry, ToolStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvidenceStatus {
    Missing,
    ReadOnly,
    Mutating,
}

impl BuildEvidenceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Missing => "no mutating evidence yet",
            Self::ReadOnly => "read-only so far",
            Self::Mutating => "mutating evidence confirmed",
        }
    }
}

pub fn current_build_status(
    agent_state: AgentStateDisplay,
    prompt: Option<&str>,
    tools: &[ToolEntry],
) -> Option<BuildEvidenceStatus> {
    if agent_state == AgentStateDisplay::Idle || !is_build_prompt(prompt) {
        return None;
    }
    Some(classify_tool_evidence(tools))
}

fn is_build_prompt(prompt: Option<&str>) -> bool {
    let Some(prompt) = prompt else {
        return false;
    };
    prompt
        .split(|c: char| !c.is_alphanumeric())
        .any(is_build_verb)
}

fn is_build_verb(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "build" | "create" | "implement" | "make"
    )
}

fn classify_tool_evidence(tools: &[ToolEntry]) -> BuildEvidenceStatus {
    if has_successful_mutation(tools) {
        return BuildEvidenceStatus::Mutating;
    }
    if has_successful_tool(tools) {
        return BuildEvidenceStatus::ReadOnly;
    }
    BuildEvidenceStatus::Missing
}

fn has_successful_mutation(tools: &[ToolEntry]) -> bool {
    tools.iter().any(ToolEntry::is_successful_mutation)
}

fn has_successful_tool(tools: &[ToolEntry]) -> bool {
    tools
        .iter()
        .any(|entry| entry.status == ToolStatus::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, status: ToolStatus) -> ToolEntry {
        ToolEntry {
            name: name.to_string(),
            status,
            detail: String::new(),
            timestamp: String::new(),
        }
    }

    #[test]
    fn unit_build_status_ignores_idle_turns() {
        let status = current_build_status(AgentStateDisplay::Idle, Some("create a game"), &[]);
        assert_eq!(status, None);
    }

    #[test]
    fn unit_build_status_reports_missing_without_successes() {
        let status = current_build_status(
            AgentStateDisplay::Thinking,
            Some("create a game"),
            &[entry("read", ToolStatus::Running)],
        );
        assert_eq!(status, Some(BuildEvidenceStatus::Missing));
    }

    #[test]
    fn unit_build_status_reports_read_only_after_read_success() {
        let status = current_build_status(
            AgentStateDisplay::ToolExec,
            Some("build a game"),
            &[entry("read", ToolStatus::Success)],
        );
        assert_eq!(status, Some(BuildEvidenceStatus::ReadOnly));
    }

    #[test]
    fn unit_build_status_reports_mutating_after_write_success() {
        let status = current_build_status(
            AgentStateDisplay::ToolExec,
            Some("build a game"),
            &[entry("write", ToolStatus::Success)],
        );
        assert_eq!(status, Some(BuildEvidenceStatus::Mutating));
    }

    #[test]
    fn unit_build_status_ignores_non_build_words() {
        let status = current_build_status(
            AgentStateDisplay::Thinking,
            Some("what is blue"),
            &[entry("read", ToolStatus::Success)],
        );
        assert_eq!(status, None);
    }
}
