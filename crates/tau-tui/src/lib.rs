//! Terminal UI primitives and rendering contracts for Tau interfaces.
//!
//! Contains reusable TUI components, view-model types, and rendering helpers
//! used by interactive terminal surfaces.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

/// Trait contract for `Component` behavior.
pub trait Component {
    fn render(&self, width: usize) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Public struct `Cursor` used across Tau components.
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `EditorBuffer` used across Tau components.
pub struct EditorBuffer {
    lines: Vec<String>,
    cursor: Cursor,
}

impl Default for EditorBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor::default(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut lines = text
            .split('\n')
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            cursor: Cursor::default(),
        }
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn insert_text(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.insert_newline();
            } else {
                self.insert_char(ch);
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let line = &mut self.lines[self.cursor.line];
        let byte_index = char_to_byte_index(line, self.cursor.column);
        line.insert(byte_index, ch);
        self.cursor.column += 1;
    }

    pub fn insert_newline(&mut self) {
        let current = &mut self.lines[self.cursor.line];
        let split_index = char_to_byte_index(current, self.cursor.column);
        let tail = current.split_off(split_index);
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.lines.insert(self.cursor.line, tail);
    }

    pub fn delete_backward(&mut self) {
        if self.cursor.column > 0 {
            let line = &mut self.lines[self.cursor.line];
            let start = char_to_byte_index(line, self.cursor.column - 1);
            let end = char_to_byte_index(line, self.cursor.column);
            line.replace_range(start..end, "");
            self.cursor.column -= 1;
            return;
        }

        if self.cursor.line == 0 {
            return;
        }

        let current = self.lines.remove(self.cursor.line);
        self.cursor.line -= 1;
        let previous = &mut self.lines[self.cursor.line];
        let previous_len = previous.chars().count();
        previous.push_str(&current);
        self.cursor.column = previous_len;
    }

    pub fn delete_forward(&mut self) {
        let line_len = self.lines[self.cursor.line].chars().count();
        if self.cursor.column < line_len {
            let line = &mut self.lines[self.cursor.line];
            let start = char_to_byte_index(line, self.cursor.column);
            let end = char_to_byte_index(line, self.cursor.column + 1);
            line.replace_range(start..end, "");
            return;
        }

        if self.cursor.line + 1 >= self.lines.len() {
            return;
        }

        let next = self.lines.remove(self.cursor.line + 1);
        self.lines[self.cursor.line].push_str(&next);
    }

    pub fn move_left(&mut self) {
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
            return;
        }

        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.column = self.lines[self.cursor.line].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines[self.cursor.line].chars().count();
        if self.cursor.column < line_len {
            self.cursor.column += 1;
            return;
        }

        if self.cursor.line + 1 < self.lines.len() {
            self.cursor.line += 1;
            self.cursor.column = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.line == 0 {
            return;
        }

        self.cursor.line -= 1;
        let line_len = self.lines[self.cursor.line].chars().count();
        self.cursor.column = self.cursor.column.min(line_len);
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 >= self.lines.len() {
            return;
        }

        self.cursor.line += 1;
        let line_len = self.lines[self.cursor.line].chars().count();
        self.cursor.column = self.cursor.column.min(line_len);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Public struct `Theme` used across Tau components.
pub struct Theme {
    pub name: String,
    pub palette: ThemePalette,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            palette: ThemePalette::default(),
        }
    }
}

impl Theme {
    pub fn from_json(source: &str) -> Result<Self, ThemeError> {
        let theme = serde_json::from_str::<Theme>(source)?;
        theme.validate()?;
        Ok(theme)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path).map_err(|source| ThemeError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_json(&raw)
    }

    pub fn paint(&self, role: ThemeRole, text: &str) -> String {
        let color = self.palette.color_code(role);
        format!("\x1b[{color}m{text}\x1b[0m")
    }

    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.name.trim().is_empty() {
            return Err(ThemeError::EmptyName);
        }

        self.palette.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Public struct `ThemePalette` used across Tau components.
pub struct ThemePalette {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub muted: String,
    pub error: String,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            primary: "36".to_string(),
            secondary: "37".to_string(),
            accent: "33".to_string(),
            muted: "90".to_string(),
            error: "31".to_string(),
        }
    }
}

impl ThemePalette {
    fn validate(&self) -> Result<(), ThemeError> {
        let pairs = [
            ("primary", self.primary.as_str()),
            ("secondary", self.secondary.as_str()),
            ("accent", self.accent.as_str()),
            ("muted", self.muted.as_str()),
            ("error", self.error.as_str()),
        ];

        for (field, code) in pairs {
            if !is_valid_ansi_color_code(code) {
                return Err(ThemeError::InvalidColorCode {
                    field,
                    code: code.to_string(),
                });
            }
        }

        Ok(())
    }

