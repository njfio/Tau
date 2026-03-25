//! Command catalog and usage contract constants for operator CLI surfaces.
//!
//! Centralizes command usage/help strings so admin/report commands present a
//! consistent contract. This module is the source of truth for command discovery
//! and prevents drift between runtime behavior and operator guidance.

use tau_access::approvals::APPROVALS_USAGE;
use tau_access::rbac::RBAC_USAGE;
use tau_cli::CommandSpec;
use tau_release_channel::RELEASE_CHANNEL_USAGE;

use crate::{CANVAS_USAGE, QA_LOOP_USAGE};

/// Public `const` `MODELS_LIST_USAGE` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub const MODELS_LIST_USAGE: &str = "/models-list [query] [--provider <name>] [--tools <true|false>] [--multimodal <true|false>] [--reasoning <true|false>] [--limit <n>]";
/// Public `const` `MODEL_SHOW_USAGE` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub const MODEL_SHOW_USAGE: &str = "/model-show <provider/model>";

/// Public `const` `COMMAND_SPECS` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "/help",
        usage: "/help [command]",
        description: "Show command list or detailed command help",
        details: "Use '/help /command' (or '/help command') for command-specific guidance.",
        example: "/help /branch",
    },
    CommandSpec {
        name: "/session",
        usage: "/session",
        description: "Show session path, entry count, and active head id",
        details: "Read-only command; does not mutate session state.",
        example: "/session",
    },
    CommandSpec {
        name: "/session-search",
        usage: "/session-search <query> [--role <role>] [--limit <n>]",
        description: "Search session entries by role/text across all branches",
        details:
            "Case-insensitive search over role and message text. Optional --role scopes by message role and --limit controls displayed row count.",
        example: "/session-search retry budget --role user --limit 10",
    },
    CommandSpec {
        name: "/session-stats",
        usage: "/session-stats [--json]",
        description: "Summarize session size, branch tips, depth, and role counts",
        details:
            "Read-only session graph diagnostics including active/latest head indicators. Add --json for machine-readable output.",
        example: "/session-stats",
    },
    CommandSpec {
        name: "/session-diff",
        usage: "/session-diff [<left-id> <right-id>]",
        description: "Compare two session lineage heads",
        details:
            "Defaults to active vs latest head when ids are omitted. Includes shared and divergent lineage rows with deterministic previews.",
        example: "/session-diff 12 24",
    },
    CommandSpec {
        name: "/qa-loop",
        usage: QA_LOOP_USAGE,
        description: "Run staged quality checks with deterministic text/json reports",
        details:
            "Runs configured stages with timeout/retry controls, bounded stdout/stderr capture, and git changed-file summary.",
        example: "/qa-loop --json",
    },
    CommandSpec {
        name: "/doctor",
        usage: "/doctor [--json] [--online]",
        description: "Run deterministic runtime diagnostics",
        details:
            "Checks provider auth/session/skills/release posture. Add --json for machine-readable output and --online to include remote release checks.",
        example: "/doctor",
    },
    CommandSpec {
        name: "/release-channel",
        usage: RELEASE_CHANNEL_USAGE,
        description: "Show or persist release track selection",
        details:
            "Supports stable/beta/dev release tracks, update plan/apply with dry-run guardrails, and cache show/clear/refresh/prune operations in project-local .tau metadata.",
        example: "/release-channel set beta",
    },
    CommandSpec {
        name: "/session-graph-export",
        usage: "/session-graph-export <path>",
        description: "Export session graph as Mermaid or DOT file",
        details:
            "Uses .dot extension for Graphviz DOT; defaults to Mermaid for other destinations.",
        example: "/session-graph-export /tmp/session-graph.mmd",
    },
    CommandSpec {
        name: "/session-export",
        usage: "/session-export <path>",
        description: "Export active lineage snapshot to a JSONL file",
        details: "Writes only the active lineage entries, including schema metadata.",
        example: "/session-export /tmp/session-snapshot.jsonl",
    },
    CommandSpec {
        name: "/session-import",
        usage: "/session-import <path>",
        description: "Import a lineage snapshot into the current session",
        details:
            "Uses --session-import-mode (merge or replace). Merge remaps colliding ids; replace overwrites current entries.",
        example: "/session-import /tmp/session-snapshot.jsonl",
    },
    CommandSpec {
        name: "/session-merge",
        usage: "/session-merge <source-id> [target-id] [--strategy <append|squash|fast-forward>]",
        description: "Merge one branch head into another using explicit strategy",
        details:
            "Defaults target-id to the active head when omitted. append replays source-only entries, squash writes one summary entry, fast-forward requires target ancestry.",
        example: "/session-merge 42 24 --strategy squash",
    },
    CommandSpec {
        name: "/policy",
        usage: "/policy",
        description: "Print the effective tool policy JSON",
        details: "Useful for debugging allowlists, limits, and sandbox settings.",
        example: "/policy",
    },
    CommandSpec {
        name: "/audit-summary",
        usage: "/audit-summary <path>",
        description: "Summarize tool and telemetry JSONL audit records",
        details:
            "Aggregates tool/provider counts, error rates, and p50/p95 durations from audit log files.",
        example: "/audit-summary .tau/audit/tool-events.jsonl",
    },
    CommandSpec {
        name: "/models-list",
        usage: MODELS_LIST_USAGE,
        description: "List/search/filter model catalog entries and capabilities",
        details:
            "Supports provider and capability filters (`--tools`, `--multimodal`, `--reasoning`) plus text query and result limits.",
        example: "/models-list gpt --provider openai --tools true --limit 20",
    },
    CommandSpec {
        name: "/model-show",
        usage: MODEL_SHOW_USAGE,
        description: "Show detailed capability metadata for one model catalog entry",
        details:
            "Accepts provider/model format and reports context window, capability flags, and configured cost metadata.",
        example: "/model-show openai/gpt-5.2",
    },
    CommandSpec {
        name: "/skills-search",
        usage: "/skills-search <query> [max_results]",
        description: "Search installed skills by name and content",
        details: "Ranks name matches first, then content-only matches.",
        example: "/skills-search checklist 10",
    },
    CommandSpec {
        name: "/skills-show",
        usage: "/skills-show <name>",
        description: "Display installed skill content and metadata",
        details: "Read-only command for inspecting a single skill by name.",
        example: "/skills-show checklist",
    },
    CommandSpec {
        name: "/skills-list",
        usage: "/skills-list",
        description: "List installed skills in the active skills directory",
        details: "Read-only command that prints deterministic skill inventory output.",
        example: "/skills-list",
    },
    CommandSpec {
        name: "/skills-lock-diff",
        usage: "/skills-lock-diff [lockfile_path] [--json]",
        description: "Inspect lockfile drift without enforcing sync",
        details: "Reports in-sync/drift/error status and supports structured JSON output.",
        example: "/skills-lock-diff .tau/skills/skills.lock.json --json",
    },
    CommandSpec {
        name: "/skills-prune",
        usage: "/skills-prune [lockfile_path] [--dry-run|--apply]",
        description: "Prune installed skills not tracked in lockfile",
        details:
            "Dry-run is default; use --apply to delete prune candidates after deterministic listing.",
        example: "/skills-prune .tau/skills/skills.lock.json --apply",
    },
    CommandSpec {
        name: "/skills-trust-list",
        usage: "/skills-trust-list [trust_root_file]",
        description: "List trust-root keys with revocation/expiry/rotation status",
        details:
            "Uses configured --skill-trust-root-file when no path argument is provided.",
        example: "/skills-trust-list .tau/skills/trust-roots.json",
    },
    CommandSpec {
        name: "/skills-trust-add",
        usage: "/skills-trust-add <id=base64_key> [trust_root_file]",
        description: "Add or update a trust-root key",
        details:
            "Mutates trust-root file atomically. Uses configured --skill-trust-root-file when path is omitted.",
        example: "/skills-trust-add root-v2=AbC... .tau/skills/trust-roots.json",
    },
    CommandSpec {
        name: "/skills-trust-revoke",
        usage: "/skills-trust-revoke <id> [trust_root_file]",
        description: "Revoke a trust-root key id",
        details:
            "Marks key as revoked in trust-root file. Uses configured --skill-trust-root-file when path is omitted.",
        example: "/skills-trust-revoke root-v1 .tau/skills/trust-roots.json",
    },
    CommandSpec {
        name: "/skills-trust-rotate",
        usage: "/skills-trust-rotate <old_id:new_id=base64_key> [trust_root_file]",
        description: "Rotate trust-root key id to a new key",
        details:
            "Revokes old id and adds/updates new id atomically. Uses configured --skill-trust-root-file when path is omitted.",
        example: "/skills-trust-rotate root-v1:root-v2=AbC... .tau/skills/trust-roots.json",
    },
    CommandSpec {
        name: "/skills-lock-write",
        usage: "/skills-lock-write [lockfile_path]",
        description: "Write/update skills lockfile from installed skills",
        details:
            "Uses <skills-dir>/skills.lock.json when path is omitted. Preserves existing source metadata when possible.",
        example: "/skills-lock-write .tau/skills/skills.lock.json",
    },
    CommandSpec {
        name: "/skills-sync",
        usage: "/skills-sync [lockfile_path]",
        description: "Validate installed skills against the lockfile",
        details:
            "Uses <skills-dir>/skills.lock.json when path is omitted. Prints drift diagnostics without exiting interactive mode.",
        example: "/skills-sync .tau/skills/skills.lock.json",
    },
    CommandSpec {
        name: "/skills-verify",
        usage: "/skills-verify [lockfile_path] [trust_root_file] [--json]",
        description: "Audit lockfile drift and trust/signature policy in one report",
        details:
            "Read-only compliance diagnostics across sync drift, signature metadata, and trust-root key status.",
        example: "/skills-verify .tau/skills/skills.lock.json .tau/skills/trust-roots.json --json",
    },
    CommandSpec {
        name: "/ops-verify",
        usage: "/ops-verify [--json]",
        description: "Run full ops verification gate checks",
        details:
            "Executes all configured verification gates and reports pass/fail status with optional JSON output.",
        example: "/ops-verify --json",
    },
    CommandSpec {
        name: "/ops-validate",
        usage: "/ops-validate [--json]",
        description: "Validate ops configuration and environment prerequisites",
        details:
            "Checks configuration files, environment variables, and dependency availability without executing gates.",
        example: "/ops-validate --json",
    },
    CommandSpec {
        name: "/ops-dev",
        usage: "/ops-dev [--watch] [--filter <gate>]",
        description: "Run verification gates in development mode with optional watch",
        details:
            "Supports iterative development with file-watch re-execution and gate filtering for focused feedback loops.",
        example: "/ops-dev --watch --filter lint",
    },
    CommandSpec {
        name: "/branches",
        usage: "/branches",
        description: "List branch tips in the current session graph",
        details: "Each row includes entry id, parent id, and a short message summary.",
        example: "/branches",
    },
    CommandSpec {
        name: "/macro",
        usage: "/macro <save|run|list|show|delete> ...",
        description: "Manage reusable command macros",
        details:
            "Persists macros in project-local config and supports dry-run validation, inspection, and deletion.",
        example: "/macro save quick-check /tmp/quick-check.commands",
    },
    CommandSpec {
        name: "/auth",
        usage: "/auth <login|reauth|status|logout|matrix|rotate-key> ...",
        description: "Manage provider authentication state and credential-store sessions",
        details:
            "Supports login/reauth/status/logout flows, credential-store key rotation, and provider-mode matrix diagnostics with optional --json output.",
        example: "/auth rotate-key --new-key next-store-key --json",
    },
    CommandSpec {
        name: "/canvas",
        usage: CANVAS_USAGE,
        description: "Manage collaborative live-canvas state backed by Yrs CRDT persistence",
        details:
            "Supports create/update/show/export/import flows for node and edge state with deterministic markdown/json renderers and replay-safe event envelopes.",
        example: "/canvas update architecture node-upsert api \"API Service\" 120 64",
    },
    CommandSpec {
        name: "/rbac",
        usage: RBAC_USAGE,
        description: "Inspect RBAC principal resolution and authorization decisions",
        details:
            "Use whoami to resolve principal bindings and check to evaluate one action against active role policy.",
        example: "/rbac check command:/policy --json",
    },
    CommandSpec {
        name: "/approvals",
        usage: APPROVALS_USAGE,
        description: "Review and decide queued HITL approval requests",
        details:
            "Use list for queue visibility and approve/reject to unblock or deny pending requests.",
        example: "/approvals list --status pending",
    },
    CommandSpec {
        name: "/integration-auth",
        usage: "/integration-auth <set|status|rotate|revoke> ...",
        description: "Manage credential-store secrets for integrations (GitHub, Slack, webhooks)",
        details:
            "Supports set/status/rotate/revoke flows for integration secret ids with optional --json output.",
        example: "/integration-auth status github-token --json",
    },
    CommandSpec {
        name: "/pair",
        usage: "/pair <add|remove|status> ...",
        description: "Manage remote channel actor pairings and allowlist visibility",
        details:
            "Writes `.tau/security/pairings.json` atomically. Use `/pair status` to inspect effective pairings and allowlist rows.",
        example: "/pair add github:owner/repo alice --ttl-seconds 3600",
    },
    CommandSpec {
        name: "/unpair",
        usage: "/unpair <channel> <actor_id>",
        description: "Remove one actor pairing from a channel",
        details:
            "Alias for `/pair remove` with deterministic removal count and atomic persistence.",
        example: "/unpair github:owner/repo alice",
    },
    CommandSpec {
        name: "/profile",
        usage: "/profile <save|load|list|show|delete> ...",
        description: "Manage model, policy, and session default profiles",
        details:
            "Profiles are persisted in project-local config. Load reports diffs from current defaults; list/show/delete support lifecycle management.",
        example: "/profile save baseline",
    },
    CommandSpec {
        name: "/branch-alias",
        usage: "/branch-alias <set|list|use> ...",
        description: "Manage persistent branch aliases for quick navigation",
        details:
            "Aliases are stored in a sidecar JSON file next to the active session file.",
        example: "/branch-alias set hotfix 42",
    },
    CommandSpec {
        name: "/session-bookmark",
        usage: "/session-bookmark <set|list|use|delete> ...",
        description: "Manage persistent session bookmarks",
        details:
            "Bookmarks are stored in project-local metadata and can switch active head by name.",
        example: "/session-bookmark set investigation 42",
    },
    CommandSpec {
        name: "/branch",
        usage: "/branch <id>",
        description: "Switch active branch head to a specific entry id",
        details: "Reloads the agent message context to the selected lineage.",
        example: "/branch 12",
    },
    CommandSpec {
        name: "/undo",
        usage: "/undo",
        description: "Rewind active session head to previous navigation target",
        details:
            "Uses persisted navigation history and skips stale targets removed by repair/compact operations.",
        example: "/undo",
    },
    CommandSpec {
        name: "/redo",
        usage: "/redo",
        description: "Re-apply one previously undone session head navigation step",
        details:
            "Available after a successful undo. Uses persisted navigation history and reports empty-stack diagnostics when unavailable.",
        example: "/redo",
    },
    CommandSpec {
        name: "/resume",
        usage: "/resume",
        description: "Jump back to the latest session head",
        details: "Resets active branch to current head and reloads lineage messages.",
        example: "/resume",
    },
    CommandSpec {
        name: "/session-repair",
        usage: "/session-repair",
        description: "Repair malformed session graphs",
        details: "Removes duplicate ids, invalid parent references, and cyclic lineage entries.",
        example: "/session-repair",
    },
    CommandSpec {
        name: "/session-compact",
        usage: "/session-compact",
        description: "Compact session to active lineage",
        details: "Prunes inactive branches and retains only entries reachable from active head.",
        example: "/session-compact",
    },
    CommandSpec {
        name: "/training-status",
        usage: "/training-status [--json]",
        description: "Show live RL training bridge status and rollout statistics",
        details:
            "Reports bridge enablement, gate state, completed rollouts, failure streak, last optimizer report, and APO auto-trigger threshold progress. Add --json for machine-readable output.",
        example: "/training-status --json",
    },
    CommandSpec {
        name: "/training-trigger",
        usage: "/training-trigger [--dry-run]",
        description: "Manually trigger an APO optimization pass over collected rollouts",
        details:
            "Forces an optimizer update regardless of the update interval. Use --dry-run to preview sample collection without executing the optimization pass.",
        example: "/training-trigger --dry-run",
    },
    CommandSpec {
        name: "/self-modify-status",
        usage: "/self-modify-status [--json]",
        description: "Show recent self-modification proposals and their outcomes",
        details:
            "Lists recent self-modification proposals with approval status, blocked rules, and timestamps. Add --json for machine-readable output.",
        example: "/self-modify-status --json",
    },
    CommandSpec {
        name: "/self-modify-review",
        usage: "/self-modify-review [proposal-id]",
        description: "Review pending self-modification proposals",
        details:
            "Shows diff, rationale, trigger source, and safety evaluation for pending proposals. Omit id to list all pending.",
        example: "/self-modify-review 42",
    },
    CommandSpec {
        name: "/self-modify-rollback",
        usage: "/self-modify-rollback <proposal-id>",
        description: "Revert a previously applied self-modification",
        details:
            "Applies the reverse diff of a completed self-modification proposal and records the rollback in the audit log.",
        example: "/self-modify-rollback 42",
    },
    CommandSpec {
        name: "/self-modify-policy",
        usage: "/self-modify-policy [--show|--add-rule <json>|--disable-rule <id>|--enable-rule <id>]",
        description: "Show or configure the self-modification safety policy",
        details:
            "Displays active self-modification rules and allows adding, enabling, or disabling rules at runtime. Changes persist in project-local .tau metadata.",
        example: "/self-modify-policy --show",
    },
    CommandSpec {
        name: "/learn-status",
        usage: "/learn-status [--json]",
        description: "Show action history stats, failure patterns, and tool success rates",
        details:
            "Read-only command that summarizes total recorded actions, top failure patterns with occurrence counts, and per-tool success rates from the action history store. Add --json for machine-readable output.",
        example: "/learn-status --json",
    },
    CommandSpec {
        name: "/learn-clear",
        usage: "/learn-clear [--confirm]",
        description: "Clear all learned action history data",
        details:
            "Removes all persisted action history records including failure patterns and tool effectiveness data. Requires --confirm flag to execute; without it, prints a dry-run summary of what would be cleared.",
        example: "/learn-clear --confirm",
    },
    CommandSpec {
        name: "/learn-export",
        usage: "/learn-export <path>",
        description: "Export learned action history data to a JSON file",
        details:
            "Writes the full action history store contents — including failure patterns, tool effectiveness, and session feedback — to the specified path as formatted JSON.",
        example: "/learn-export /tmp/tau-learned-data.json",
    },
    CommandSpec {
        name: "/init",
        usage: "/init [--auto] [--from-env]",
        description: "Initialize agent configuration by generating a .tau.toml file",
        details:
            "Interactive mode asks key questions. Use --auto for sensible defaults with no prompts, or --from-env to generate from current environment variables.",
        example: "/init --auto",
    },
    CommandSpec {
        name: "/config-validate",
        usage: "/config-validate [path]",
        description: "Validate a .tau.toml configuration file against the schema",
        details:
            "Parses the config file and reports any errors. Uses .tau.toml in the current directory when path is omitted.",
        example: "/config-validate .tau.toml",
    },
    CommandSpec {
        name: "/config-show",
        usage: "/config-show [--json]",
        description: "Show the resolved agent configuration",
        details:
            "Displays the effective configuration merged from .tau.toml, environment variables, and CLI flags. Add --json for machine-readable output.",
        example: "/config-show --json",
    },
    CommandSpec {
        name: "/quit",
        usage: "/quit",
        description: "Exit interactive mode",
        details: "Alias: /exit",
        example: "/quit",
    },
];

