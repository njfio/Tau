//! Deploy process supervisor implementations.
//!
//! The gateway deploy endpoint owns the HTTP/state contract; this module owns
//! the optional process lifecycle boundary used by configured runtimes.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tau_core::current_unix_timestamp_ms;

const DEPLOY_PROCESS_STATUS_NOT_CONFIGURED: &str = "not_configured";
const DEPLOY_PROCESS_STATUS_NOT_RUNNING: &str = "not_running";
const DEPLOY_PROCESS_STATUS_RUNNING: &str = "running";
const DEPLOY_PROCESS_STATUS_STOPPED: &str = "stopped";
const DEPLOY_PROCESS_PROGRAM_ENV: &str = "TAU_GATEWAY_DEPLOY_PROCESS_PROGRAM";
const DEPLOY_PROCESS_ARGS_ENV: &str = "TAU_GATEWAY_DEPLOY_PROCESS_ARGS";
const DEPLOY_PROCESS_ARGS_JSON_ENV: &str = "TAU_GATEWAY_DEPLOY_PROCESS_ARGS_JSON";
const DEPLOY_PROCESS_TERMINATE_GRACE_ATTEMPTS: usize = 10;
const DEPLOY_PROCESS_TERMINATE_GRACE_DELAY: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayDeployProcessStartRequest {
    pub agent_id: String,
    pub profile: String,
    pub model: String,
    pub state_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayDeployProcessStartResult {
    pub process_id: String,
    pub pid: Option<u32>,
    pub status: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayDeployProcessStopResult {
    pub process_id: String,
    pub pid: Option<u32>,
    pub status: String,
    pub stopped_unix_ms: u64,
    pub stop_reason: String,
    pub exit_status: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayDeployProcessError {
    code: &'static str,
    message: String,
}

impl GatewayDeployProcessError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn start_failed(message: impl Into<String>) -> Self {
        Self::new("deploy_process_start_failed", message)
    }

    fn stop_failed(message: impl Into<String>) -> Self {
        Self::new("deploy_process_stop_failed", message)
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for GatewayDeployProcessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message.as_str())
    }
}

impl std::error::Error for GatewayDeployProcessError {}

pub trait GatewayDeployProcessSupervisor: Send + Sync {
    fn start(
        &self,
        request: GatewayDeployProcessStartRequest,
    ) -> Result<GatewayDeployProcessStartResult, GatewayDeployProcessError>;

    fn stop(
        &self,
        agent_id: &str,
        stop_reason: &str,
    ) -> Result<GatewayDeployProcessStopResult, GatewayDeployProcessError>;
}

#[derive(Debug, Default)]
pub struct NoopGatewayDeployProcessSupervisor;

impl GatewayDeployProcessSupervisor for NoopGatewayDeployProcessSupervisor {
    fn start(
        &self,
        request: GatewayDeployProcessStartRequest,
    ) -> Result<GatewayDeployProcessStartResult, GatewayDeployProcessError> {
        Ok(GatewayDeployProcessStartResult {
            process_id: format!("state-only:{}", request.agent_id),
            pid: None,
            status: DEPLOY_PROCESS_STATUS_NOT_CONFIGURED.to_string(),
            started_unix_ms: current_unix_timestamp_ms(),
        })
    }

    fn stop(
        &self,
        agent_id: &str,
        stop_reason: &str,
    ) -> Result<GatewayDeployProcessStopResult, GatewayDeployProcessError> {
        Ok(GatewayDeployProcessStopResult {
            process_id: format!("state-only:{agent_id}"),
            pid: None,
            status: DEPLOY_PROCESS_STATUS_NOT_CONFIGURED.to_string(),
            stopped_unix_ms: current_unix_timestamp_ms(),
            stop_reason: stop_reason.to_string(),
            exit_status: None,
        })
    }
}

#[derive(Debug)]
pub struct CommandGatewayDeployProcessSupervisor {
    program: PathBuf,
    args: Vec<String>,
    children: Mutex<BTreeMap<String, Child>>,
}

impl CommandGatewayDeployProcessSupervisor {
    pub fn new<I, S>(program: impl Into<PathBuf>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            children: Mutex::new(BTreeMap::new()),
        }
    }

    fn lock_children(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, BTreeMap<String, Child>>, GatewayDeployProcessError> {
        self.children
            .lock()
            .map_err(|_| GatewayDeployProcessError::start_failed("deploy process table poisoned"))
    }

    fn terminate_child(child: &mut Child) -> Result<Option<i32>, GatewayDeployProcessError> {
        let pid = child.id();
        if let Some(status) = child.try_wait().map_err(|error| {
            GatewayDeployProcessError::stop_failed(format!(
                "failed to inspect deploy process '{pid}' before termination: {error}"
            ))
        })? {
            return Ok(status.code());
        }

        request_graceful_process_termination(pid);
        for _ in 0..DEPLOY_PROCESS_TERMINATE_GRACE_ATTEMPTS {
            std::thread::sleep(DEPLOY_PROCESS_TERMINATE_GRACE_DELAY);
            if let Some(status) = child.try_wait().map_err(|error| {
                GatewayDeployProcessError::stop_failed(format!(
                    "failed to inspect deploy process '{pid}' during termination: {error}"
                ))
            })? {
                return Ok(status.code());
            }
        }

        child.kill().map_err(|error| {
            GatewayDeployProcessError::stop_failed(format!(
                "failed to terminate deploy process '{pid}': {error}"
            ))
        })?;
        child
            .wait()
            .map_err(|error| {
                GatewayDeployProcessError::stop_failed(format!(
                    "failed to wait for deploy process '{pid}' after termination: {error}"
                ))
            })
            .map(|status| status.code())
    }
}

impl GatewayDeployProcessSupervisor for CommandGatewayDeployProcessSupervisor {
    fn start(
        &self,
        request: GatewayDeployProcessStartRequest,
    ) -> Result<GatewayDeployProcessStartResult, GatewayDeployProcessError> {
        let mut children = self.lock_children()?;
        if let Some(mut existing) = children.remove(request.agent_id.as_str()) {
            let _ = Self::terminate_child(&mut existing);
        }

        let mut child = Command::new(&self.program)
            .args(&self.args)
            .env("TAU_DEPLOY_AGENT_ID", request.agent_id.as_str())
            .env("TAU_DEPLOY_PROFILE", request.profile.as_str())
            .env("TAU_DEPLOY_MODEL", request.model.as_str())
            .env("TAU_GATEWAY_STATE_DIR", request.state_dir.as_os_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                GatewayDeployProcessError::start_failed(format!(
                    "failed to spawn deploy process '{}': {error}",
                    self.program.display()
                ))
            })?;
        let pid = child.id();
        std::thread::sleep(Duration::from_millis(20));
        match child.try_wait() {
            Ok(Some(status)) => Err(GatewayDeployProcessError::start_failed(format!(
                "deploy process '{}' exited during startup with status {status}",
                self.program.display()
            ))),
            Ok(None) => {
                children.insert(request.agent_id.clone(), child);
                Ok(GatewayDeployProcessStartResult {
                    process_id: format!("gateway-deploy:{}:{pid}", request.agent_id),
                    pid: Some(pid),
                    status: DEPLOY_PROCESS_STATUS_RUNNING.to_string(),
                    started_unix_ms: current_unix_timestamp_ms(),
                })
            }
            Err(error) => Err(GatewayDeployProcessError::start_failed(format!(
                "failed to inspect deploy process '{pid}' during startup: {error}"
            ))),
        }
    }

    fn stop(
        &self,
        agent_id: &str,
        stop_reason: &str,
    ) -> Result<GatewayDeployProcessStopResult, GatewayDeployProcessError> {
        let mut children = self
            .children
            .lock()
            .map_err(|_| GatewayDeployProcessError::stop_failed("deploy process table poisoned"))?;
        let Some(mut child) = children.remove(agent_id) else {
            return Ok(GatewayDeployProcessStopResult {
                process_id: format!("gateway-deploy:{agent_id}:unknown"),
                pid: None,
                status: DEPLOY_PROCESS_STATUS_NOT_RUNNING.to_string(),
                stopped_unix_ms: current_unix_timestamp_ms(),
                stop_reason: stop_reason.to_string(),
                exit_status: None,
            });
        };
        let pid = child.id();
        let exit_status = Self::terminate_child(&mut child)?;
        Ok(GatewayDeployProcessStopResult {
            process_id: format!("gateway-deploy:{agent_id}:{pid}"),
            pid: Some(pid),
            status: DEPLOY_PROCESS_STATUS_STOPPED.to_string(),
            stopped_unix_ms: current_unix_timestamp_ms(),
            stop_reason: stop_reason.to_string(),
            exit_status,
        })
    }
}