    fn color_code(&self, role: ThemeRole) -> &str {
        match role {
            ThemeRole::Primary => self.primary.as_str(),
            ThemeRole::Secondary => self.secondary.as_str(),
            ThemeRole::Accent => self.accent.as_str(),
            ThemeRole::Muted => self.muted.as_str(),
            ThemeRole::Error => self.error.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumerates supported `ThemeRole` values.
pub enum ThemeRole {
    Primary,
    Secondary,
    Accent,
    Muted,
    Error,
}

#[derive(Debug, Error)]
/// Enumerates supported `ThemeError` values.
pub enum ThemeError {
    #[error("failed to parse theme JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("failed to read theme file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("theme name must not be empty")]
    EmptyName,
    #[error("invalid ANSI color code '{code}' for field '{field}'")]
    InvalidColorCode { field: &'static str, code: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `Text` used across Tau components.
pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Component for Text {
    fn render(&self, width: usize) -> Vec<String> {
        wrap_text(&self.content, width)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `LumaImage` used across Tau components.
pub struct LumaImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl LumaImage {
    pub fn from_luma(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::EmptyDimensions);
        }

        let expected = width
            .checked_mul(height)
            .ok_or(ImageError::DimensionsTooLarge)?;
        if pixels.len() != expected {
            return Err(ImageError::InvalidPixelCount {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn render_fit(&self, max_width: usize) -> Vec<String> {
        if max_width == 0 {
            return vec![String::new()];
        }

        let target_width = self.width.min(max_width).max(1);
        let target_height = ((self.height * target_width) / self.width).max(1);
        let mut lines = Vec::with_capacity(target_height);
        for target_y in 0..target_height {
            let src_y = target_y * self.height / target_height;
            let mut line = String::with_capacity(target_width);
            for target_x in 0..target_width {
                let src_x = target_x * self.width / target_width;
                let value = self.pixels[src_y * self.width + src_x];
                line.push(luma_to_ascii(value));
            }
            lines.push(line);
        }
        lines
    }
}

impl Component for LumaImage {
    fn render(&self, width: usize) -> Vec<String> {
        self.render_fit(width)
    }
}

#[derive(Debug, Error)]
/// Enumerates supported `ImageError` values.
pub enum ImageError {
    #[error("image dimensions must be greater than zero")]
    EmptyDimensions,
    #[error("image dimensions overflowed while computing pixel count")]
    DimensionsTooLarge,
    #[error("invalid pixel count: expected {expected}, got {actual}")]
    InvalidPixelCount { expected: usize, actual: usize },
}

#[derive(Debug, Clone, Copy)]
/// Public struct `EditorView` used across Tau components.
pub struct EditorView<'a> {
    buffer: &'a EditorBuffer,
    viewport_top: usize,
    viewport_height: usize,
    show_line_numbers: bool,
    show_cursor: bool,
}

impl<'a> EditorView<'a> {
    pub fn new(buffer: &'a EditorBuffer) -> Self {
        Self {
            buffer,
            viewport_top: 0,
            viewport_height: buffer.lines().len().max(1),
            show_line_numbers: true,
            show_cursor: true,
        }
    }

    pub fn with_viewport(mut self, top: usize, height: usize) -> Self {
        self.viewport_top = top;
        self.viewport_height = height.max(1);
        self
    }

    pub fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.show_line_numbers = enabled;
        self
    }

    pub fn with_cursor(mut self, enabled: bool) -> Self {
        self.show_cursor = enabled;
        self
    }
}

impl Component for EditorView<'_> {
    fn render(&self, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![String::new()];
        }

        let lines = self.buffer.lines();
        if lines.is_empty() {
            return vec![String::new()];
        }

        let total_line_digits = lines.len().to_string().len();
        let number_prefix_width = if self.show_line_numbers {
            total_line_digits + 2
        } else {
            0
        };
        let text_width = width.saturating_sub(number_prefix_width).max(1);
        let cursor = self.buffer.cursor();

        let mut rendered = Vec::new();
        let end = (self.viewport_top + self.viewport_height).min(lines.len());
        for (line_index, line) in lines.iter().enumerate().take(end).skip(self.viewport_top) {
            let mut text = line.clone();
            if self.show_cursor && cursor.line == line_index {
                text = insert_marker_at_char(&text, cursor.column, '|');
            }

            text = truncate_to_char_width(&text, text_width);
            if self.show_line_numbers {
                let prefix = format!("{:>width$} ", line_index + 1, width = total_line_digits);
                let mut line = prefix;
                line.push(' ');
                line.push_str(&text);
                rendered.push(truncate_to_char_width(&line, width));
            } else {
                rendered.push(text);
            }
        }

        if rendered.is_empty() {
            rendered.push(String::new());
        }
        rendered
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Enumerates supported `RenderOp` values.
pub enum RenderOp {
    Update { line: usize, content: String },
    ClearFrom { line: usize },
}

impl fmt::Display for RenderOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderOp::Update { line, content } => write!(f, "update({line}):{content}"),
            RenderOp::ClearFrom { line } => write!(f, "clear_from({line})"),
        }
    }
}

#[derive(Default, Debug, Clone)]
/// Public struct `DiffRenderer` used across Tau components.
pub struct DiffRenderer {
    previous: Vec<String>,
}

impl DiffRenderer {
    pub fn new() -> Self {
        Self {
            previous: Vec::new(),
        }
    }

    pub fn diff(&mut self, next: Vec<String>) -> Vec<RenderOp> {
        let mut operations = Vec::new();
        let max_len = self.previous.len().max(next.len());

        for index in 0..max_len {
            match (self.previous.get(index), next.get(index)) {
                (Some(old), Some(new)) if old != new => operations.push(RenderOp::Update {
                    line: index,
                    content: new.clone(),
                }),
                (None, Some(new)) => operations.push(RenderOp::Update {
                    line: index,
                    content: new.clone(),
                }),
                _ => {}
            }
        }

        if next.len() < self.previous.len() {
            operations.push(RenderOp::ClearFrom { line: next.len() });
        }

        self.previous = next;
        operations
    }

    pub fn snapshot(&self) -> &[String] {
        &self.previous
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Provider auth row shown in the operator shell auth panel.
pub struct OperatorShellAuthRow {
    pub provider: String,
    pub mode: String,
    pub state: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Operator-shell view model for deterministic status rendering.
pub struct OperatorShellFrame {
    pub environment: String,
    pub profile: String,
    pub heartbeat: String,
    pub health_reason: String,
    pub auth_mode: String,
    pub auth_required: bool,
    pub rollout_total: usize,
    pub rollout_succeeded: usize,
    pub rollout_failed: usize,
    pub queue_depth: usize,
    pub failure_streak: usize,
    pub primary_alert_code: String,
    pub primary_alert_severity: String,
    pub primary_alert_message: String,
    pub auth_rows: Vec<OperatorShellAuthRow>,
    pub alerts: Vec<String>,
    pub actions: Vec<String>,
}

impl OperatorShellFrame {
    /// Returns a deterministic fixture payload for local shell rendering.
    pub fn deterministic_fixture(profile: String) -> Self {
        Self {
            environment: "local".to_string(),
            profile,
            heartbeat: "healthy".to_string(),
            health_reason: "dashboard runtime health is nominal".to_string(),
            auth_mode: "token".to_string(),
            auth_required: true,
            rollout_total: 12,
            rollout_succeeded: 11,
            rollout_failed: 1,
            queue_depth: 0,
            failure_streak: 1,
            primary_alert_code: "rollout_failure_recent".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "1 rollout failed in last cycle".to_string(),
            auth_rows: vec![
                OperatorShellAuthRow {
                    provider: "openai".to_string(),
                    mode: "oauth_token".to_string(),
                    state: "ready".to_string(),
                    source: "credential_store".to_string(),
                },
                OperatorShellAuthRow {
                    provider: "anthropic".to_string(),
                    mode: "api_key".to_string(),
                    state: "ready".to_string(),
                    source: "env".to_string(),
                },
                OperatorShellAuthRow {
                    provider: "google".to_string(),
                    mode: "adc".to_string(),
                    state: "ready".to_string(),
                    source: "gcloud".to_string(),
                },
            ],
            alerts: vec!["1 rollout failed in last cycle".to_string()],
            actions: vec![
                "press r: rerun failed rollout".to_string(),
                "press a: open auth matrix".to_string(),
                "press t: open training summary".to_string(),
            ],
        }
    }

    /// Builds a live operator-shell frame from persisted dashboard/training artifacts.
    pub fn from_dashboard_state_dir(profile: String, dashboard_state_dir: &Path) -> Self {
        let dashboard_root = dashboard_state_dir.to_path_buf();
        let training_root = resolve_training_root(dashboard_state_dir);
        let state_path = dashboard_root.join("state.json");
        let events_log_path = dashboard_root.join("runtime-events.jsonl");
        let actions_log_path = dashboard_root.join("actions-audit.jsonl");
        let control_state_path = dashboard_root.join("control-state.json");
        let auth_status_path = dashboard_root.join("auth-status.json");
        let training_status_path = training_root.join("status.json");

        let mut artifact_diagnostics = Vec::new();
        let runtime_state = read_json_file::<LiveDashboardRuntimeStateFile>(
            &state_path,
            &mut artifact_diagnostics,
            true,
        );
        let control_state = read_json_file::<LiveDashboardControlStateFile>(
            &control_state_path,
            &mut artifact_diagnostics,
            true,
        );
        let auth_status = read_json_file::<LiveDashboardAuthStatusFile>(
            &auth_status_path,
            &mut artifact_diagnostics,
            true,
        );
        let training_status = read_json_file::<LiveDashboardTrainingStatusFile>(
            &training_status_path,
            &mut artifact_diagnostics,
            false,
        );
        let last_cycle = read_last_jsonl_record::<LiveDashboardCycleReportLine>(
            &events_log_path,
            &mut artifact_diagnostics,
            false,
        );
        let last_action = read_last_jsonl_record::<LiveDashboardActionAuditRecord>(
            &actions_log_path,
            &mut artifact_diagnostics,
            false,
        )
        .or_else(|| {
            control_state
                .as_ref()
                .and_then(|state| state.last_action.clone())
        });

        let queue_depth = runtime_state
            .as_ref()
            .map_or(0, |state| state.health.queue_depth);
        let failure_streak = runtime_state
            .as_ref()
            .map_or(0, |state| state.health.failure_streak);
        let processed_case_count = runtime_state
            .as_ref()
            .map_or(0, |state| state.processed_case_keys.len());

        let heartbeat = if failure_streak >= 3 {
            "failing"
        } else if failure_streak > 0 || queue_depth > 0 {
            "degraded"
        } else {
            "healthy"
        };

        let mut health_reason = last_cycle
            .as_ref()
            .map(|cycle| cycle.health_reason.trim())
            .filter(|reason| !reason.is_empty())
            .map(str::to_string)
            .unwrap_or_default();
        if health_reason.is_empty() {
            health_reason = if runtime_state.is_some() {
                "dashboard runtime state loaded".to_string()
            } else {
                format!(
                    "dashboard runtime state unavailable at {}",
                    state_path.display()
                )
            };
        }

        let control_mode = normalize_dashboard_control_mode(
            control_state
                .as_ref()
                .map_or("running", |state| state.mode.as_str()),
        );
        let control_paused = control_mode == "paused";

        let rollout_total = training_status
            .as_ref()
            .map_or(0, |status| status.total_rollouts);
        let rollout_succeeded = training_status
            .as_ref()
            .map_or(0, |status| status.succeeded);
        let rollout_failed = training_status
            .as_ref()
            .map_or(0, |status| status.failed.saturating_add(status.cancelled));
        let training_run_state = training_status
            .as_ref()
            .map_or("unknown".to_string(), |status| {
                normalize_non_empty(status.run_state.as_str(), "unknown")
            });

        let mut alerts = Vec::new();
        if control_paused {
            alerts.push("operator pause action is active".to_string());
        }
        if queue_depth > 0 {
            alerts.push(format!(
                "runtime queue backlog observed (queue_depth={queue_depth})"
            ));
        }
        if failure_streak > 0 {
            alerts.push(format!(
                "runtime failure streak observed (failure_streak={failure_streak})"
            ));
        }
        if rollout_failed > 0 {
            alerts.push(format!(
                "training failures observed (failed_or_cancelled={rollout_failed})"
            ));
        }
        if let Some(status) = &training_status {
            alerts.extend(status.live_learning_alerts.iter().filter_map(|alert| {
                let code = alert.code.trim();
                let severity = normalize_non_empty(alert.severity.as_str(), "info");
                let message = alert.message.trim();
                if code.is_empty() || message.is_empty() {
                    return None;
                }
                Some(format!("{severity}:{code}: {message}"))
            }));
        }
        if alerts.is_empty() {
            alerts.push("dashboard runtime health is nominal".to_string());
        }
        if !artifact_diagnostics.is_empty() {
            alerts.push(format!(
                "artifact diagnostics active ({})",
                artifact_diagnostics.len()
            ));
            for diagnostic in artifact_diagnostics.iter().take(3) {
                alerts.push(format!("artifact: {diagnostic}"));
            }
        }

        let (primary_alert_code, primary_alert_severity, primary_alert_message) = if control_paused
        {
            (
                "operator_pause_active".to_string(),
                "info".to_string(),
                "operator pause action is active".to_string(),
            )
        } else if failure_streak > 0 {
            (
                "runtime_failure_streak".to_string(),
                if failure_streak >= 3 {
                    "critical".to_string()
                } else {
                    "warning".to_string()
                },
                format!("runtime failure streak observed ({failure_streak})"),
            )
        } else if queue_depth > 0 {
            (
                "runtime_queue_backlog".to_string(),
                "warning".to_string(),
                format!("runtime queue backlog observed (queue_depth={queue_depth})"),
            )
        } else if rollout_failed > 0 {
            (
                "training_failures".to_string(),
                "warning".to_string(),
                format!("training failures observed (failed_or_cancelled={rollout_failed})"),
            )
        } else {
            (
                "dashboard_healthy".to_string(),
                "info".to_string(),
                "dashboard runtime health is nominal".to_string(),
            )
        };

        let auth_mode = auth_status
            .as_ref()
            .map_or("unknown".to_string(), |status| {
                normalize_non_empty(status.mode.as_str(), "unknown")
            });
        let auth_required = auth_status.as_ref().is_some_and(|status| status.required);
        let auth_rows = auth_status.as_ref().map_or_else(Vec::new, |status| {
            status
                .providers
                .iter()
                .map(|provider| OperatorShellAuthRow {
                    provider: normalize_non_empty(provider.provider.as_str(), "unknown"),
                    mode: normalize_non_empty(provider.mode.as_str(), "unknown"),
                    state: normalize_non_empty(provider.state.as_str(), "unknown"),
                    source: normalize_non_empty(provider.source.as_str(), "unknown"),
                })
                .collect::<Vec<_>>()
        });
        let auth_rows = if auth_rows.is_empty() {
            vec![OperatorShellAuthRow {
                provider: "n/a".to_string(),
                mode: "unknown".to_string(),
                state: "unknown".to_string(),
                source: if auth_status_path.exists() {
                    "auth-status.json has no providers".to_string()
                } else {
                    "auth-status.json missing".to_string()
                },
            }]
        } else {
            auth_rows
        };

        let mut actions = vec![
            format!("control.mode: {control_mode}"),
            if control_paused {
                "next action: resume".to_string()
            } else {
                "next action: pause or refresh".to_string()
            },
            format!("training.run_state: {training_run_state}"),
            format!("processed cases: {processed_case_count}"),
        ];
        if let Some(action) = last_action {
            let action_name = normalize_non_empty(action.action.as_str(), "unknown");
            let action_actor = normalize_non_empty(action.actor.as_str(), "unknown");
            actions.push(format!(
                "last action: {action_name} by {action_actor} @ {}",
                action.timestamp_unix_ms
            ));
        } else {
            actions.push("last action: none".to_string());
        }
        if let Some(cycle) = &last_cycle {
            if !cycle.reason_codes.is_empty() {
                actions.push(format!(
                    "last reason codes: {}",
                    cycle.reason_codes.join(",")
                ));
            }
        }
        if !artifact_diagnostics.is_empty() {
            let diagnostic_counts = summarize_artifact_diagnostics(&artifact_diagnostics);
            actions.push(format!(
                "artifact diagnostics count: {}",
                artifact_diagnostics.len()
            ));
            actions.push(format!(
                "artifact diagnostics summary: parse_failed={} missing={} jsonl_malformed={} read_failed={}",
                diagnostic_counts.parse_failed,
                diagnostic_counts.missing,
                diagnostic_counts.jsonl_malformed,
                diagnostic_counts.read_failed
            ));
            for diagnostic in artifact_diagnostics.iter().take(4) {
                actions.push(format!("diag: {diagnostic}"));
            }
        }

        Self {
            environment: "live".to_string(),
            profile,
            heartbeat: heartbeat.to_string(),
            health_reason,
            auth_mode,
            auth_required,
            rollout_total,
            rollout_succeeded,
            rollout_failed,
            queue_depth,
            failure_streak,
            primary_alert_code,
            primary_alert_severity,
            primary_alert_message,
            auth_rows,
            alerts,
            actions,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardRuntimeStateFile {
    #[serde(default)]
    processed_case_keys: Vec<String>,
    #[serde(default)]
    health: LiveDashboardHealth,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardHealth {
    #[serde(default)]
    queue_depth: usize,
    #[serde(default)]
    failure_streak: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardControlStateFile {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    last_action: Option<LiveDashboardActionAuditRecord>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardTrainingStatusFile {
    #[serde(default)]
    run_state: String,
    #[serde(default)]
    total_rollouts: usize,
    #[serde(default)]
    succeeded: usize,
    #[serde(default)]
    failed: usize,
    #[serde(default)]
    cancelled: usize,
    #[serde(default)]
    live_learning_alerts: Vec<LiveDashboardTrainingAlert>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardTrainingAlert {
    #[serde(default)]
    code: String,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardCycleReportLine {
    #[serde(default)]
    health_reason: String,
    #[serde(default)]
    reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardActionAuditRecord {
    #[serde(default)]
    action: String,
    #[serde(default)]
    actor: String,
    #[serde(default)]
    timestamp_unix_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardAuthStatusFile {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    providers: Vec<LiveDashboardAuthProviderRow>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LiveDashboardAuthProviderRow {
    #[serde(default)]
    provider: String,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    source: String,
}

/// Renders a deterministic multi-panel operator shell frame.
pub fn render_operator_shell_frame(frame: &OperatorShellFrame, width: usize) -> Vec<String> {
    let panel_width = width.max(40);
    let mut output = Vec::new();
    output.push(format!(
        "operator profile={} environment={} heartbeat={}",
        frame.profile, frame.environment, frame.heartbeat
    ));
    output.push(String::new());

    let status_lines = vec![
        format!("environment : {}", frame.environment),
        format!("profile     : {}", frame.profile),
        format!("heartbeat   : {}", frame.heartbeat),
        format!("health.reason : {}", frame.health_reason),
    ];
    output.extend(render_shell_panel("STATUS", &status_lines, panel_width));
    output.push(String::new());

    let mut auth_lines = vec![
        format!("auth.mode     : {}", frame.auth_mode),
        format!(
            "auth.required : {}",
            if frame.auth_required { "true" } else { "false" }
        ),
        "provider      mode          state      source".to_string(),
    ];
    if frame.auth_rows.is_empty() {
        auth_lines.push("none".to_string());
    } else {
        auth_lines.extend(frame.auth_rows.iter().map(|row| {
            format!(
                "{:<12}  {:<12}  {:<9}  {}",
                row.provider, row.mode, row.state, row.source
            )
        }));
    }
    output.extend(render_shell_panel("AUTH", &auth_lines, panel_width));
    output.push(String::new());

    let training_lines = vec![
        format!("rollouts.total     : {}", frame.rollout_total),
        format!("rollouts.succeeded : {}", frame.rollout_succeeded),
        format!("rollouts.failed    : {}", frame.rollout_failed),
        format!(
            "rollouts.pass_rate : {:.2}%",
            compute_pass_rate(frame.rollout_total, frame.rollout_succeeded)
        ),
        format!("queue.depth        : {}", frame.queue_depth),
        format!("failure.streak     : {}", frame.failure_streak),
    ];
    output.extend(render_shell_panel("TRAINING", &training_lines, panel_width));
    output.push(String::new());

    let mut alert_lines = vec![
        format!("primary_alert.code     : {}", frame.primary_alert_code),
        format!("primary_alert.severity : {}", frame.primary_alert_severity),
        format!("primary_alert.message  : {}", frame.primary_alert_message),
    ];
    if frame.alerts.is_empty() {
        alert_lines.push("none".to_string());
    } else {
        alert_lines.extend(frame.alerts.clone());
    }
    output.extend(render_shell_panel("ALERTS", &alert_lines, panel_width));
    output.push(String::new());

    let action_lines = if frame.actions.is_empty() {
        vec!["none".to_string()]
    } else {
        frame.actions.clone()
    };
    output.extend(render_shell_panel("ACTIONS", &action_lines, panel_width));
    output
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let required = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };

            if required <= width {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
                continue;
            }

            if !current.is_empty() {
                lines.push(current);
                current = String::new();
            }

            if word.len() > width {
                let mut start = 0;
                let bytes = word.as_bytes();
                while start < bytes.len() {
                    let end = (start + width).min(bytes.len());
                    let segment = &word[start..end];
                    lines.push(segment.to_string());
                    start = end;
                }
            } else {
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

pub fn apply_overlay(base: &[String], overlay: &[String], top: usize, left: usize) -> Vec<String> {
    let mut output = base.to_vec();

    for (row, overlay_line) in overlay.iter().enumerate() {
        let line_index = top + row;
        while output.len() <= line_index {
            output.push(String::new());
        }

        write_at(&mut output[line_index], left, overlay_line);
    }

    output
}

fn luma_to_ascii(value: u8) -> char {
    const SCALE: &[u8] = b" .:-=+*#%@";
    let index = (usize::from(value) * (SCALE.len() - 1)) / 255;
    SCALE[index] as char
}

fn truncate_to_char_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    text.chars().take(width).collect()
}

fn insert_marker_at_char(text: &str, column: usize, marker: char) -> String {
    let mut output = String::new();
    let mut inserted = false;
    for (index, ch) in text.chars().enumerate() {
        if index == column {
            output.push(marker);
            inserted = true;
        }
        output.push(ch);
    }
    if !inserted {
        while output.chars().count() < column {
            output.push(' ');
        }
        output.push(marker);
    }
    output
}

fn char_to_byte_index(line: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }

    line.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or_else(|| line.len())
}

fn is_valid_ansi_color_code(code: &str) -> bool {
    if code.is_empty() {
        return false;
    }

    code.split(';')
        .all(|segment| !segment.is_empty() && segment.bytes().all(|byte| byte.is_ascii_digit()))
}

fn resolve_training_root(dashboard_state_dir: &Path) -> PathBuf {
    let tau_root = dashboard_state_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dashboard_state_dir.to_path_buf());
    tau_root.join("training")
}

fn read_json_file<T: DeserializeOwned>(
    path: &Path,
    diagnostics: &mut Vec<String>,
    report_missing: bool,
) -> Option<T> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                if report_missing {
                    diagnostics.push(format!("missing:{}", file_display_name(path)));
                }
            } else {
                diagnostics.push(format!(
                    "read_failed:{}:{}",
                    file_display_name(path),
                    read_error_kind_label(error.kind())
                ));
            }
            return None;
        }
    };

    match serde_json::from_str::<T>(&raw) {
        Ok(parsed) => Some(parsed),
        Err(_) => {
            diagnostics.push(format!("parse_failed:{}", file_display_name(path)));
            None
        }
    }
}

fn read_last_jsonl_record<T: DeserializeOwned>(
    path: &Path,
    diagnostics: &mut Vec<String>,
    report_missing: bool,
) -> Option<T> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                if report_missing {
                    diagnostics.push(format!("missing:{}", file_display_name(path)));
                }
            } else {
                diagnostics.push(format!(
                    "read_failed:{}:{}",
                    file_display_name(path),
                    read_error_kind_label(error.kind())
                ));
            }
            return None;
        }
    };

    let mut saw_non_empty_line = false;
    let mut saw_malformed_line = false;
    for line in raw.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        saw_non_empty_line = true;
        match serde_json::from_str::<T>(trimmed) {
            Ok(record) => return Some(record),
            Err(_) => saw_malformed_line = true,
        }
    }

    if saw_non_empty_line && saw_malformed_line {
        diagnostics.push(format!("jsonl_malformed:{}", file_display_name(path)));
    }
    None
}

fn file_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

fn read_error_kind_label(kind: std::io::ErrorKind) -> &'static str {
    match kind {
        std::io::ErrorKind::PermissionDenied => "permission_denied",
        std::io::ErrorKind::TimedOut => "timed_out",
        std::io::ErrorKind::Interrupted => "interrupted",
        std::io::ErrorKind::InvalidData => "invalid_data",
        std::io::ErrorKind::InvalidInput => "invalid_input",
        std::io::ErrorKind::AlreadyExists => "already_exists",
        std::io::ErrorKind::WouldBlock => "would_block",
        std::io::ErrorKind::UnexpectedEof => "unexpected_eof",
        std::io::ErrorKind::WriteZero => "write_zero",
        std::io::ErrorKind::Unsupported => "unsupported",
        std::io::ErrorKind::OutOfMemory => "out_of_memory",
        std::io::ErrorKind::BrokenPipe => "broken_pipe",
        std::io::ErrorKind::NotConnected => "not_connected",
        std::io::ErrorKind::AddrInUse => "addr_in_use",
        std::io::ErrorKind::AddrNotAvailable => "addr_not_available",
        std::io::ErrorKind::NetworkDown => "network_down",
        std::io::ErrorKind::NetworkUnreachable => "network_unreachable",
        std::io::ErrorKind::ConnectionAborted => "connection_aborted",
        std::io::ErrorKind::ConnectionRefused => "connection_refused",
        std::io::ErrorKind::ConnectionReset => "connection_reset",
        std::io::ErrorKind::HostUnreachable => "host_unreachable",
        std::io::ErrorKind::NotFound => "not_found",
        _ => "other",
    }
}

#[derive(Debug, Default)]
struct ArtifactDiagnosticCounts {
    parse_failed: usize,
    missing: usize,
    jsonl_malformed: usize,
    read_failed: usize,
}

fn summarize_artifact_diagnostics(diagnostics: &[String]) -> ArtifactDiagnosticCounts {
    let mut counts = ArtifactDiagnosticCounts::default();
    for diagnostic in diagnostics {
        if diagnostic.starts_with("parse_failed:") {
            counts.parse_failed += 1;
            continue;
        }
        if diagnostic.starts_with("missing:") {
            counts.missing += 1;
            continue;
        }
        if diagnostic.starts_with("jsonl_malformed:") {
            counts.jsonl_malformed += 1;
            continue;
        }
        if diagnostic.starts_with("read_failed:") {
            counts.read_failed += 1;
        }
    }
    counts
}

fn normalize_non_empty(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_dashboard_control_mode(raw: &str) -> String {
    if raw.trim().eq_ignore_ascii_case("paused") {
        "paused".to_string()
    } else {
        "running".to_string()
    }
}

fn compute_pass_rate(total: usize, succeeded: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (succeeded as f64 / total as f64) * 100.0
}

fn render_shell_panel(title: &str, lines: &[String], width: usize) -> Vec<String> {
    let inner_width = width.saturating_sub(4).max(16);
    let mut rendered = Vec::with_capacity(lines.len() + 3);
    rendered.push(format!("+{}+", "-".repeat(inner_width + 2)));
    rendered.push(format!("| {:<inner_width$} |", title));
    rendered.push(format!("+{}+", "-".repeat(inner_width + 2)));
    for line in lines {
        rendered.push(format!(
            "| {:<inner_width$} |",
            truncate_to_char_width(line, inner_width)
        ));
    }
    rendered.push(format!("+{}+", "-".repeat(inner_width + 2)));
    rendered
}

fn write_at(line: &mut String, left: usize, overlay: &str) {
    let mut chars = line.chars().collect::<Vec<_>>();
    while chars.len() < left {
        chars.push(' ');
    }

    for (index, ch) in overlay.chars().enumerate() {
        let position = left + index;
        if position < chars.len() {
            chars[position] = ch;
        } else {
            chars.push(ch);
        }
    }

    *line = chars.into_iter().collect();
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        apply_overlay, wrap_text, Cursor, DiffRenderer, EditorBuffer, EditorView, ImageError,
        LumaImage, OperatorShellAuthRow, OperatorShellFrame, RenderOp, Text, Theme, ThemeError,
        ThemeRole,
    };
    use crate::Component;

    #[test]
    fn wraps_text_to_width() {
        let lines = wrap_text("one two three", 7);
        assert_eq!(lines, vec!["one two", "three"]);
    }

    #[test]
    fn wraps_long_word() {
        let lines = wrap_text("abcdefghij", 4);
        assert_eq!(lines, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn text_component_renders_with_wrap() {
        let component = Text::new("hello world");
        assert_eq!(component.render(5), vec!["hello", "world"]);
    }

    #[test]
    fn unit_luma_image_rejects_invalid_pixel_count() {
        let error = LumaImage::from_luma(2, 2, vec![0, 1, 2]).expect_err("invalid size");
        assert!(matches!(error, ImageError::InvalidPixelCount { .. }));
    }

    #[test]
    fn functional_luma_image_renders_gradient_to_ascii() {
        let image =
            LumaImage::from_luma(4, 1, vec![0, 64, 192, 255]).expect("image should construct");
        assert_eq!(image.render(8), vec![" :*@".to_string()]);
    }

    #[test]
    fn regression_luma_image_render_handles_zero_width() {
        let image = LumaImage::from_luma(1, 1, vec![128]).expect("image");
        assert_eq!(image.render(0), vec![String::new()]);
    }

    #[test]
    fn renderer_outputs_only_changed_lines() {
        let mut renderer = DiffRenderer::new();
        let first = renderer.diff(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            first,
            vec![
                RenderOp::Update {
                    line: 0,
                    content: "a".to_string()
                },
                RenderOp::Update {
                    line: 1,
                    content: "b".to_string()
                }
            ]
        );

        let second = renderer.diff(vec!["a".to_string(), "c".to_string()]);
        assert_eq!(
            second,
            vec![RenderOp::Update {
                line: 1,
                content: "c".to_string()
            }]
        );

        let third = renderer.diff(vec!["a".to_string()]);
        assert_eq!(third, vec![RenderOp::ClearFrom { line: 1 }]);
    }

    #[test]
    fn unit_theme_from_json_parses_and_paints_text() {
        let theme = Theme::from_json(
            r#"{
                "name":"ocean",
                "palette":{
                    "primary":"36",
                    "secondary":"37",
                    "accent":"33",
                    "muted":"90",
                    "error":"31"
                }
            }"#,
        )
        .expect("theme should parse");

        let painted = theme.paint(ThemeRole::Primary, "hello");
        assert_eq!(painted, "\u{1b}[36mhello\u{1b}[0m");
    }

    #[test]
    fn functional_theme_from_path_loads_valid_file() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("theme.json");
        std::fs::write(
            &path,
            r#"{
                "name":"forest",
                "palette":{
                    "primary":"32",
                    "secondary":"37",
                    "accent":"33",
                    "muted":"90",
                    "error":"31"
                }
            }"#,
        )
        .expect("write theme file");

        let theme = Theme::from_path(&path).expect("theme should load");
        assert_eq!(theme.name, "forest");
    }

    #[test]
    fn regression_theme_rejects_invalid_color_code() {
        let error = Theme::from_json(
            r#"{
                "name":"bad",
                "palette":{
                    "primary":"36;foo",
                    "secondary":"37",
                    "accent":"33",
                    "muted":"90",
                    "error":"31"
                }
            }"#,
        )
        .expect_err("invalid color must fail");

        assert!(matches!(error, ThemeError::InvalidColorCode { .. }));
    }

    #[test]
    fn integration_default_theme_is_valid() {
        let theme = Theme::default();
        theme.validate().expect("default theme should be valid");
        assert_eq!(theme.paint(ThemeRole::Error, "x"), "\u{1b}[31mx\u{1b}[0m");
    }

    #[test]
    fn unit_apply_overlay_replaces_existing_text() {
        let base = vec!["hello world".to_string()];
        let output = apply_overlay(&base, &["rust".to_string()], 0, 6);
        assert_eq!(output, vec!["hello rustd".to_string()]);
    }

    #[test]
    fn functional_apply_overlay_extends_canvas_when_needed() {
        let base = vec!["abc".to_string()];
        let output = apply_overlay(&base, &["xyz".to_string(), "123".to_string()], 1, 2);
        assert_eq!(
            output,
            vec!["abc".to_string(), "  xyz".to_string(), "  123".to_string()]
        );
    }

    #[test]
    fn regression_apply_overlay_handles_unicode_content() {
        let base = vec!["status".to_string()];
        let output = apply_overlay(&base, &["🙂ok".to_string()], 0, 7);
        assert_eq!(output, vec!["status 🙂ok".to_string()]);
    }

    #[test]
    fn integration_renderer_diff_with_overlay_updates_only_changed_lines() {
        let mut renderer = DiffRenderer::new();
        let base = vec!["alpha".to_string(), "beta".to_string()];
        let first = apply_overlay(&base, &["ONE".to_string()], 1, 0);
        let second = apply_overlay(&base, &["TWO".to_string()], 1, 0);

        let initial = renderer.diff(first);
        assert_eq!(initial.len(), 2);
        let delta = renderer.diff(second);
        assert_eq!(
            delta,
            vec![RenderOp::Update {
                line: 1,
                content: "TWOa".to_string(),
            }]
        );
    }

    #[test]
    fn unit_editor_buffer_insert_and_delete_single_line() {
        let mut editor = EditorBuffer::new();
        editor.insert_text("rust");
        assert_eq!(editor.to_text(), "rust");
        assert_eq!(editor.cursor(), Cursor { line: 0, column: 4 });

        editor.delete_backward();
        assert_eq!(editor.to_text(), "rus");
        assert_eq!(editor.cursor(), Cursor { line: 0, column: 3 });
    }

    #[test]
    fn functional_editor_buffer_multiline_editing_and_navigation() {
        let mut editor = EditorBuffer::from_text("one\ntwo");
        editor.move_down();
        editor.move_right();
        editor.move_right();
        editor.insert_newline();
        editor.insert_text("x");

        assert_eq!(editor.lines().len(), 3);
        assert_eq!(editor.to_text(), "one\ntw\nxo");
        assert_eq!(editor.cursor(), Cursor { line: 2, column: 1 });
    }

    #[test]
    fn unit_editor_view_renders_line_numbers_and_cursor() {
        let mut editor = EditorBuffer::from_text("alpha\nbeta");
        editor.move_right();
        editor.move_right();
        let view = EditorView::new(&editor).with_viewport(0, 2);

        assert_eq!(view.render(20), vec!["1  al|pha", "2  beta"]);
    }

    #[test]
    fn functional_editor_view_hides_line_numbers_when_disabled() {
        let editor = EditorBuffer::from_text("a\nb\nc");
        let view = EditorView::new(&editor)
            .with_viewport(1, 2)
            .with_line_numbers(false)
            .with_cursor(false);

        assert_eq!(view.render(20), vec!["b", "c"]);
    }

    #[test]
    fn regression_editor_delete_backward_merges_lines_without_panic() {
        let mut editor = EditorBuffer::from_text("ab\ncd");
        editor.move_down();
        editor.delete_backward();
        assert_eq!(editor.to_text(), "abcd");
        assert_eq!(editor.cursor(), Cursor { line: 0, column: 2 });
    }

    #[test]
    fn integration_editor_buffer_diff_renderer_tracks_line_changes() {
        let mut renderer = DiffRenderer::new();
        let mut editor = EditorBuffer::from_text("a\nb");

        let initial = renderer.diff(editor.lines().to_vec());
        assert_eq!(initial.len(), 2);

        editor.move_down();
        editor.insert_text("!");
        let delta = renderer.diff(editor.lines().to_vec());
        assert_eq!(
            delta,
            vec![RenderOp::Update {
                line: 1,
                content: "!b".to_string(),
            }]
        );
    }

    #[test]
    fn integration_editor_view_overlay_and_diff_renderer_updates_cursor_line_only() {
        let mut renderer = DiffRenderer::new();
        let mut editor = EditorBuffer::from_text("hello\nworld");
        let base = vec!["status: ok".to_string()];

        let first_view = EditorView::new(&editor).with_viewport(0, 2).render(20);
        let first_frame = apply_overlay(&base, &first_view, 1, 0);
        let initial = renderer.diff(first_frame);
        assert_eq!(initial.len(), 3);

        editor.move_down();
        editor.move_right();
        let second_view = EditorView::new(&editor).with_viewport(0, 2).render(20);
        let second_frame = apply_overlay(&base, &second_view, 1, 0);
        let delta = renderer.diff(second_frame);

        assert_eq!(
            delta,
            vec![
                RenderOp::Update {
                    line: 1,
                    content: "1  hello".to_string(),
                },
                RenderOp::Update {
                    line: 2,
                    content: "2  w|orld".to_string(),
                },
            ]
        );
    }

    #[test]
    fn spec_c01_operator_shell_renderer_outputs_expected_panel_headers() {
        let frame = OperatorShellFrame::deterministic_fixture("local-dev".to_string());
        let rendered = super::render_operator_shell_frame(&frame, 72).join("\n");
        for panel in ["STATUS", "AUTH", "TRAINING", "ALERTS", "ACTIONS"] {
            assert!(rendered.contains(panel), "missing panel {panel}");
        }
    }

    #[test]
    fn functional_operator_shell_renderer_includes_auth_rows_and_training_metrics() {
        let frame = OperatorShellFrame {
            environment: "staging".to_string(),
            profile: "ops-west".to_string(),
            heartbeat: "degraded".to_string(),
            health_reason: "queue backlog observed".to_string(),
            auth_mode: "password-session".to_string(),
            auth_required: true,
            rollout_total: 4,
            rollout_succeeded: 3,
            rollout_failed: 1,
            queue_depth: 2,
            failure_streak: 1,
            primary_alert_code: "dashboard_queue_backlog".to_string(),
            primary_alert_severity: "warning".to_string(),
            primary_alert_message: "runtime backlog detected (queue_depth=2)".to_string(),
            auth_rows: vec![OperatorShellAuthRow {
                provider: "openrouter".to_string(),
                mode: "api_key".to_string(),
                state: "ready".to_string(),
                source: "env".to_string(),
            }],
            alerts: vec!["gateway latency elevated".to_string()],
            actions: vec!["press a: open auth".to_string()],
        };

        let rendered = super::render_operator_shell_frame(&frame, 68).join("\n");
        assert!(rendered.contains("openrouter"));
        assert!(rendered.contains("rollouts.total"));
        assert!(rendered.contains("75.00%"));
    }

    #[test]
    fn functional_live_shell_frame_loads_dashboard_and_training_artifacts() {
        let temp = tempdir().expect("tempdir");
        let tau_root = temp.path().join(".tau");
        let dashboard_root = tau_root.join("dashboard");
        let training_root = tau_root.join("training");
        std::fs::create_dir_all(&dashboard_root).expect("create dashboard dir");
        std::fs::create_dir_all(&training_root).expect("create training dir");

        std::fs::write(
            dashboard_root.join("state.json"),
            r#"{
  "processed_case_keys": ["case-1", "case-2"],
  "health": {
    "queue_depth": 2,
    "failure_streak": 1
  }
}
"#,
        )
        .expect("write state");
        std::fs::write(
            dashboard_root.join("runtime-events.jsonl"),
            r#"{"health_reason":"queue backlog observed","reason_codes":["queue_backpressure_applied","control_actions_applied"]}
"#,
        )
        .expect("write events");
        std::fs::write(
            dashboard_root.join("control-state.json"),
            r#"{
  "mode": "paused",
  "last_action": {
    "action": "pause",
    "actor": "ops-user",
    "timestamp_unix_ms": 90210
  }
}
"#,
        )
        .expect("write control state");
        std::fs::write(
            dashboard_root.join("auth-status.json"),
            r#"{
  "mode": "token",
  "required": true,
  "providers": [
    {"provider":"openai","mode":"oauth_token","state":"ready","source":"credential_store"}
  ]
}
"#,
        )
        .expect("write auth status");
        std::fs::write(
            training_root.join("status.json"),
            r#"{
  "run_state": "completed",
  "total_rollouts": 12,
  "succeeded": 11,
  "failed": 1,
  "cancelled": 0
}
"#,
        )
        .expect("write training status");

        let frame = OperatorShellFrame::from_dashboard_state_dir(
            "ops-live".to_string(),
            dashboard_root.as_path(),
        );
        assert_eq!(frame.environment, "live");
        assert_eq!(frame.profile, "ops-live");
        assert_eq!(frame.heartbeat, "degraded");
        assert_eq!(frame.health_reason, "queue backlog observed");
        assert_eq!(frame.auth_mode, "token");
        assert!(frame.auth_required);
        assert_eq!(frame.rollout_total, 12);
        assert_eq!(frame.rollout_succeeded, 11);
        assert_eq!(frame.rollout_failed, 1);
        assert_eq!(frame.queue_depth, 2);
        assert_eq!(frame.failure_streak, 1);
        assert_eq!(frame.primary_alert_code, "operator_pause_active");
        assert_eq!(frame.primary_alert_severity, "info");
        assert!(frame.primary_alert_message.contains("pause"));
        assert!(frame
            .auth_rows
            .iter()
            .any(|row| row.provider == "openai" && row.state == "ready"));
        assert!(frame
            .actions
            .iter()
            .any(|line| line.contains("last action: pause")));
        assert!(frame
            .actions
            .iter()
            .any(|line| line.contains("last reason codes")));
    }

