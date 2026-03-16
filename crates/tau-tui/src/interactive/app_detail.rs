use super::{App, DetailSection, FocusPanel};

impl App {
    pub(super) fn show_detail_section(&mut self, section: DetailSection) {
        self.detail_section = section;
        self.show_tool_panel = true;
    }

    pub(super) fn cycle_detail_section_forward(&mut self) {
        self.detail_section = match self.detail_section {
            DetailSection::Tools => DetailSection::Memory,
            DetailSection::Memory => DetailSection::Cortex,
            DetailSection::Cortex => DetailSection::Sessions,
            DetailSection::Sessions => DetailSection::Tools,
        };
        self.show_tool_panel = true;
        self.focus = FocusPanel::Tools;
    }

    pub(super) fn cycle_detail_section_backward(&mut self) {
        self.detail_section = match self.detail_section {
            DetailSection::Tools => DetailSection::Sessions,
            DetailSection::Memory => DetailSection::Tools,
            DetailSection::Cortex => DetailSection::Memory,
            DetailSection::Sessions => DetailSection::Cortex,
        };
        self.show_tool_panel = true;
        self.focus = FocusPanel::Tools;
    }
}