/// Public `const` `COMMAND_NAMES` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub const COMMAND_NAMES: &[&str] = &[
    "/help",
    "/session",
    "/session-search",
    "/session-stats",
    "/session-diff",
    "/qa-loop",
    "/doctor",
    "/release-channel",
    "/session-graph-export",
    "/session-export",
    "/session-import",
    "/session-merge",
    "/policy",
    "/audit-summary",
    "/models-list",
    "/model-show",
    "/skills-search",
    "/skills-show",
    "/skills-list",
    "/skills-lock-diff",
    "/skills-prune",
    "/skills-trust-list",
    "/skills-trust-add",
    "/skills-trust-revoke",
    "/skills-trust-rotate",
    "/skills-lock-write",
    "/skills-sync",
    "/skills-verify",
    "/ops-verify",
    "/ops-validate",
    "/ops-dev",
    "/branches",
    "/macro",
    "/auth",
    "/canvas",
    "/rbac",
    "/approvals",
    "/integration-auth",
    "/pair",
    "/unpair",
    "/profile",
    "/branch-alias",
    "/session-bookmark",
    "/branch",
    "/undo",
    "/redo",
    "/resume",
    "/session-repair",
    "/session-compact",
    "/training-status",
    "/training-trigger",
    "/self-modify-status",
    "/self-modify-review",
    "/self-modify-rollback",
    "/self-modify-policy",
    "/learn-status",
    "/learn-clear",
    "/learn-export",
    "/init",
    "/config-validate",
    "/config-show",
    "/quit",
    "/exit",
];