    #[test]
    fn spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics() {
        let temp = tempdir().expect("tempdir");
        let tau_root = temp.path().join(".tau");
        let dashboard_root = tau_root.join("dashboard");
        std::fs::create_dir_all(&dashboard_root).expect("create dashboard dir");

        std::fs::write(dashboard_root.join("state.json"), "{").expect("write malformed state");
        std::fs::write(
            dashboard_root.join("control-state.json"),
            r#"{"mode":"running"}"#,
        )
        .expect("write control state");
        std::fs::write(dashboard_root.join("auth-status.json"), "{").expect("write malformed auth");

        let frame = OperatorShellFrame::from_dashboard_state_dir(
            "ops-malformed-json".to_string(),
            dashboard_root.as_path(),
        );

        assert!(frame
            .alerts
            .iter()
            .any(|line| line.contains("parse_failed:state.json")));
        assert!(frame
            .alerts
            .iter()
            .any(|line| line.contains("parse_failed:auth-status.json")));
        assert!(frame
            .actions
            .iter()
            .any(|line| line.contains("diag: parse_failed:state.json")));
        assert!(frame.actions.iter().any(|line| {
            line.contains("artifact diagnostics summary:")
                && line.contains("parse_failed=2")
                && line.contains("missing=0")
        }));
    }

