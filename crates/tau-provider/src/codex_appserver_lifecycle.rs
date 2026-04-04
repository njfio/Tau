//! Lifecycle management for the codex app-server subprocess.
//!
//! Automatically spawns `codex app-server --listen ws://127.0.0.1:<port>` and
//! provides the WebSocket URL to the client. Cleans up orphans on restart.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const PID_FILE_NAME: &str = "codex-appserver.pid";

/// Managed codex app-server process.
pub struct CodexAppServerProcess {
    child: Option<Child>,
    port: u16,
    pid_file: PathBuf,
}

impl CodexAppServerProcess {
    /// Spawn `codex app-server` on a free port. Kills any orphan from a previous run.
    pub fn spawn(codex_cli: &str, state_dir: &Path) -> Result<Self, String> {
        let pid_file = state_dir.join(PID_FILE_NAME);
        kill_orphan_from_pid_file(&pid_file);

        let port = find_free_port().map_err(|e| format!("failed to find free port: {e}"))?;
        let url = format!("ws://127.0.0.1:{port}");

        let child = Command::new(codex_cli)
            .args(["app-server", "--listen", &url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn codex app-server: {e}"))?;

        let pid = child.id();
        tracing::info!(pid = pid, port = port, "codex app-server spawned");

        // Write PID file for orphan cleanup on next restart
        if let Some(parent) = pid_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&pid_file, format!("{pid}\n{port}"));

        // Give the server a moment to bind
        std::thread::sleep(std::time::Duration::from_millis(1500));

        Ok(Self {
            child: Some(child),
            port,
            pid_file,
        })
    }

    /// WebSocket URL to connect to.
    pub fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

impl Drop for CodexAppServerProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::info!(pid = child.id(), "shutting down codex app-server");
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_file(&self.pid_file);
    }
}

fn find_free_port() -> Result<u16, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

fn kill_orphan_from_pid_file(pid_file: &Path) {
    let content = match std::fs::read_to_string(pid_file) {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut lines = content.lines();
    let Some(pid_str) = lines.next() else { return };
    let Ok(pid) = pid_str.trim().parse::<u32>() else {
        return;
    };

    tracing::info!(
        pid = pid,
        "killing orphan codex app-server from previous run"
    );

    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }

    let _ = std::fs::remove_file(pid_file);
}
