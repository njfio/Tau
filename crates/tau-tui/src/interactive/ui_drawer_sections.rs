use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use super::super::app::App;
use super::super::chat::MessageRole;
use super::super::status::AgentStateDisplay;
use super::super::tools::ToolStatus;
use super::shared::latest_running_tool;

pub(super) fn tool_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Tool activity"),
        detail_line_item(format!(
            "{} active / {} total",
            app.tools.active_count(),
            app.tools.total_count()
        )),
        detail_line_item("Open Memory, Cortex, or Sessions from the tabs above.".to_string()),
    ];
    let entries = app.tools.entries();
    if entries.is_empty() {
        items.push(detail_line_item("No tool executions yet.".to_string()));
        return items;
    }
    let start = entries.len().saturating_sub(4);
    items.extend(entries[start..].iter().map(|entry| {
        let status_style = match entry.status {
            ToolStatus::Running => Style::default().fg(Color::Yellow),
            ToolStatus::Success => Style::default().fg(Color::Green),
            ToolStatus::Failed => Style::default().fg(Color::Red),
            ToolStatus::Timeout => Style::default().fg(Color::Magenta),
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", entry.status.label()), status_style),
            Span::styled(entry.name.clone(), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(
                truncate(
                    &entry.detail,
                    max_width.saturating_sub(entry.name.len() + 8),
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
    }));
    items
}

pub(super) fn memory_items(app: &App) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Memory"),
        detail_line_item("Status: degraded".to_string()),
        detail_line_item("shared state unavailable".to_string()),
    ];
    if let Some(context) = latest_user_context(app) {
        items.push(metric_item("Recent user context", context));
    } else {
        items.push(detail_line_item("No stored memory yet.".to_string()));
    }
    items
}

pub(super) fn cortex_items(app: &App) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Observer posture"),
        metric_item("State", state_label(app.status.agent_state).to_string()),
        metric_item("Active tools", app.tools.active_count().to_string()),
        metric_item(
            "Pending approval",
            yes_no(app.approval_request.is_some()).to_string(),
        ),
    ];
    if let Some(state) = &app.last_operator_state {
        items.push(metric_item("Entity", state.entity.clone()));
        if let Some(phase) = &state.phase {
            items.push(metric_item("Phase", phase.clone()));
        }
        if let Some(reason) = &state.reason_code {
            items.push(metric_item("Reason", reason.clone()));
        }
    }
    if let Some(tool) = latest_running_tool(app) {
        items.push(metric_item("Current tool", tool.name.clone()));
    }
    items
}

pub(super) fn session_items(app: &App) -> Vec<ListItem<'static>> {
    vec![
        section_item("Sessions"),
        metric_item("Current session", app.config.session_key.clone()),
        metric_item("Messages", app.chat.len().to_string()),
        metric_item("Tokens", app.status.total_tokens.to_string()),
        metric_item(
            "Approvals pending",
            usize::from(app.approval_request.is_some()).to_string(),
        ),
        metric_item("Last prompt", last_prompt(app)),
        metric_item("Assistant msgs", assistant_message_count(app).to_string()),
    ]
}

pub(super) fn section_item(title: &str) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )))
}

pub(super) fn detail_line_item(text: String) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::Gray),
    )))
}

pub(super) fn metric_item(label: &str, value: String) -> ListItem<'static> {
    detail_line_item(format!("{label}: {value}"))
}

fn latest_user_context(app: &App) -> Option<String> {
    app.chat
        .latest_content_by_role(MessageRole::User)
        .map(|content| truncate(content, 72))
}

fn last_prompt(app: &App) -> String {
    app.last_submitted_input
        .clone()
        .or_else(|| {
            app.chat
                .latest_content_by_role(MessageRole::User)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "none".to_string())
}

fn assistant_message_count(app: &App) -> usize {
    app.chat.count_by_role(MessageRole::Assistant)
}

fn state_label(state: AgentStateDisplay) -> &'static str {
    match state {
        AgentStateDisplay::Idle => "idle",
        AgentStateDisplay::Thinking => "thinking",
        AgentStateDisplay::ToolExec => "tool",
        AgentStateDisplay::Streaming => "streaming",
        AgentStateDisplay::Error => "error",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    if max > 3 {
        return format!("{}...", &input[..max - 3]);
    }
    input[..max].to_string()
}