    #[test]
    fn spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics() {
        let temp = tempdir().expect("tempdir");
        let tau_root = temp.path().join(".tau");
        let dashboard_root = tau_root.join("dashboard");
        std::fs::create_dir_all(&dashboard_root).expect("create dashboard dir");

        std::fs::write(
            dashboard_root.join("state.json"),
            r#"{"processed_case_keys":[],"health":{"queue_depth":0,"failure_streak":0}}"#,
        )
        .expect("write state");
        std::fs::write(
            dashboard_root.join("control-state.json"),
            r#"{"mode":"running"}"#,
        )
        .expect("write control state");
        std::fs::write(
            dashboard_root.join("auth-status.json"),
            r#"{"mode":"token","required":false,"providers":[]}"#,
        )
        .expect("write auth status");
        std::fs::write(
            dashboard_root.join("runtime-events.jsonl"),
            "{invalid-jsonl",
        )
        .expect("write malformed events");
        std::fs::write(dashboard_root.join("actions-audit.jsonl"), "{invalid-jsonl")
            .expect("write malformed actions");

        let frame = OperatorShellFrame::from_dashboard_state_dir(
            "ops-malformed-jsonl".to_string(),
            dashboard_root.as_path(),
        );

        assert!(frame
            .alerts
            .iter()
            .any(|line| line.contains("jsonl_malformed:runtime-events.jsonl")));
        assert!(frame
            .alerts
            .iter()
            .any(|line| line.contains("jsonl_malformed:actions-audit.jsonl")));
        assert!(frame
            .actions
            .iter()
            .any(|line| line.contains("diag: jsonl_malformed:runtime-events.jsonl")));
    }

