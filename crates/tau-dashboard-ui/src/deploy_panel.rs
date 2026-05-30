use leptos::prelude::*;

use crate::TauOpsDashboardDeploySnapshot;

fn encode_ops_path_segment(raw: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in raw.as_bytes().iter().copied() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => {
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded
}

pub(crate) fn render_tau_ops_deploy_process_lifecycle(
    snapshot: TauOpsDashboardDeploySnapshot,
    theme: &str,
    sidebar: &str,
    session_key: &str,
) -> impl IntoView {
    let deploy_state_source = snapshot.state_source;
    let deploy_state_status = snapshot.state_status;
    let deploy_agent_count = snapshot.agent_count.to_string();
    let deploy_running_count = snapshot.running_count.to_string();
    let deploy_stopped_count = snapshot.stopped_count.to_string();
    let deploy_agent_row_count = snapshot.rows.len().to_string();
    let deploy_empty_row = if snapshot.rows.is_empty() {
        Some(view! {
            <tr id="tau-ops-deploy-process-empty-row" data-empty-state="true">
                <td colspan="8">No deployed agent process records yet.</td>
            </tr>
        })
    } else {
        None
    };
    let deploy_agent_rows_view = snapshot
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row_id = format!("tau-ops-deploy-process-row-{index}");
            let stop_form_id = format!("tau-ops-deploy-stop-form-{index}");
            let stop_button_id = format!("tau-ops-deploy-stop-button-{index}");
            let stop_theme_input_id = format!("tau-ops-deploy-stop-theme-{index}");
            let stop_sidebar_input_id = format!("tau-ops-deploy-stop-sidebar-{index}");
            let stop_session_input_id = format!("tau-ops-deploy-stop-session-{index}");
            let stop_agent_path_segment = encode_ops_path_segment(row.agent_id.as_str());
            let stop_action = format!("/ops/deploy/agents/{stop_agent_path_segment}/stop");
            let stop_disabled = row.process_status != "running";
            let stop_aria_disabled = if stop_disabled { "true" } else { "false" };
            let stop_theme = theme.to_string();
            let stop_sidebar = sidebar.to_string();
            let stop_session_key = session_key.to_string();
            let process_started = row.process_started_unix_ms.to_string();
            let process_stopped = row.process_stopped_unix_ms.to_string();
            let updated_unix_ms = row.updated_unix_ms.to_string();
            view! {
                <tr
                    id=row_id
                    data-agent-id=row.agent_id.clone()
                    data-agent-status=row.status.clone()
                    data-process-id=row.process_id.clone()
                    data-process-status=row.process_status.clone()
                    data-process-pid=row.process_pid.clone()
                    data-process-started-unix-ms=process_started
                    data-process-stopped-unix-ms=process_stopped
                    data-process-stop-reason=row.process_stop_reason.clone()
                    data-process-exit-status=row.process_exit_status.clone()
                    data-updated-unix-ms=updated_unix_ms
                >
                    <td>{row.agent_id.clone()}</td>
                    <td>{row.status.clone()}</td>
                    <td>{row.profile.clone()}</td>
                    <td>{row.model.clone()}</td>
                    <td>{row.process_status.clone()}</td>
                    <td>{row.process_pid.clone()}</td>
                    <td>{row.process_stop_reason.clone()}</td>
                    <td>
                        <form
                            id=stop_form_id
                            action=stop_action
                            method="post"
                            data-action="stop-agent"
                            data-agent-id=row.agent_id.clone()
                            data-process-id=row.process_id.clone()
                            data-process-status=row.process_status.clone()
                            data-preserves-shell-context="true"
                        >
                            <input id=stop_theme_input_id type="hidden" name="theme" value=stop_theme />
                            <input id=stop_sidebar_input_id type="hidden" name="sidebar" value=stop_sidebar />
                            <input id=stop_session_input_id type="hidden" name="session" value=stop_session_key />
                            <button
                                id=stop_button_id
                                type="submit"
                                data-action="stop-agent"
                                data-agent-id=row.agent_id.clone()
                                data-process-status=row.process_status.clone()
                                aria-disabled=stop_aria_disabled
                                disabled=stop_disabled
                            >
                                Stop
                            </button>
                        </form>
                    </td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <section
            id="tau-ops-deploy-processes"
            data-component="DeployProcessLifecycle"
            data-state-source=deploy_state_source
            data-state-status=deploy_state_status
            data-agent-count=deploy_agent_count
            data-running-count=deploy_running_count
            data-stopped-count=deploy_stopped_count
            data-row-count=deploy_agent_row_count
        >
            <h3>Process Lifecycle</h3>
            <table>
                <thead>
                    <tr>
                        <th scope="col">Agent</th>
                        <th scope="col">Agent Status</th>
                        <th scope="col">Profile</th>
                        <th scope="col">Model</th>
                        <th scope="col">Process Status</th>
                        <th scope="col">PID</th>
                        <th scope="col">Stop Reason</th>
                        <th scope="col">Action</th>
                    </tr>
                </thead>
                <tbody id="tau-ops-deploy-processes-body">
                    {deploy_agent_rows_view}
                    {deploy_empty_row}
                </tbody>
            </table>
        </section>
    }
}
