use super::*;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use tau_ai::{ChatRequest, ChatResponse};
use tempfile::{tempdir, TempDir};

pub(super) struct TauE2eHarness {
    workspace: TempDir,
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
    client: Client,
    token: String,
    scripted_client: Arc<ScriptedGatewayLlmClient>,
}

impl TauE2eHarness {
    pub(super) async fn new(scripted_responses: Vec<ChatResponse>) -> Self {
        Self::new_with_bulletin(scripted_responses, None).await
    }

    pub(super) async fn new_with_bulletin(
        scripted_responses: Vec<ChatResponse>,
        bulletin: Option<&str>,
    ) -> Self {
        let workspace = tempdir().expect("tempdir");
        let token = "secret".to_string();
        let scripted_client = Arc::new(ScriptedGatewayLlmClient::new(scripted_responses));
        let state = test_state_with_client_and_auth(
            workspace.path(),
            10_000,
            scripted_client.clone(),
            Arc::new(NoopGatewayToolRegistrar),
            GatewayOpenResponsesAuthMode::Token,
            Some(token.as_str()),
            None,
            60,
            120,
        );
        if let Some(snapshot) = bulletin {
            state.cortex.set_bulletin_for_test(snapshot);
        }
        let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
        Self {
            workspace,
            addr,
            handle,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("build reqwest client"),
            token,
            scripted_client,
        }
    }

    pub(super) fn workspace_root(&self) -> &Path {
        self.workspace.path()
    }