    #[test]
    fn regression_live_shell_frame_handles_missing_artifacts_without_panicking() {
        let temp = tempdir().expect("tempdir");
        let dashboard_root = temp.path().join(".tau").join("dashboard");
        std::fs::create_dir_all(&dashboard_root).expect("create dashboard dir");

        let frame = OperatorShellFrame::from_dashboard_state_dir(
            "ops-empty".to_string(),
            dashboard_root.as_path(),
        );
        assert_eq!(frame.environment, "live");
        assert_eq!(frame.profile, "ops-empty");
        assert_eq!(frame.heartbeat, "healthy");
        assert_eq!(frame.auth_mode, "unknown");
        assert!(!frame.auth_required);
        assert_eq!(frame.rollout_total, 0);
        assert_eq!(frame.queue_depth, 0);
        assert_eq!(frame.failure_streak, 0);
        assert!(frame.health_reason.contains("state.json"));
        assert!(frame
            .auth_rows
            .iter()
            .any(|row| row.source.contains("missing")));
        assert!(frame
            .alerts
            .iter()
            .any(|line| line.contains("missing:state.json")));
        assert!(frame
            .actions
            .iter()
            .any(|line| line.contains("diag: missing:state.json")));
    }

    #[test]
    fn spec_c28_regression_operator_shell_auth_panel_requires_auth_mode_and_required_markers() {
        let frame = OperatorShellFrame::deterministic_fixture("ops-west".to_string());
        let rendered = super::render_operator_shell_frame(&frame, 72).join("\n");

        assert!(
            rendered.contains("auth.mode"),
            "missing auth.mode parity marker in:\n{rendered}"
        );
        assert!(
            rendered.contains("auth.required"),
            "missing auth.required parity marker in:\n{rendered}"
        );
    }

    #[test]
    fn spec_c28_regression_operator_shell_status_and_alert_panels_require_control_plane_markers() {
        let frame = OperatorShellFrame::deterministic_fixture("ops-west".to_string());
        let rendered = super::render_operator_shell_frame(&frame, 72).join("\n");

        for marker in [
            "health.reason",
            "queue.depth",
            "failure.streak",
            "primary_alert.code",
        ] {
            assert!(
                rendered.contains(marker),
                "missing control-plane parity marker `{marker}` in:\n{rendered}"
            );
        }
    }
}
