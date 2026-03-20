use super::super::{app::App, tools::ToolStatus};

const ACTION_TERMS: &[&str] = &[
    "create",
    "build",
    "implement",
    "scaffold",
    "make",
    "develop",
    "fix",
    "add",
    "edit",
    "update",
    "refactor",
];

const TARGET_TERMS: &[&str] = &[
    "game",
    "app",
    "application",
    "site",
    "website",
    "page",
    "component",
    "feature",
    "ui",
    "scene",
    "phaser",
    "phaserjs",
    "project",
    "prototype",
    "script",
];

const MUTATING_TOOL_NAMES: &[&str] = &["write", "edit"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildEvidenceState {
    NoMutatingEvidenceYet,
    ReadOnlySoFar,
    MutatingEvidenceConfirmed,
}

impl BuildEvidenceState {
    pub(super) fn activity_text(self) -> &'static str {
        match self {
            Self::NoMutatingEvidenceYet => "no mutating evidence yet",
            Self::ReadOnlySoFar => "read-only so far",
            Self::MutatingEvidenceConfirmed => "mutating evidence confirmed",
        }
    }

    pub(super) fn run_state_text(self) -> &'static str {
        match self {
            Self::NoMutatingEvidenceYet => "no mutating evidence yet",
            Self::ReadOnlySoFar => "still read-only",
            Self::MutatingEvidenceConfirmed => "write/edit confirmed",
        }
    }
}

pub(super) fn build_evidence_state(app: &App) -> Option<BuildEvidenceState> {
    let prompt = app.last_submitted_input.as_deref()?;
    if !prompt_requests_build_or_create(prompt) {
        return None;
    }
    Some(tool_evidence_state(app))
}

fn prompt_requests_build_or_create(prompt: &str) -> bool {
    let normalized = prompt.to_lowercase();
    contains_any(&normalized, ACTION_TERMS) && contains_any(&normalized, TARGET_TERMS)
}

fn contains_any(input: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| input.contains(term))
}

fn tool_evidence_state(app: &App) -> BuildEvidenceState {
    let mut has_successful_tool = false;
    for entry in app.tools.entries() {
        if entry.status != ToolStatus::Success {
            continue;
        }
        has_successful_tool = true;
        if MUTATING_TOOL_NAMES.contains(&entry.name.as_str()) {
            return BuildEvidenceState::MutatingEvidenceConfirmed;
        }
    }
    if has_successful_tool {
        BuildEvidenceState::ReadOnlySoFar
    } else {
        BuildEvidenceState::NoMutatingEvidenceYet
    }
}
