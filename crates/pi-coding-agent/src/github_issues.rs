use std::{
    collections::HashSet,
    future::pending,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use pi_agent_core::{Agent, AgentConfig, AgentEvent};
use pi_ai::LlmClient;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    current_unix_timestamp_ms, run_prompt_with_cancellation, write_text_atomic, PromptRunStatus,
    RenderOptions, SessionRuntime,
};
use crate::{session::SessionStore, tools::ToolPolicy};

const GITHUB_STATE_SCHEMA_VERSION: u32 = 1;
const EVENT_KEY_MARKER_PREFIX: &str = "<!-- rsbot-event-key:";
const EVENT_KEY_MARKER_SUFFIX: &str = " -->";

#[derive(Clone)]
pub(crate) struct GithubIssuesBridgeRuntimeConfig {
    pub client: Arc<dyn LlmClient>,
    pub model: String,
    pub system_prompt: String,
    pub max_turns: usize,
    pub tool_policy: ToolPolicy,
    pub turn_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub render_options: RenderOptions,
    pub session_lock_wait_ms: u64,
    pub session_lock_stale_ms: u64,
    pub state_dir: PathBuf,
    pub repo_slug: String,
    pub api_base: String,
    pub token: String,
    pub bot_login: Option<String>,
    pub poll_interval: Duration,
    pub include_issue_body: bool,
    pub include_edited_comments: bool,
    pub processed_event_cap: usize,
    pub retry_max_attempts: usize,
    pub retry_base_delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepoRef {
    owner: String,
    name: String,
}

impl RepoRef {
    fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        let (owner, name) = trimmed
            .split_once('/')
            .ok_or_else(|| anyhow!("invalid --github-repo '{raw}', expected owner/repo"))?;
        let owner = owner.trim();
        let name = name.trim();
        if owner.is_empty() || name.is_empty() || name.contains('/') {
            bail!("invalid --github-repo '{raw}', expected owner/repo");
        }
        Ok(Self {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }

    fn as_slug(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubIssuesBridgeState {
    schema_version: u32,
    #[serde(default)]
    last_issue_scan_at: Option<String>,
    #[serde(default)]
    processed_event_keys: Vec<String>,
}

impl Default for GithubIssuesBridgeState {
    fn default() -> Self {
        Self {
            schema_version: GITHUB_STATE_SCHEMA_VERSION,
            last_issue_scan_at: None,
            processed_event_keys: Vec::new(),
        }
    }
}

struct GithubIssuesBridgeStateStore {
    path: PathBuf,
    cap: usize,
    state: GithubIssuesBridgeState,
    processed_index: HashSet<String>,
}

impl GithubIssuesBridgeStateStore {
    fn load(path: PathBuf, cap: usize) -> Result<Self> {
        let mut state = if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read state file {}", path.display()))?;
            serde_json::from_str::<GithubIssuesBridgeState>(&raw).with_context(|| {
                format!(
                    "failed to parse github issues bridge state file {}",
                    path.display()
                )
            })?
        } else {
            GithubIssuesBridgeState::default()
        };

        if state.schema_version != GITHUB_STATE_SCHEMA_VERSION {
            bail!(
                "unsupported github issues bridge state schema: expected {}, found {}",
                GITHUB_STATE_SCHEMA_VERSION,
                state.schema_version
            );
        }

        let cap = cap.max(1);
        if state.processed_event_keys.len() > cap {
            let keep_from = state.processed_event_keys.len() - cap;
            state.processed_event_keys = state.processed_event_keys[keep_from..].to_vec();
        }
        let processed_index = state
            .processed_event_keys
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        Ok(Self {
            path,
            cap,
            state,
            processed_index,
        })
    }

    fn contains(&self, key: &str) -> bool {
        self.processed_index.contains(key)
    }

    fn mark_processed(&mut self, key: &str) -> bool {
        if self.processed_index.contains(key) {
            return false;
        }
        self.state.processed_event_keys.push(key.to_string());
        self.processed_index.insert(key.to_string());
        while self.state.processed_event_keys.len() > self.cap {
            let removed = self.state.processed_event_keys.remove(0);
            self.processed_index.remove(&removed);
        }
        true
    }

    fn last_issue_scan_at(&self) -> Option<&str> {
        self.state.last_issue_scan_at.as_deref()
    }

    fn update_last_issue_scan_at(&mut self, value: Option<String>) -> bool {
        if self.state.last_issue_scan_at == value {
            return false;
        }
        self.state.last_issue_scan_at = value;
        true
    }

    fn save(&self) -> Result<()> {
        let mut payload =
            serde_json::to_string_pretty(&self.state).context("failed to serialize state")?;
        payload.push('\n');
        write_text_atomic(&self.path, &payload)
            .with_context(|| format!("failed to write state file {}", self.path.display()))?;
        Ok(())
    }
}

#[derive(Clone)]
struct JsonlEventLog {
    path: PathBuf,
    file: Arc<Mutex<std::fs::File>>,
}

impl JsonlEventLog {
    fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;
        Ok(Self {
            path,
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn append(&self, value: &Value) -> Result<()> {
        let line = serde_json::to_string(value).context("failed to encode log event")?;
        let mut file = self
            .file
            .lock()
            .map_err(|_| anyhow!("event log mutex is poisoned"))?;
        writeln!(file, "{line}")
            .with_context(|| format!("failed to append to {}", self.path.display()))?;
        file.flush()
            .with_context(|| format!("failed to flush {}", self.path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GithubUser {
    login: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GithubIssue {
    id: u64,
    number: u64,
    title: String,
    body: Option<String>,
    created_at: String,
    updated_at: String,
    user: GithubUser,
    #[serde(default)]
    pull_request: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GithubIssueComment {
    id: u64,
    body: Option<String>,
    created_at: String,
    updated_at: String,
    user: GithubUser,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubCommentCreateResponse {
    id: u64,
    html_url: Option<String>,
}

#[derive(Clone)]
struct GithubApiClient {
    http: reqwest::Client,
    api_base: String,
    repo: RepoRef,
    retry_max_attempts: usize,
    retry_base_delay_ms: u64,
}

impl GithubApiClient {
    fn new(
        api_base: String,
        token: String,
        repo: RepoRef,
        request_timeout_ms: u64,
        retry_max_attempts: usize,
        retry_base_delay_ms: u64,
    ) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("rsBot-github-issues-bridge"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "x-github-api-version",
            reqwest::header::HeaderValue::from_static("2022-11-28"),
        );
        let auth_header = format!("Bearer {}", token.trim());
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_header)
                .context("invalid github authorization header")?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(request_timeout_ms.max(1)))
            .build()
            .context("failed to create github api client")?;
        Ok(Self {
            http: client,
            api_base: api_base.trim_end_matches('/').to_string(),
            repo,
            retry_max_attempts: retry_max_attempts.max(1),
            retry_base_delay_ms: retry_base_delay_ms.max(1),
        })
    }

    async fn resolve_bot_login(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct Viewer {
            login: String,
        }

        let viewer: Viewer = self
            .request_json("resolve bot login", || {
                self.http.get(format!("{}/user", self.api_base))
            })
            .await?;
        Ok(viewer.login)
    }

    async fn list_updated_issues(&self, since: Option<&str>) -> Result<Vec<GithubIssue>> {
        let mut page = 1_u32;
        let mut rows = Vec::new();
        loop {
            let mut request = self.http.get(format!(
                "{}/repos/{}/{}/issues",
                self.api_base, self.repo.owner, self.repo.name
            ));
            request = request.query(&[
                ("state", "open"),
                ("sort", "updated"),
                ("direction", "asc"),
                ("per_page", "100"),
                ("page", &page.to_string()),
            ]);
            if let Some(since_value) = since {
                request = request.query(&[("since", since_value)]);
            }
            let chunk: Vec<GithubIssue> = self
                .request_json("list issues", || {
                    request.try_clone().expect("cloned request")
                })
                .await?;
            let chunk_len = chunk.len();
            rows.extend(
                chunk
                    .into_iter()
                    .filter(|issue| issue.pull_request.is_none()),
            );
            if chunk_len < 100 {
                break;
            }
            page = page.saturating_add(1);
        }
        Ok(rows)
    }

    async fn list_issue_comments(&self, issue_number: u64) -> Result<Vec<GithubIssueComment>> {
        let mut page = 1_u32;
        let mut rows = Vec::new();
        loop {
            let request = self
                .http
                .get(format!(
                    "{}/repos/{}/{}/issues/{}/comments",
                    self.api_base, self.repo.owner, self.repo.name, issue_number
                ))
                .query(&[
                    ("sort", "created"),
                    ("direction", "asc"),
                    ("per_page", "100"),
                    ("page", &page.to_string()),
                ]);
            let chunk: Vec<GithubIssueComment> = self
                .request_json("list issue comments", || {
                    request.try_clone().expect("cloned request")
                })
                .await?;
            let chunk_len = chunk.len();
            rows.extend(chunk);
            if chunk_len < 100 {
                break;
            }
            page = page.saturating_add(1);
        }
        Ok(rows)
    }

    async fn create_issue_comment(
        &self,
        issue_number: u64,
        body: &str,
    ) -> Result<GithubCommentCreateResponse> {
        let payload = json!({ "body": body });
        self.request_json("create issue comment", || {
            self.http
                .post(format!(
                    "{}/repos/{}/{}/issues/{}/comments",
                    self.api_base, self.repo.owner, self.repo.name, issue_number
                ))
                .json(&payload)
        })
        .await
    }

    async fn request_json<T, F>(&self, operation: &str, mut request_builder: F) -> Result<T>
    where
        T: DeserializeOwned,
        F: FnMut() -> reqwest::RequestBuilder,
    {
        let mut attempt = 0_usize;
        loop {
            attempt = attempt.saturating_add(1);
            let response = request_builder()
                .header(
                    "x-rsbot-retry-attempt",
                    attempt.saturating_sub(1).to_string(),
                )
                .send()
                .await;
            match response {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let parsed = response
                            .json::<T>()
                            .await
                            .with_context(|| format!("failed to decode github {operation}"))?;
                        return Ok(parsed);
                    }

                    let retry_after = parse_retry_after(response.headers());
                    let body = response.text().await.unwrap_or_default();
                    if attempt < self.retry_max_attempts
                        && is_retryable_github_status(status.as_u16())
                    {
                        tokio::time::sleep(retry_delay(
                            self.retry_base_delay_ms,
                            attempt,
                            retry_after,
                        ))
                        .await;
                        continue;
                    }

                    bail!(
                        "github api {operation} failed with status {}: {}",
                        status.as_u16(),
                        truncate_for_error(&body, 800)
                    );
                }
                Err(error) => {
                    if attempt < self.retry_max_attempts && is_retryable_transport_error(&error) {
                        tokio::time::sleep(retry_delay(self.retry_base_delay_ms, attempt, None))
                            .await;
                        continue;
                    }
                    return Err(error)
                        .with_context(|| format!("github api {operation} request failed"));
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum GithubBridgeEventKind {
    Opened,
    CommentCreated,
    CommentEdited,
}

impl GithubBridgeEventKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Opened => "issue_opened",
            Self::CommentCreated => "issue_comment_created",
            Self::CommentEdited => "issue_comment_edited",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubBridgeEvent {
    key: String,
    kind: GithubBridgeEventKind,
    issue_number: u64,
    issue_title: String,
    author_login: String,
    occurred_at: String,
    body: String,
    raw_payload: Value,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct PromptUsageSummary {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    request_duration_ms: u64,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct PromptRunReport {
    run_id: String,
    model: String,
    status: PromptRunStatus,
    assistant_reply: String,
    usage: PromptUsageSummary,
}

#[derive(Debug, Default)]
pub(crate) struct PollCycleReport {
    pub discovered_events: usize,
    pub processed_events: usize,
    pub skipped_duplicate_events: usize,
    pub failed_events: usize,
}

pub(crate) async fn run_github_issues_bridge(
    config: GithubIssuesBridgeRuntimeConfig,
) -> Result<()> {
    let mut runtime = GithubIssuesBridgeRuntime::new(config).await?;
    runtime.run().await
}

struct GithubIssuesBridgeRuntime {
    config: GithubIssuesBridgeRuntimeConfig,
    repo: RepoRef,
    github_client: GithubApiClient,
    state_store: GithubIssuesBridgeStateStore,
    inbound_log: JsonlEventLog,
    outbound_log: JsonlEventLog,
    bot_login: String,
    repository_state_dir: PathBuf,
}

impl GithubIssuesBridgeRuntime {
    async fn new(config: GithubIssuesBridgeRuntimeConfig) -> Result<Self> {
        let repo = RepoRef::parse(&config.repo_slug)?;
        let github_client = GithubApiClient::new(
            config.api_base.clone(),
            config.token.clone(),
            repo.clone(),
            config.request_timeout_ms,
            config.retry_max_attempts,
            config.retry_base_delay_ms,
        )?;
        let bot_login = match config.bot_login.clone() {
            Some(login) if !login.trim().is_empty() => login.trim().to_string(),
            _ => github_client.resolve_bot_login().await?,
        };
        let repository_state_dir = config
            .state_dir
            .join(sanitize_for_path(&format!("{}__{}", repo.owner, repo.name)));
        std::fs::create_dir_all(&repository_state_dir)
            .with_context(|| format!("failed to create {}", repository_state_dir.display()))?;

        let state_store = GithubIssuesBridgeStateStore::load(
            repository_state_dir.join("state.json"),
            config.processed_event_cap,
        )?;
        let inbound_log = JsonlEventLog::open(repository_state_dir.join("inbound-events.jsonl"))?;
        let outbound_log = JsonlEventLog::open(repository_state_dir.join("outbound-events.jsonl"))?;
        Ok(Self {
            config,
            repo,
            github_client,
            state_store,
            inbound_log,
            outbound_log,
            bot_login,
            repository_state_dir,
        })
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            match self.poll_once().await {
                Ok(report) => {
                    println!(
                        "github bridge poll: repo={} discovered={} processed={} duplicate_skips={} failed={}",
                        self.repo.as_slug(),
                        report.discovered_events,
                        report.processed_events,
                        report.skipped_duplicate_events,
                        report.failed_events
                    );
                }
                Err(error) => {
                    eprintln!("github bridge poll error: {error}");
                }
            }

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("github bridge shutdown requested");
                    return Ok(());
                }
                _ = tokio::time::sleep(self.config.poll_interval) => {}
            }
        }
    }

    async fn poll_once(&mut self) -> Result<PollCycleReport> {
        let issues = self
            .github_client
            .list_updated_issues(self.state_store.last_issue_scan_at())
            .await?;
        let mut report = PollCycleReport::default();
        let mut state_dirty = false;
        let mut latest_issue_scan = self.state_store.last_issue_scan_at().map(str::to_string);

        for issue in issues {
            latest_issue_scan = match latest_issue_scan {
                Some(existing) if existing >= issue.updated_at => Some(existing),
                _ => Some(issue.updated_at.clone()),
            };

            let comments = self.github_client.list_issue_comments(issue.number).await?;
            let known_event_keys = comments
                .iter()
                .filter(|comment| comment.user.login == self.bot_login)
                .flat_map(|comment| {
                    comment
                        .body
                        .as_deref()
                        .map(extract_footer_event_keys)
                        .unwrap_or_default()
                })
                .collect::<HashSet<_>>();

            for key in &known_event_keys {
                if self.state_store.mark_processed(key) {
                    state_dirty = true;
                }
            }

            let events = collect_issue_events(
                &issue,
                &comments,
                &self.bot_login,
                self.config.include_issue_body,
                self.config.include_edited_comments,
            );
            report.discovered_events = report.discovered_events.saturating_add(events.len());

            for event in events {
                if self.state_store.contains(&event.key) || known_event_keys.contains(&event.key) {
                    report.skipped_duplicate_events =
                        report.skipped_duplicate_events.saturating_add(1);
                    continue;
                }

                self.inbound_log.append(&json!({
                    "timestamp_unix_ms": current_unix_timestamp_ms(),
                    "repo": self.repo.as_slug(),
                    "event_key": event.key,
                    "kind": event.kind.as_str(),
                    "issue_number": event.issue_number,
                    "payload": event.raw_payload,
                }))?;

                match self.process_event(&event).await {
                    Ok(outbound) => {
                        self.outbound_log.append(&outbound)?;
                        if self.state_store.mark_processed(&event.key) {
                            state_dirty = true;
                        }
                        report.processed_events = report.processed_events.saturating_add(1);
                    }
                    Err(error) => {
                        report.failed_events = report.failed_events.saturating_add(1);
                        eprintln!(
                            "github bridge event failed: repo={} issue=#{} key={} error={error}",
                            self.repo.as_slug(),
                            event.issue_number,
                            event.key
                        );
                    }
                }
            }
        }

        if self
            .state_store
            .update_last_issue_scan_at(latest_issue_scan)
        {
            state_dirty = true;
        }
        if state_dirty {
            self.state_store.save()?;
        }
        Ok(report)
    }

    async fn process_event(&self, event: &GithubBridgeEvent) -> Result<Value> {
        let run = self.run_prompt_for_event(event).await?;
        let comment_body = render_issue_comment_response(event, &run);
        let posted = self
            .github_client
            .create_issue_comment(event.issue_number, &comment_body)
            .await?;

        Ok(json!({
            "timestamp_unix_ms": current_unix_timestamp_ms(),
            "repo": self.repo.as_slug(),
            "event_key": event.key,
            "issue_number": event.issue_number,
            "run_id": run.run_id,
            "status": format!("{:?}", run.status).to_lowercase(),
            "posted_comment_id": posted.id,
            "posted_comment_url": posted.html_url,
            "model": run.model,
            "usage": {
                "input_tokens": run.usage.input_tokens,
                "output_tokens": run.usage.output_tokens,
                "total_tokens": run.usage.total_tokens,
                "request_duration_ms": run.usage.request_duration_ms,
                "finish_reason": run.usage.finish_reason,
            }
        }))
    }

    async fn run_prompt_for_event(&self, event: &GithubBridgeEvent) -> Result<PromptRunReport> {
        let session_path = session_path_for_issue(&self.repository_state_dir, event.issue_number);
        let mut agent = Agent::new(
            self.config.client.clone(),
            AgentConfig {
                model: self.config.model.clone(),
                system_prompt: self.config.system_prompt.clone(),
                max_turns: self.config.max_turns,
                temperature: Some(0.0),
                max_tokens: None,
            },
        );
        crate::tools::register_builtin_tools(&mut agent, self.config.tool_policy.clone());

        let usage = Arc::new(Mutex::new(PromptUsageSummary::default()));
        agent.subscribe({
            let usage = usage.clone();
            move |event| {
                if let AgentEvent::TurnEnd {
                    usage: turn_usage,
                    request_duration_ms,
                    finish_reason,
                    ..
                } = event
                {
                    if let Ok(mut guard) = usage.lock() {
                        guard.input_tokens =
                            guard.input_tokens.saturating_add(turn_usage.input_tokens);
                        guard.output_tokens =
                            guard.output_tokens.saturating_add(turn_usage.output_tokens);
                        guard.total_tokens =
                            guard.total_tokens.saturating_add(turn_usage.total_tokens);
                        guard.request_duration_ms = guard
                            .request_duration_ms
                            .saturating_add(*request_duration_ms);
                        guard.finish_reason = finish_reason.clone();
                    }
                }
            }
        });

        let mut session_runtime = Some(initialize_issue_session_runtime(
            &session_path,
            &self.config.system_prompt,
            self.config.session_lock_wait_ms,
            self.config.session_lock_stale_ms,
            &mut agent,
        )?);

        let prompt = render_event_prompt(&self.repo, event);
        let start_index = agent.messages().len();
        let status = run_prompt_with_cancellation(
            &mut agent,
            &mut session_runtime,
            &prompt,
            self.config.turn_timeout_ms,
            pending::<()>(),
            self.config.render_options,
        )
        .await?;
        let assistant_reply = collect_assistant_reply(&agent.messages()[start_index..]);
        let usage = usage
            .lock()
            .map_err(|_| anyhow!("prompt usage lock is poisoned"))?
            .clone();
        let run_id = format!(
            "gh-{}-{}-{}",
            event.issue_number,
            current_unix_timestamp_ms(),
            short_key_hash(&event.key)
        );
        Ok(PromptRunReport {
            run_id,
            model: self.config.model.clone(),
            status,
            assistant_reply,
            usage,
        })
    }
}

fn initialize_issue_session_runtime(
    session_path: &Path,
    system_prompt: &str,
    lock_wait_ms: u64,
    lock_stale_ms: u64,
    agent: &mut Agent,
) -> Result<SessionRuntime> {
    if let Some(parent) = session_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    let mut store = SessionStore::load(session_path)?;
    store.set_lock_policy(lock_wait_ms.max(1), lock_stale_ms);
    let active_head = store.ensure_initialized(system_prompt)?;
    let lineage = store.lineage_messages(active_head)?;
    if !lineage.is_empty() {
        agent.replace_messages(lineage);
    }
    Ok(SessionRuntime { store, active_head })
}

fn collect_assistant_reply(messages: &[pi_ai::Message]) -> String {
    let content = messages
        .iter()
        .filter(|message| message.role == pi_ai::MessageRole::Assistant)
        .map(|message| message.text_content())
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if content.trim().is_empty() {
        "I couldn't generate a textual response for this event.".to_string()
    } else {
        content
    }
}

fn render_event_prompt(repo: &RepoRef, event: &GithubBridgeEvent) -> String {
    format!(
        "You are responding as rsBot inside GitHub issues.\nRepository: {}\nIssue: #{} ({})\nAuthor: @{}\nEvent: {}\n\nUser message:\n{}\n\nProvide a direct, actionable response suitable for a GitHub issue comment.",
        repo.as_slug(),
        event.issue_number,
        event.issue_title,
        event.author_login,
        event.kind.as_str(),
        event.body
    )
}

fn render_issue_comment_response(event: &GithubBridgeEvent, run: &PromptRunReport) -> String {
    let mut body = run.assistant_reply.trim().to_string();
    if body.is_empty() {
        body = "I couldn't generate a textual response for this event.".to_string();
    }
    let usage = &run.usage;
    let status = format!("{:?}", run.status).to_lowercase();
    body.push_str("\n\n---\n");
    body.push_str(&format!(
        "{EVENT_KEY_MARKER_PREFIX}{}{EVENT_KEY_MARKER_SUFFIX}\n_rsBot run `{}` | status `{}` | model `{}` | tokens in/out/total `{}/{}/{}` | cost `unavailable`_",
        event.key,
        run.run_id,
        status,
        run.model,
        usage.input_tokens,
        usage.output_tokens,
        usage.total_tokens
    ));
    body
}

fn collect_issue_events(
    issue: &GithubIssue,
    comments: &[GithubIssueComment],
    bot_login: &str,
    include_issue_body: bool,
    include_edited_comments: bool,
) -> Vec<GithubBridgeEvent> {
    let mut events = Vec::new();
    if include_issue_body
        && issue.user.login != bot_login
        && !issue.body.as_deref().unwrap_or_default().trim().is_empty()
    {
        let body = issue.body.clone().unwrap_or_default();
        events.push(GithubBridgeEvent {
            key: format!("issue-opened:{}", issue.id),
            kind: GithubBridgeEventKind::Opened,
            issue_number: issue.number,
            issue_title: issue.title.clone(),
            author_login: issue.user.login.clone(),
            occurred_at: issue.created_at.clone(),
            body,
            raw_payload: serde_json::to_value(issue).unwrap_or(Value::Null),
        });
    }

    for comment in comments {
        if comment.user.login == bot_login {
            continue;
        }
        let body = comment
            .body
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string();
        if body.is_empty() {
            continue;
        }
        let is_edit = comment.updated_at != comment.created_at;
        if is_edit && !include_edited_comments {
            continue;
        }
        let (key, kind) = if is_edit {
            (
                format!("issue-comment-edited:{}:{}", comment.id, comment.updated_at),
                GithubBridgeEventKind::CommentEdited,
            )
        } else {
            (
                format!("issue-comment-created:{}", comment.id),
                GithubBridgeEventKind::CommentCreated,
            )
        };
        events.push(GithubBridgeEvent {
            key,
            kind,
            issue_number: issue.number,
            issue_title: issue.title.clone(),
            author_login: comment.user.login.clone(),
            occurred_at: comment.created_at.clone(),
            body: body.to_string(),
            raw_payload: serde_json::to_value(comment).unwrap_or(Value::Null),
        });
    }

    events.sort_by(|left, right| {
        left.occurred_at
            .cmp(&right.occurred_at)
            .then(left.key.cmp(&right.key))
    });
    events
}

fn session_path_for_issue(repo_state_dir: &Path, issue_number: u64) -> PathBuf {
    repo_state_dir
        .join("sessions")
        .join(format!("issue-{}.jsonl", issue_number))
}

fn sanitize_for_path(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn extract_footer_event_keys(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut cursor = text;
    while let Some(start) = cursor.find(EVENT_KEY_MARKER_PREFIX) {
        let after_start = &cursor[start + EVENT_KEY_MARKER_PREFIX.len()..];
        let Some(end) = after_start.find(EVENT_KEY_MARKER_SUFFIX) else {
            break;
        };
        let key = after_start[..end].trim();
        if !key.is_empty() {
            keys.push(key.to_string());
        }
        cursor = &after_start[end + EVENT_KEY_MARKER_SUFFIX.len()..];
    }
    keys
}

fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let raw = headers.get("retry-after")?.to_str().ok()?;
    let seconds = raw.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds))
}

fn retry_delay(base_delay_ms: u64, attempt: usize, retry_after: Option<Duration>) -> Duration {
    if let Some(delay) = retry_after {
        return delay.max(Duration::from_millis(base_delay_ms));
    }
    let exponent = attempt.saturating_sub(1).min(10) as u32;
    let scaled = base_delay_ms.saturating_mul(2_u64.saturating_pow(exponent));
    Duration::from_millis(scaled.min(30_000))
}

fn is_retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request()
}

fn is_retryable_github_status(status: u16) -> bool {
    status == 429 || status >= 500
}

fn truncate_for_error(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut truncated = text.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn short_key_hash(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        digest[0], digest[1], digest[2], digest[3]
    )
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Arc, time::Duration};

    use async_trait::async_trait;
    use httpmock::prelude::*;
    use pi_ai::{ChatRequest, ChatResponse, ChatUsage, LlmClient, Message, PiAiError};
    use serde_json::json;
    use tempfile::tempdir;

    use super::{
        collect_issue_events, extract_footer_event_keys, is_retryable_github_status, retry_delay,
        sanitize_for_path, session_path_for_issue, GithubApiClient, GithubBridgeEventKind,
        GithubIssue, GithubIssueComment, GithubIssuesBridgeRuntime,
        GithubIssuesBridgeRuntimeConfig, GithubIssuesBridgeStateStore, GithubUser, RepoRef,
    };
    use crate::{tools::ToolPolicy, RenderOptions};

    struct StaticReplyClient;

    #[async_trait]
    impl LlmClient for StaticReplyClient {
        async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, PiAiError> {
            Ok(ChatResponse {
                message: Message::assistant_text("bridge reply"),
                finish_reason: Some("stop".to_string()),
                usage: ChatUsage {
                    input_tokens: 11,
                    output_tokens: 7,
                    total_tokens: 18,
                },
            })
        }
    }

    fn test_bridge_config(base_url: &str, state_dir: &Path) -> GithubIssuesBridgeRuntimeConfig {
        GithubIssuesBridgeRuntimeConfig {
            client: Arc::new(StaticReplyClient),
            model: "openai/gpt-4o-mini".to_string(),
            system_prompt: "You are rsBot.".to_string(),
            max_turns: 4,
            tool_policy: ToolPolicy::new(vec![state_dir.to_path_buf()]),
            turn_timeout_ms: 0,
            request_timeout_ms: 3_000,
            render_options: RenderOptions {
                stream_output: false,
                stream_delay_ms: 0,
            },
            session_lock_wait_ms: 2_000,
            session_lock_stale_ms: 30_000,
            state_dir: state_dir.to_path_buf(),
            repo_slug: "owner/repo".to_string(),
            api_base: base_url.to_string(),
            token: "test-token".to_string(),
            bot_login: Some("rsbot".to_string()),
            poll_interval: Duration::from_millis(1),
            include_issue_body: false,
            include_edited_comments: true,
            processed_event_cap: 32,
            retry_max_attempts: 3,
            retry_base_delay_ms: 5,
        }
    }

    #[test]
    fn unit_repo_ref_parse_accepts_owner_repo_shape() {
        let repo = RepoRef::parse("njfio/rsBot").expect("parse repo");
        assert_eq!(repo.owner, "njfio");
        assert_eq!(repo.name, "rsBot");

        let error = RepoRef::parse("missing").expect_err("invalid repo should fail");
        assert!(error.to_string().contains("expected owner/repo"));
    }

    #[test]
    fn functional_collect_issue_events_supports_created_and_edited_comments() {
        let issue = GithubIssue {
            id: 100,
            number: 42,
            title: "Issue".to_string(),
            body: Some("initial issue body".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:10Z".to_string(),
            user: GithubUser {
                login: "alice".to_string(),
            },
            pull_request: None,
        };
        let comments = vec![
            GithubIssueComment {
                id: 1,
                body: Some("first".to_string()),
                created_at: "2026-01-01T00:00:01Z".to_string(),
                updated_at: "2026-01-01T00:00:01Z".to_string(),
                user: GithubUser {
                    login: "bob".to_string(),
                },
            },
            GithubIssueComment {
                id: 2,
                body: Some("second edited".to_string()),
                created_at: "2026-01-01T00:00:02Z".to_string(),
                updated_at: "2026-01-01T00:10:02Z".to_string(),
                user: GithubUser {
                    login: "carol".to_string(),
                },
            },
        ];
        let events = collect_issue_events(&issue, &comments, "rsbot", true, true);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].kind, GithubBridgeEventKind::Opened);
        assert_eq!(events[1].kind, GithubBridgeEventKind::CommentCreated);
        assert_eq!(events[2].kind, GithubBridgeEventKind::CommentEdited);
    }

    #[test]
    fn regression_state_store_caps_processed_event_history() {
        let temp = tempdir().expect("tempdir");
        let state_path = temp.path().join("state.json");
        let mut state = GithubIssuesBridgeStateStore::load(state_path, 2).expect("load store");
        assert!(state.mark_processed("a"));
        assert!(state.mark_processed("b"));
        assert!(state.mark_processed("c"));
        assert!(!state.contains("a"));
        assert!(state.contains("b"));
        assert!(state.contains("c"));
    }

    #[test]
    fn unit_retry_helpers_identify_retryable_status_and_delays() {
        assert!(is_retryable_github_status(429));
        assert!(is_retryable_github_status(500));
        assert!(!is_retryable_github_status(404));
        let delay = retry_delay(100, 3, None);
        assert_eq!(delay, Duration::from_millis(400));
    }

    #[test]
    fn unit_footer_key_extraction_and_path_helpers_are_stable() {
        let text = "hello\n<!-- rsbot-event-key:abc -->\nworld\n<!-- rsbot-event-key:def -->";
        let keys = extract_footer_event_keys(text);
        assert_eq!(keys, vec!["abc".to_string(), "def".to_string()]);

        let root = Path::new("/tmp/state");
        let session = session_path_for_issue(root, 9);
        assert!(session.ends_with("sessions/issue-9.jsonl"));
        assert_eq!(sanitize_for_path("owner/repo"), "owner_repo");
    }

    #[tokio::test]
    async fn integration_github_api_client_retries_rate_limits() {
        let server = MockServer::start();
        let first = server.mock(|when, then| {
            when.method(GET)
                .path("/repos/owner/repo/issues")
                .header("x-rsbot-retry-attempt", "0");
            then.status(429)
                .header("retry-after", "0")
                .body("rate limit");
        });
        let second = server.mock(|when, then| {
            when.method(GET)
                .path("/repos/owner/repo/issues")
                .header("x-rsbot-retry-attempt", "1");
            then.status(200).json_body(json!([]));
        });

        let repo = RepoRef::parse("owner/repo").expect("repo parse");
        let client =
            GithubApiClient::new(server.base_url(), "token".to_string(), repo, 2_000, 3, 1)
                .expect("client");
        let issues = client
            .list_updated_issues(None)
            .await
            .expect("list issues should eventually succeed");
        assert!(issues.is_empty());
        assert_eq!(first.calls(), 1);
        assert_eq!(second.calls(), 1);
    }

    #[tokio::test]
    async fn integration_bridge_poll_processes_issue_comment_and_posts_reply() {
        let server = MockServer::start();
        let issues = server.mock(|when, then| {
            when.method(GET).path("/repos/owner/repo/issues");
            then.status(200).json_body(json!([{
                "id": 10,
                "number": 7,
                "title": "Bridge me",
                "body": "",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:05Z",
                "user": {"login":"alice"}
            }]));
        });
        let comments = server.mock(|when, then| {
            when.method(GET).path("/repos/owner/repo/issues/7/comments");
            then.status(200).json_body(json!([{
                "id": 200,
                "body": "hello from issue stream",
                "created_at": "2026-01-01T00:00:01Z",
                "updated_at": "2026-01-01T00:00:01Z",
                "user": {"login":"alice"}
            }]));
        });
        let post = server.mock(|when, then| {
            when.method(POST)
                .path("/repos/owner/repo/issues/7/comments")
                .body_includes("bridge reply")
                .body_includes("rsbot-event-key:issue-comment-created:200");
            then.status(201).json_body(json!({
                "id": 901,
                "html_url": "https://example.test/comment/901"
            }));
        });

        let temp = tempdir().expect("tempdir");
        let config = test_bridge_config(&server.base_url(), temp.path());
        let mut runtime = GithubIssuesBridgeRuntime::new(config)
            .await
            .expect("runtime");
        let report = runtime.poll_once().await.expect("poll");
        assert_eq!(report.discovered_events, 1);
        assert_eq!(report.processed_events, 1);
        assert_eq!(report.failed_events, 0);
        issues.assert_calls(1);
        comments.assert_calls(1);
        post.assert_calls(1);

        let outbound = std::fs::read_to_string(
            temp.path()
                .join("owner__repo")
                .join("outbound-events.jsonl"),
        )
        .expect("read outbound log");
        assert!(outbound.contains("\"posted_comment_id\":901"));
    }

    #[tokio::test]
    async fn regression_bridge_poll_replay_does_not_duplicate_responses() {
        let server = MockServer::start();
        let _issues = server.mock(|when, then| {
            when.method(GET).path("/repos/owner/repo/issues");
            then.status(200).json_body(json!([{
                "id": 11,
                "number": 8,
                "title": "Replay",
                "body": "",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:05Z",
                "user": {"login":"alice"}
            }]));
        });
        let _comments = server.mock(|when, then| {
            when.method(GET).path("/repos/owner/repo/issues/8/comments");
            then.status(200).json_body(json!([{
                "id": 201,
                "body": "same comment every poll",
                "created_at": "2026-01-01T00:00:01Z",
                "updated_at": "2026-01-01T00:00:01Z",
                "user": {"login":"alice"}
            }]));
        });
        let post = server.mock(|when, then| {
            when.method(POST)
                .path("/repos/owner/repo/issues/8/comments");
            then.status(201).json_body(json!({
                "id": 902,
                "html_url": "https://example.test/comment/902"
            }));
        });

        let temp = tempdir().expect("tempdir");
        let config = test_bridge_config(&server.base_url(), temp.path());
        let mut runtime = GithubIssuesBridgeRuntime::new(config)
            .await
            .expect("runtime");
        let first = runtime.poll_once().await.expect("first poll");
        assert_eq!(first.processed_events, 1);
        let second = runtime.poll_once().await.expect("second poll");
        assert_eq!(second.processed_events, 0);
        assert_eq!(second.skipped_duplicate_events, 1);
        post.assert_calls(1);
    }
}
