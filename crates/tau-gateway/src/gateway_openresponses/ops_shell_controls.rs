use serde::Deserialize;
use tau_dashboard_ui::{TauOpsDashboardSidebarState, TauOpsDashboardTheme};

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct OpsShellControlsQuery {
    #[serde(default)]
    theme: String,
    #[serde(default)]
    sidebar: String,
    #[serde(default)]
    range: String,
}

impl OpsShellControlsQuery {
    pub(super) fn theme(&self) -> TauOpsDashboardTheme {
        match self.theme.as_str() {
            "light" => TauOpsDashboardTheme::Light,
            _ => TauOpsDashboardTheme::Dark,
        }
    }

    pub(super) fn sidebar_state(&self) -> TauOpsDashboardSidebarState {
        match self.sidebar.as_str() {
            "collapsed" => TauOpsDashboardSidebarState::Collapsed,
            _ => TauOpsDashboardSidebarState::Expanded,
        }
    }

    pub(super) fn timeline_range(&self) -> &'static str {
        match self.range.as_str() {
            "6h" => "6h",
            "24h" => "24h",
            _ => "1h",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpsShellControlsQuery;

    #[test]
    fn unit_timeline_range_returns_selected_supported_values() {
        let six_hours = OpsShellControlsQuery {
            range: "6h".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(six_hours.timeline_range(), "6h");

        let twenty_four_hours = OpsShellControlsQuery {
            range: "24h".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(twenty_four_hours.timeline_range(), "24h");
    }

    #[test]
    fn unit_timeline_range_defaults_to_one_hour_for_invalid_values() {
        let invalid = OpsShellControlsQuery {
            range: "quarter".to_string(),
            ..OpsShellControlsQuery::default()
        };
        assert_eq!(invalid.timeline_range(), "1h");

        let empty = OpsShellControlsQuery::default();
        assert_eq!(empty.timeline_range(), "1h");
    }
}
