pub(super) fn resolve_session_endpoint(template: &str, session_id: &str) -> String {
    template.replace("{session_id}", session_id)
}

pub(super) fn resolve_job_endpoint(template: &str, job_id: &str) -> String {
    template.replace("{job_id}", job_id)
}

pub(super) fn resolve_agent_stop_endpoint(template: &str, agent_id: &str) -> String {
    template.replace("{agent_id}", agent_id)
}

pub(super) fn expand_session_template(template: &str, session_key: &str) -> String {
    template.replace("{session_key}", session_key)
}

pub(super) fn expand_mission_template(template: &str, mission_id: &str) -> String {
    template.replace("{mission_id}", mission_id)
}

pub(super) fn expand_memory_entry_template(
    template: &str,
    session_key: &str,
    entry_id: &str,
) -> String {
    template
        .replace("{session_key}", session_key)
        .replace("{entry_id}", entry_id)
}

pub(super) fn expand_channel_template(template: &str, channel: &str) -> String {
    template.replace("{channel}", channel)
}
