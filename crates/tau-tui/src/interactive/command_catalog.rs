pub(super) struct CommandSpec {
    pub name: &'static str,
    pub summary: &'static str,
}

const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "thinking",
        summary: "Show live turn context",
    },
    CommandSpec {
        name: "details",
        summary: "Toggle the detail drawer",
    },
    CommandSpec {
        name: "tools",
        summary: "Open tool activity details",
    },
    CommandSpec {
        name: "memory",
        summary: "Open memory context",
    },
    CommandSpec {
        name: "cortex",
        summary: "Open cortex/runtime posture",
    },
    CommandSpec {
        name: "sessions",
        summary: "Open session metrics",
    },
    CommandSpec {
        name: "approval-needed",
        summary: "Simulate an approval request",
    },
    CommandSpec {
        name: "approve",
        summary: "Approve the pending request",
    },
    CommandSpec {
        name: "reject",
        summary: "Reject the pending request",
    },
    CommandSpec {
        name: "interrupt",
        summary: "Interrupt the active turn",
    },
    CommandSpec {
        name: "retry",
        summary: "Replay the last prompt",
    },
    CommandSpec {
        name: "help",
        summary: "Open keyboard help",
    },
    CommandSpec {
        name: "clear",
        summary: "Clear the transcript",
    },
    CommandSpec {
        name: "quit",
        summary: "Exit the TUI",
    },
    CommandSpec {
        name: "q",
        summary: "Exit the TUI",
    },
];

pub(super) fn is_known_command(text: &str) -> bool {
    let query = normalize_query(text);
    COMMAND_SPECS.iter().any(|spec| spec.name == query)
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