    fn auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.bearer_auth(self.token.as_str())
    }

    pub(super) async fn get_gateway_status(&self) -> reqwest::Response {
        self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, GATEWAY_STATUS_ENDPOINT)),
        )
        .send()
        .await
        .expect("gateway status")
    }

    pub(super) async fn post_openresponses(
        &self,
        input: &str,
        session_id: &str,
        stream: bool,
    ) -> reqwest::Response {
        self.auth(
            self.client
                .post(format!("http://{}{}", self.addr, OPENRESPONSES_ENDPOINT)),
        )
        .json(&json!({
            "input": input,
            "stream": stream,
            "metadata": {"session_id": session_id}
        }))
        .send()
        .await
        .expect("openresponses request")
    }

    pub(super) async fn list_sessions(&self) -> reqwest::Response {
        self.auth(self.client.get(format!(
            "http://{}{}?limit=200",
            self.addr, GATEWAY_SESSIONS_ENDPOINT
        )))
        .send()
        .await
        .expect("sessions list")
    }

    pub(super) async fn get_ops_shell(&self) -> reqwest::Response {
        self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, OPS_DASHBOARD_ENDPOINT)),
        )
        .send()
        .await
        .expect("ops shell")
    }

    pub(super) async fn post_ops_control_action(&self, action: &str) -> reqwest::Response {
        self.auth(self.client.post(format!(
            "http://{}{}",
            self.addr, OPS_DASHBOARD_CONTROL_ACTION_ENDPOINT
        )))
        .form(&[
            ("action", action),
            ("reason", "spec-3448-wave1"),
            ("theme", "dark"),
            ("sidebar", "expanded"),
        ])
        .send()
        .await
        .expect("ops control action")
    }

    pub(super) async fn get_dashboard_health(&self) -> reqwest::Response {
        self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, DASHBOARD_HEALTH_ENDPOINT)),
        )
        .send()
        .await
        .expect("dashboard health")
    }

    pub(super) async fn put_memory_entry(
        &self,
        session_key: &str,
        memory_id: &str,
        body: Value,
    ) -> reqwest::Response {
        let endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, memory_id);
        self.auth(self.client.put(format!("http://{}{}", self.addr, endpoint)))
            .json(&body)
            .send()
            .await
            .expect("put memory entry")
    }

    pub(super) async fn get_memory_entry(
        &self,
        session_key: &str,
        memory_id: &str,
    ) -> reqwest::Response {
        let endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, memory_id);
        self.auth(self.client.get(format!("http://{}{}", self.addr, endpoint)))
            .send()
            .await
            .expect("get memory entry")
    }

    pub(super) async fn delete_memory_entry(
        &self,
        session_key: &str,
        memory_id: &str,
    ) -> reqwest::Response {
        let endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, memory_id);
        self.auth(
            self.client
                .delete(format!("http://{}{}", self.addr, endpoint)),
        )
        .json(&json!({"policy_gate": MEMORY_WRITE_POLICY_GATE}))
        .send()
        .await
        .expect("delete memory entry")
    }

    pub(super) async fn search_memory(&self, session_key: &str, query: &str) -> reqwest::Response {
        let endpoint = expand_session_template(GATEWAY_MEMORY_ENDPOINT, session_key);
        let url = if query.is_empty() {
            format!("http://{}{}", self.addr, endpoint)
        } else {
            format!("http://{}{}?{}", self.addr, endpoint, query)
        };
        self.auth(self.client.get(url))
            .send()
            .await
            .expect("search memory")
    }

    pub(super) async fn put_legacy_memory(
        &self,
        session_key: &str,
        content: &str,
    ) -> reqwest::Response {
        let endpoint = expand_session_template(GATEWAY_MEMORY_ENDPOINT, session_key);
        self.auth(self.client.put(format!("http://{}{}", self.addr, endpoint)))
            .json(&json!({
                "content": content,
                "policy_gate": MEMORY_WRITE_POLICY_GATE
            }))
            .send()
            .await
            .expect("put legacy memory")
    }

    pub(super) async fn get_memory_graph(
        &self,
        session_key: &str,
        query: Option<&str>,
    ) -> reqwest::Response {
        let endpoint = expand_session_template(GATEWAY_MEMORY_GRAPH_ENDPOINT, session_key);
        let url = match query {
            Some(query) if !query.is_empty() => {
                format!("http://{}{}?{}", self.addr, endpoint, query)
            }
            _ => format!("http://{}{}", self.addr, endpoint),
        };
        self.auth(self.client.get(url))
            .send()
            .await
            .expect("get memory graph")
    }

    pub(super) async fn post_cortex_chat(&self, input: &str) -> reqwest::Response {
        self.auth(
            self.client
                .post(format!("http://{}{}", self.addr, CORTEX_CHAT_ENDPOINT)),
        )
        .json(&json!({ "input": input }))
        .send()
        .await
        .expect("post cortex chat")
    }

    pub(super) async fn get_cortex_status(&self) -> reqwest::Response {
        self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, CORTEX_STATUS_ENDPOINT)),
        )
        .send()
        .await
        .expect("get cortex status")
    }

    pub(super) async fn captured_llm_requests(&self) -> Vec<ChatRequest> {
        self.scripted_client.captured_requests().await
    }

    pub(super) async fn get_dashboard_widgets(&self) -> reqwest::Response {
        self.auth(self.client.get(format!(
            "http://{}{}",
            self.addr, DASHBOARD_WIDGETS_ENDPOINT
        )))
        .send()
        .await
        .expect("get dashboard widgets")
    }

    pub(super) async fn get_dashboard_alerts(&self) -> reqwest::Response {
        self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, DASHBOARD_ALERTS_ENDPOINT)),
        )
        .send()
        .await
        .expect("get dashboard alerts")
    }

    pub(super) async fn get_dashboard_queue_timeline(&self) -> reqwest::Response {
        self.auth(self.client.get(format!(
            "http://{}{}",
            self.addr, DASHBOARD_QUEUE_TIMELINE_ENDPOINT
        )))
        .send()
        .await
        .expect("get dashboard queue timeline")
    }

    pub(super) async fn get_dashboard_stream(
        &self,
        last_event_id: Option<&str>,
    ) -> reqwest::Response {
        let request = self.auth(
            self.client
                .get(format!("http://{}{}", self.addr, DASHBOARD_STREAM_ENDPOINT)),
        );
        let request = if let Some(last_event_id) = last_event_id {
            request.header("last-event-id", last_event_id)
        } else {
            request
        };
        request.send().await.expect("get dashboard stream")
    }

    pub(super) async fn read_sse_buffer(
        &self,
        response: reqwest::Response,
        stop_marker: &str,
    ) -> String {
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            let maybe_chunk =
                tokio::time::timeout(std::time::Duration::from_millis(300), stream.next()).await;
            let Ok(Some(Ok(chunk))) = maybe_chunk else {
                continue;
            };
            buffer.push_str(String::from_utf8_lossy(&chunk).as_ref());
            if buffer.contains(stop_marker) {
                break;
            }
        }
        buffer
    }
}

impl Drop for TauE2eHarness {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