/// Public `fn` `render_help_overview` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn render_help_overview() -> String {
    tau_cli::render_help_overview(COMMAND_SPECS)
}

/// Public `fn` `render_command_help` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn render_command_help(topic: &str) -> Option<String> {
    tau_cli::render_command_help(topic, COMMAND_SPECS)
}

/// Public `fn` `unknown_help_topic_message` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn unknown_help_topic_message(topic: &str) -> String {
    tau_cli::unknown_help_topic_message(topic, COMMAND_NAMES)
}

/// Public `fn` `unknown_command_message` in `tau-ops`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn unknown_command_message(command: &str) -> String {
    tau_cli::unknown_command_message(command, COMMAND_NAMES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_command_catalog_contains_expected_entries() {
        assert!(COMMAND_NAMES.contains(&"/help"));
        assert!(COMMAND_NAMES.contains(&"/canvas"));
        assert!(COMMAND_NAMES.contains(&"/undo"));
        assert!(COMMAND_NAMES.contains(&"/redo"));
        assert!(COMMAND_NAMES.contains(&"/exit"));
        assert_eq!(COMMAND_SPECS[0].name, "/help");
    }

    #[test]
    fn functional_render_help_overview_includes_key_commands() {
        let rendered = render_help_overview();
        assert!(rendered.contains("/help"));
        assert!(rendered.contains("/session"));
        assert!(rendered.contains("/undo"));
        assert!(rendered.contains("/qa-loop"));
    }

    #[test]
    fn unit_training_command_specs_are_registered() {
        assert!(COMMAND_NAMES.contains(&"/training-status"));
        assert!(COMMAND_NAMES.contains(&"/training-trigger"));
        let training_status_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/training-status");
        assert!(
            training_status_spec.is_some(),
            "/training-status spec must exist in COMMAND_SPECS"
        );
        let training_trigger_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/training-trigger");
        assert!(
            training_trigger_spec.is_some(),
            "/training-trigger spec must exist in COMMAND_SPECS"
        );
    }

    #[test]
    fn regression_unknown_command_message_suggests_nearby_match() {
        let rendered = unknown_command_message("/sesion");
        assert!(rendered.contains("unknown command"));
        assert!(rendered.contains("/session"));
    }

    #[test]
    fn unit_learn_command_specs_are_registered() {
        assert!(
            COMMAND_NAMES.contains(&"/learn-status"),
            "/learn-status must be in COMMAND_NAMES"
        );
        assert!(
            COMMAND_NAMES.contains(&"/learn-clear"),
            "/learn-clear must be in COMMAND_NAMES"
        );
        assert!(
            COMMAND_NAMES.contains(&"/learn-export"),
            "/learn-export must be in COMMAND_NAMES"
        );

        let learn_status_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/learn-status");
        assert!(
            learn_status_spec.is_some(),
            "/learn-status spec must exist in COMMAND_SPECS"
        );

        let learn_clear_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/learn-clear");
        assert!(
            learn_clear_spec.is_some(),
            "/learn-clear spec must exist in COMMAND_SPECS"
        );

        let learn_export_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/learn-export");
        assert!(
            learn_export_spec.is_some(),
            "/learn-export spec must exist in COMMAND_SPECS"
        );
    }

    #[test]
    fn unit_config_command_specs_are_registered() {
        assert!(COMMAND_NAMES.contains(&"/init"));
        assert!(COMMAND_NAMES.contains(&"/config-validate"));
        assert!(COMMAND_NAMES.contains(&"/config-show"));
        let init_spec = COMMAND_SPECS.iter().find(|spec| spec.name == "/init");
        assert!(
            init_spec.is_some(),
            "/init spec must exist in COMMAND_SPECS"
        );
        let validate_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/config-validate");
        assert!(
            validate_spec.is_some(),
            "/config-validate spec must exist in COMMAND_SPECS"
        );
        let show_spec = COMMAND_SPECS
            .iter()
            .find(|spec| spec.name == "/config-show");
        assert!(
            show_spec.is_some(),
            "/config-show spec must exist in COMMAND_SPECS"
        );
    }

    #[test]
    fn unit_self_modify_commands_registered_in_catalog() {
        let expected = [
            "/self-modify-status",
            "/self-modify-review",
            "/self-modify-rollback",
            "/self-modify-policy",
        ];
        for name in &expected {
            assert!(
                COMMAND_NAMES.contains(name),
                "{name} must be in COMMAND_NAMES"
            );
            let spec = COMMAND_SPECS.iter().find(|s| s.name == *name);
            assert!(
                spec.is_some(),
                "{name} must have a CommandSpec in COMMAND_SPECS"
            );
        }
    }
}
