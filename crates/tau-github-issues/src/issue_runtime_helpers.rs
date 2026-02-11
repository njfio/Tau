use std::path::{Path, PathBuf};

pub fn normalize_relative_channel_path(
    path: &Path,
    channel_root: &Path,
    label: &str,
) -> Result<String, String> {
    let relative = path.strip_prefix(channel_root).map_err(|_| {
        format!(
            "failed to derive relative path for {label}: {}",
            path.display()
        )
    })?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    if normalized.trim().is_empty() {
        return Err(format!(
            "derived empty relative path for {label}: {}",
            path.display()
        ));
    }
    Ok(normalized)
}

pub fn normalize_artifact_retention_days(days: u64) -> Option<u64> {
    if days == 0 {
        None
    } else {
        Some(days)
    }
}

pub fn render_issue_artifact_pointer_line(
    label: &str,
    artifact_id: &str,
    relative_path: &str,
    bytes: u64,
) -> String {
    format!("{label}: id=`{artifact_id}` path=`{relative_path}` bytes=`{bytes}`")
}

pub fn session_path_for_issue(repo_state_dir: &Path, issue_number: u64) -> PathBuf {
    repo_state_dir
        .join("sessions")
        .join(format!("issue-{issue_number}.jsonl"))
}

pub fn issue_session_id(issue_number: u64) -> String {
    format!("issue-{issue_number}")
}

#[cfg(test)]
mod tests {
    use super::{
        issue_session_id, normalize_artifact_retention_days, normalize_relative_channel_path,
        render_issue_artifact_pointer_line, session_path_for_issue,
    };
    use std::path::Path;

    #[test]
    fn unit_normalize_artifact_retention_days_maps_zero_to_none() {
        assert_eq!(normalize_artifact_retention_days(0), None);
        assert_eq!(normalize_artifact_retention_days(30), Some(30));
    }

    #[test]
    fn functional_normalize_relative_channel_path_normalizes_windows_separators() {
        let root = Path::new("/tmp/channel");
        let file = Path::new("/tmp/channel/attachments/nested\\trace.log");
        let normalized = normalize_relative_channel_path(file, root, "attachment")
            .expect("normalized relative path");
        assert_eq!(normalized, "attachments/nested/trace.log");
    }

    #[test]
    fn integration_render_issue_artifact_pointer_line_formats_expected_shape() {
        let rendered =
            render_issue_artifact_pointer_line("artifact", "id-1", "artifacts/run-1.md", 42);
        assert_eq!(
            rendered,
            "artifact: id=`id-1` path=`artifacts/run-1.md` bytes=`42`"
        );
    }

    #[test]
    fn regression_normalize_relative_channel_path_rejects_outside_and_empty_paths() {
        let root = Path::new("/tmp/channel");
        let outside = Path::new("/tmp/other/artifacts/run-1.md");
        let outside_error = normalize_relative_channel_path(outside, root, "artifact")
            .expect_err("outside path should fail");
        assert!(outside_error.contains("failed to derive relative path"));

        let root_path_error = normalize_relative_channel_path(root, root, "artifact")
            .expect_err("root path should fail");
        assert!(root_path_error.contains("derived empty relative path"));
    }

    #[test]
    fn unit_issue_session_id_formats_issue_identifier() {
        assert_eq!(issue_session_id(42), "issue-42");
    }

    #[test]
    fn integration_session_path_for_issue_builds_expected_repo_relative_path() {
        let root = Path::new("/tmp/repo");
        let path = session_path_for_issue(root, 9);
        assert_eq!(path, Path::new("/tmp/repo/sessions/issue-9.jsonl"));
    }
}
