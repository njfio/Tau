use super::FocusPanel;

pub(super) fn next_normal_focus(current: FocusPanel, show_tool_panel: bool) -> FocusPanel {
    match current {
        FocusPanel::Chat => FocusPanel::Input,
        FocusPanel::Input => {
            if show_tool_panel {
                FocusPanel::Tools
            } else {
                FocusPanel::Chat
            }
        }
        FocusPanel::Tools => FocusPanel::Chat,
        FocusPanel::CommandPalette => FocusPanel::Input,
    }
}

pub(super) fn next_insert_focus(current: FocusPanel, show_tool_panel: bool) -> FocusPanel {
    match current {
        FocusPanel::Input => FocusPanel::Chat,
        FocusPanel::Chat => {
            if show_tool_panel {
                FocusPanel::Tools
            } else {
                FocusPanel::Input
            }
        }
        FocusPanel::Tools => FocusPanel::Input,
        FocusPanel::CommandPalette => FocusPanel::Input,
    }
}
