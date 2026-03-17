#[derive(Clone, Copy)]
pub(super) enum CommandId {
    Thinking,
    Details,
    Tools,
    Memory,
    Cortex,
    Sessions,
    ApprovalNeeded,
    Approve,
    Reject,
    Interrupt,
    Retry,
    Help,
    Clear,
    Quit,
}

pub(super) struct CommandSpec {
    pub command: CommandId,
    pub name: &'static str,
    pub summary: &'static str,
}

const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        command: CommandId::Thinking,
        name: "thinking",
        summary: "Show live turn context",
    },
    CommandSpec {
        command: CommandId::Details,
        name: "details",
        summary: "Toggle the detail drawer",
    },
    CommandSpec {
        command: CommandId::Tools,
        name: "tools",
        summary: "Open tool activity details",
    },
    CommandSpec {
        command: CommandId::Memory,
        name: "memory",
        summary: "Open memory context",
    },
    CommandSpec {
        command: CommandId::Cortex,
        name: "cortex",
        summary: "Open cortex/runtime posture",
    },
    CommandSpec {
        command: CommandId::Sessions,
        name: "sessions",
        summary: "Open session metrics",
    },
    CommandSpec {
        command: CommandId::ApprovalNeeded,
        name: "approval-needed",
        summary: "Simulate an approval request",
    },
    CommandSpec {
        command: CommandId::Approve,
        name: "approve",
        summary: "Approve the pending request",
    },
    CommandSpec {
        command: CommandId::Reject,
        name: "reject",
        summary: "Reject the pending request",
    },
    CommandSpec {
        command: CommandId::Interrupt,
        name: "interrupt",
        summary: "Interrupt the active turn",
    },
    CommandSpec {
        command: CommandId::Retry,
        name: "retry",
        summary: "Replay the last prompt",
    },
    CommandSpec {
        command: CommandId::Help,
        name: "help",
        summary: "Open keyboard help",
    },
    CommandSpec {
        command: CommandId::Clear,
        name: "clear",
        summary: "Clear the transcript",
    },
    CommandSpec {
        command: CommandId::Quit,
        name: "quit",
        summary: "Exit the TUI",
    },
    CommandSpec {
        command: CommandId::Quit,
        name: "q",
        summary: "Exit the TUI",
    },
];

pub(super) fn parse_command(text: &str) -> Option<CommandId> {
    let query = normalize_query(text);
    COMMAND_SPECS
        .iter()
        .find(|spec| spec.name == query)
        .map(|spec| spec.command)
}

pub(super) fn matching_commands(query: &str) -> Vec<&'static CommandSpec> {
    let query = normalize_query(query);
    if query.is_empty() {
        return COMMAND_SPECS.iter().collect();
    }
    let matches = COMMAND_SPECS
        .iter()
        .filter(|spec| spec.name.contains(query))
        .collect::<Vec<_>>();
    let (mut prefix_matches, contains_matches): (Vec<_>, Vec<_>) = matches
        .into_iter()
        .partition(|spec| spec.name.starts_with(query));
    prefix_matches.extend(contains_matches);
    prefix_matches
}

fn normalize_query(text: &str) -> &str {
    text.trim().trim_start_matches('/')
}
