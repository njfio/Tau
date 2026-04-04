//! Lifecycle management for the codex app-server subprocess.
//!
//! Automatically spawns `codex app-server --listen ws://127.0.0.1:<port>` and
//! provides the WebSocket URL to the client. Cleans up on drop.

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};

/// Managed codex app-server process.
pub struct CodexAppServerProcess {
    child: Option<Child>,
    port: u16,
}

impl CodexAppServerProcess {
    /// Spawn `codex app-server` on a free port and return the managed process.
    pub fn spawn(codex_cli: &str) -> Result<Self, String> {
        let port = find_free_port().map_err(|e| format!("failed to find free port: {e}"))?;
        let url = format!("ws://127.0.0.1:{port}");

        let child = Command::new(codex_cli)
            .args(["app-server", "--listen", &url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn codex app-server: {e}"))?;

        tracing::info!(
            pid = child.id(),
            port = port,
            "codex app-server spawned"
        );

        // Give the server a moment to bind
        std::thread::sleep(std::time::Duration::from_millis(1500));

        Ok(Self {
            child: Some(child),
            port,
        })
    }

    /// WebSocket URL to connect to.
    pub fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Port the server is listening on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for CodexAppServerProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::info!(pid = child.id(), "shutting down codex app-server");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn find_free_port() -> Result<u16, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