impl Drop for CommandGatewayDeployProcessSupervisor {
    fn drop(&mut self) {
        if let Ok(mut children) = self.children.lock() {
            for (_, mut child) in std::mem::take(&mut *children) {
                let _ = CommandGatewayDeployProcessSupervisor::terminate_child(&mut child);
            }
        }
    }
}

#[cfg(unix)]
fn request_graceful_process_termination(pid: u32) {
    let _ = Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn request_graceful_process_termination(_pid: u32) {}

pub(super) fn gateway_deploy_process_supervisor_from_env() -> Arc<dyn GatewayDeployProcessSupervisor>
{
    let Ok(program) = std::env::var(DEPLOY_PROCESS_PROGRAM_ENV) else {
        return Arc::new(NoopGatewayDeployProcessSupervisor);
    };
    let trimmed_program = program.trim();
    if trimmed_program.is_empty() {
        return Arc::new(NoopGatewayDeployProcessSupervisor);
    }
    let args = deploy_process_args_from_env();
    Arc::new(CommandGatewayDeployProcessSupervisor::new(
        trimmed_program,
        args,
    ))
}

fn deploy_process_args_from_env() -> Vec<String> {
    std::env::var(DEPLOY_PROCESS_ARGS_JSON_ENV)
        .ok()
        .and_then(|raw| parse_deploy_process_args_json(raw.as_str()))
        .unwrap_or_else(|| {
            std::env::var(DEPLOY_PROCESS_ARGS_ENV)
                .map(|raw| parse_deploy_process_args(raw.as_str()))
                .unwrap_or_default()
        })
}

fn parse_deploy_process_args(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(str::to_string).collect()
}

fn parse_deploy_process_args_json(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<String>>(trimmed).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_parse_deploy_process_args_preserves_legacy_whitespace_contract() {
        assert_eq!(
            parse_deploy_process_args(" --port 9000 --mode worker "),
            vec!["--port", "9000", "--mode", "worker"]
        );
    }

    #[test]
    fn unit_parse_deploy_process_args_json_preserves_quoted_arguments() {
        assert_eq!(
            parse_deploy_process_args_json(r#"["--label","agent one","--flag"]"#),
            Some(vec![
                "--label".to_string(),
                "agent one".to_string(),
                "--flag".to_string()
            ])
        );
    }

    #[test]
    fn unit_parse_deploy_process_args_json_rejects_empty_or_invalid_json() {
        assert_eq!(parse_deploy_process_args_json("   "), None);
        assert_eq!(parse_deploy_process_args_json("--label agent"), None);
    }
}
