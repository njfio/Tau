//! Verification gates for milestone readiness.
//!
//! Each gate runs a set of checks (cargo tests, artifact presence, grep markers)
//! and produces a [`GateResult`] with pass/fail status, duration, and failure
//! details. This module is the Rust equivalent of the `scripts/verify/m*` shell
//! scripts, providing deterministic, composable verification that can be invoked
//! via `tau ops verify`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Outcome of a single verification step within a gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepResult {
    pub id: String,
    pub status: StepStatus,
    pub command: String,
    pub log: Option<String>,
}

/// Status of a verification step.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pass,
    Fail,
    Skip,
}

impl fmt::Display for StepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepStatus::Pass => write!(f, "pass"),
            StepStatus::Fail => write!(f, "fail"),
            StepStatus::Skip => write!(f, "skip"),
        }
    }
}

/// Result of running a complete verification gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub failures: Vec<String>,
    pub steps: Vec<StepResult>,
}

/// Aggregated report across all gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub schema_version: u32,
    pub suite_id: String,
    pub overall: String,
    pub gates: Vec<GateResult>,
}

// ---------------------------------------------------------------------------
// Step runner
// ---------------------------------------------------------------------------

/// Run a single shell command as a verification step, capturing output.
fn run_step(id: &str, command: &str, root_dir: &Path) -> StepResult {
    let result = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(root_dir)
        .output();

    match result {
        Ok(output) => {
            let status = if output.status.success() {
                StepStatus::Pass
            } else if output.status.code() == Some(20) {
                StepStatus::Skip
            } else {
                StepStatus::Fail
            };
            let stderr = String::from_utf8_lossy(&output.stderr);
            let log = if stderr.is_empty() {
                None
            } else {
                Some(stderr.into_owned())
            };
            StepResult {
                id: id.to_string(),
                status,
                command: command.to_string(),
                log,
            }
        }
        Err(e) => StepResult {
            id: id.to_string(),
            status: StepStatus::Fail,
            command: command.to_string(),
            log: Some(format!("failed to execute command: {e}")),
        },
    }
}

/// Run a cargo test as a verification step.
fn cargo_test_step(id: &str, package: &str, test_filter: &str, root_dir: &Path) -> StepResult {
    let cmd = format!("cargo test -p {package} {test_filter} -- --nocapture 2>&1");
    run_step(id, &cmd, root_dir)
}

/// Check that a file exists.
#[allow(dead_code)]
fn file_exists_step(id: &str, path: &Path, root_dir: &Path) -> StepResult {
    let full_path = root_dir.join(path);
    let status = if full_path.exists() {
        StepStatus::Pass
    } else {
        StepStatus::Fail
    };
    StepResult {
        id: id.to_string(),
        status,
        command: format!("test -f {}", path.display()),
        log: if status == StepStatus::Fail {
            Some(format!("file not found: {}", full_path.display()))
        } else {
            None
        },
    }
}

/// Check that a pattern appears in a file via grep.
#[allow(dead_code)]
fn grep_step(id: &str, pattern: &str, file: &Path, root_dir: &Path) -> StepResult {
    let cmd = format!("grep -qn '{}' '{}'", pattern, file.display());
    run_step(id, &cmd, root_dir)
}

// ---------------------------------------------------------------------------
// Gate builder
// ---------------------------------------------------------------------------

/// Builder for constructing a gate from a sequence of steps.
struct GateBuilder {
    name: String,
    steps: Vec<StepResult>,
    start: Instant,
}

impl GateBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
            start: Instant::now(),
        }
    }

    fn add_step(&mut self, step: StepResult) {
        self.steps.push(step);
    }

    fn finish(self) -> GateResult {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        let failures: Vec<String> = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Fail)
            .map(|s| s.id.clone())
            .collect();
        let passed = failures.is_empty();
        GateResult {
            name: self.name,
            passed,
            duration_ms,
            failures,
            steps: self.steps,
        }
    }
}

// ---------------------------------------------------------------------------
// Concrete gate implementations
// ---------------------------------------------------------------------------

/// Registry entry for a named gate.
pub struct GateEntry {
    pub id: &'static str,
    pub description: &'static str,
    pub run: fn(&Path) -> GateResult,
}

/// All registered verification gates.
pub static GATE_REGISTRY: &[GateEntry] = &[
    GateEntry {
        id: "m295_operator_maturity_wave",
        description: "Core operator maturity wave (TUI, RL, auth conformance)",
        run: gate_m295_operator_maturity_wave,
    },
    GateEntry {
        id: "m303_auth_workflow_depth",
        description: "Auth workflow depth (provider + gateway auth lifecycle)",
        run: gate_m303_auth_workflow_depth,
    },
    GateEntry {
        id: "m310_rl_policy_ops_depth",
        description: "RL policy operations depth (promotion, rollback, significance)",
        run: gate_m310_rl_policy_ops_depth,
    },
    GateEntry {
        id: "m313_e2e_core_scenario_depth",
        description: "E2E core scenario depth (integration + gateway roundtrips)",
        run: gate_m313_e2e_core_scenario_depth,
    },
    GateEntry {
        id: "workspace_compile_check",
        description: "Verify entire workspace compiles without errors",
        run: gate_workspace_compile_check,
    },
];

/// M295 — Operator maturity wave: TUI shell conformance, RL e2e, auth matrices.
pub fn gate_m295_operator_maturity_wave(root_dir: &Path) -> GateResult {
    let mut g = GateBuilder::new("m295_operator_maturity_wave");

    g.add_step(cargo_test_step(
        "tui_shell_conformance",
        "tau-tui",
        "spec_c01_shell_renderer_includes_all_operator_panels",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "rl_e2e_conformance",
        "tau-trainer",
        "--test rl_e2e_harness",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "auth_provider_matrix",
        "tau-provider",
        "--test auth_workflow_conformance",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "auth_coding_agent_matrix",
        "tau-coding-agent",
        "auth_provider::auth_and_provider::spec_c04_auth_matrix_accepts_openrouter_provider_filter",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "auth_gateway_session_lifecycle",
        "tau-gateway",
        "conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts",
        root_dir,
    ));

    g.finish()
}

/// M303 — Auth workflow depth.
pub fn gate_m303_auth_workflow_depth(root_dir: &Path) -> GateResult {
    let mut g = GateBuilder::new("m303_auth_workflow_depth");

    g.add_step(cargo_test_step(
        "provider_auth_workflow_conformance",
        "tau-provider",
        "--test auth_workflow_conformance",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_bootstrap_token_mode_contract",
        "tau-gateway",
        "functional_spec_2786_c01_gateway_auth_bootstrap_endpoint_reports_token_mode_contract",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_bootstrap_localhost_dev_mode_contract",
        "tau-gateway",
        "functional_spec_2786_c02_gateway_auth_bootstrap_maps_localhost_dev_to_none_mode",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_auth_session_lifecycle_conformance",
        "tau-gateway",
        "conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_localhost_dev_allows_no_bearer",
        "tau-gateway",
        "integration_localhost_dev_mode_allows_requests_without_bearer_token",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_invalid_password_rejected",
        "tau-gateway",
        "regression_gateway_auth_session_rejects_invalid_password",
        root_dir,
    ));

    g.finish()
}

/// M310 — RL policy operations depth.
pub fn gate_m310_rl_policy_ops_depth(root_dir: &Path) -> GateResult {
    let mut g = GateBuilder::new("m310_rl_policy_ops_depth");

    g.add_step(cargo_test_step(
        "rl_harness_deterministic_artifact_conformance",
        "tau-trainer",
        "spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "rl_significance_policy_gain_conformance",
        "tau-trainer",
        "spec_c02_policy_comparison_reports_significant_improvement",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "rl_promotion_significance_stability_integration",
        "tau-trainer",
        "integration_checkpoint_promotion_gate_requires_significance_stability",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "rl_rollback_required_gate_integration",
        "tau-trainer",
        "spec_c05_rl_e2e_rollback_gate_requires_rollback_when_gate_signals_failures",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "rl_rollback_denied_without_failures_regression",
        "tau-trainer",
        "spec_c06_rl_e2e_rollback_gate_blocks_when_promotion_denied_without_failed_checks",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "runtime_promotion_gate_audit_regression",
        "tau-runtime",
        "regression_checkpoint_promotion_gate_audit_json_reports_blocked_threshold_reason",
        root_dir,
    ));

    g.finish()
}

/// M313 — E2E core scenario depth.
pub fn gate_m313_e2e_core_scenario_depth(root_dir: &Path) -> GateResult {
    let mut g = GateBuilder::new("m313_e2e_core_scenario_depth");

    g.add_step(cargo_test_step(
        "integration_workspace_runs_package",
        "tau-integration-tests",
        "integration_spec_2608_c01_workspace_runs_integration_package",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "integration_agent_tool_memory_roundtrip_conformance",
        "tau-integration-tests",
        "conformance_spec_2608_c02_agent_tool_memory_roundtrip",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "integration_isolated_memory_state_regression",
        "tau-integration-tests",
        "regression_spec_2608_c03_harness_uses_isolated_memory_state",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_e2e_harness_lifecycle_session_flow",
        "tau-gateway",
        "integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow",
        root_dir,
    ));
    g.add_step(cargo_test_step(
        "gateway_openresponses_http_roundtrip_session_persistence",
        "tau-gateway",
        "integration_openresponses_http_roundtrip_persists_session_state",
        root_dir,
    ));

    g.finish()
}

/// Workspace compile check — verifies `cargo check --workspace` succeeds.
pub fn gate_workspace_compile_check(root_dir: &Path) -> GateResult {
    let mut g = GateBuilder::new("workspace_compile_check");
    g.add_step(run_step(
        "cargo_check_workspace",
        "cargo check --workspace 2>&1",
        root_dir,
    ));
    g.finish()
}

// ---------------------------------------------------------------------------
// Gate runner
// ---------------------------------------------------------------------------

/// Run all registered verification gates and return a full report.
pub fn run_all_gates(root_dir: &Path) -> VerificationReport {
    let gates: Vec<GateResult> = GATE_REGISTRY
        .iter()
        .map(|entry| (entry.run)(root_dir))
        .collect();
    let overall = if gates.iter().all(|g| g.passed) {
        "pass"
    } else {
        "fail"
    };
    VerificationReport {
        schema_version: 1,
        suite_id: "tau_ops_verify_all".to_string(),
        overall: overall.to_string(),
        gates,
    }
}

/// Run a single gate by id.
pub fn run_gate(gate_id: &str, root_dir: &Path) -> Option<GateResult> {
    GATE_REGISTRY
        .iter()
        .find(|entry| entry.id == gate_id)
        .map(|entry| (entry.run)(root_dir))
}

/// List all available gate ids and descriptions.
pub fn list_gates() -> Vec<(&'static str, &'static str)> {
    GATE_REGISTRY
        .iter()
        .map(|entry| (entry.id, entry.description))
        .collect()
}

// ---------------------------------------------------------------------------
// Report rendering
// ---------------------------------------------------------------------------

/// Render a human-readable report from gate results.
pub fn render_gate_report(results: &[GateResult]) -> String {
    let mut out = String::new();
    out.push_str("=== Verification Gate Report ===\n\n");

    let total = results.len();
    let passed = results.iter().filter(|g| g.passed).count();
    let failed = total - passed;

    for gate in results {
        let status_marker = if gate.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "[{status_marker}] {} ({} ms)\n",
            gate.name, gate.duration_ms
        ));
        for step in &gate.steps {
            let step_marker = match step.status {
                StepStatus::Pass => " PASS",
                StepStatus::Fail => " FAIL",
                StepStatus::Skip => " SKIP",
            };
            out.push_str(&format!("  [{step_marker}] {}\n", step.id));
        }
        if !gate.failures.is_empty() {
            out.push_str(&format!("  Failures: {}\n", gate.failures.join(", ")));
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "Summary: {passed}/{total} gates passed, {failed} failed\n"
    ));

    let overall = if failed == 0 { "PASS" } else { "FAIL" };
    out.push_str(&format!("Overall: {overall}\n"));

    out
}

/// Render a gate report as JSON.
pub fn render_gate_report_json(results: &[GateResult]) -> Result<String, serde_json::Error> {
    let overall = if results.iter().all(|g| g.passed) {
        "pass"
    } else {
        "fail"
    };
    let report = VerificationReport {
        schema_version: 1,
        suite_id: "tau_ops_verify".to_string(),
        overall: overall.to_string(),
        gates: results.to_vec(),
    };
    serde_json::to_string_pretty(&report)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_gate_result_serialization_roundtrip() {
        let gate = GateResult {
            name: "test_gate".to_string(),
            passed: true,
            duration_ms: 42,
            failures: vec![],
            steps: vec![
                StepResult {
                    id: "step_1".to_string(),
                    status: StepStatus::Pass,
                    command: "echo hello".to_string(),
                    log: None,
                },
                StepResult {
                    id: "step_2".to_string(),
                    status: StepStatus::Fail,
                    command: "false".to_string(),
                    log: Some("command failed".to_string()),
                },
            ],
        };

        let json = serde_json::to_string_pretty(&gate).expect("serialize");
        let deserialized: GateResult = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, "test_gate");
        assert!(deserialized.passed);
        assert_eq!(deserialized.duration_ms, 42);
        assert_eq!(deserialized.steps.len(), 2);
        assert_eq!(deserialized.steps[0].status, StepStatus::Pass);
        assert_eq!(deserialized.steps[1].status, StepStatus::Fail);
        assert_eq!(deserialized.steps[1].log.as_deref(), Some("command failed"));
    }

    #[test]
    fn unit_step_status_display() {
        assert_eq!(StepStatus::Pass.to_string(), "pass");
        assert_eq!(StepStatus::Fail.to_string(), "fail");
        assert_eq!(StepStatus::Skip.to_string(), "skip");
    }

    #[test]
    fn unit_render_gate_report_produces_readable_output() {
        let results = vec![
            GateResult {
                name: "alpha_gate".to_string(),
                passed: true,
                duration_ms: 100,
                failures: vec![],
                steps: vec![StepResult {
                    id: "step_a".to_string(),
                    status: StepStatus::Pass,
                    command: "cargo test".to_string(),
                    log: None,
                }],
            },
            GateResult {
                name: "beta_gate".to_string(),
                passed: false,
                duration_ms: 250,
                failures: vec!["step_b".to_string()],
                steps: vec![
                    StepResult {
                        id: "step_b".to_string(),
                        status: StepStatus::Fail,
                        command: "cargo test -p broken".to_string(),
                        log: Some("compilation error".to_string()),
                    },
                    StepResult {
                        id: "step_c".to_string(),
                        status: StepStatus::Pass,
                        command: "cargo check".to_string(),
                        log: None,
                    },
                ],
            },
        ];

        let report = render_gate_report(&results);

        assert!(report.contains("=== Verification Gate Report ==="));
        assert!(report.contains("[PASS] alpha_gate"));
        assert!(report.contains("[FAIL] beta_gate"));
        assert!(report.contains("step_b"));
        assert!(report.contains("1/2 gates passed, 1 failed"));
        assert!(report.contains("Overall: FAIL"));
    }

    #[test]
    fn unit_render_gate_report_all_pass() {
        let results = vec![GateResult {
            name: "only_gate".to_string(),
            passed: true,
            duration_ms: 10,
            failures: vec![],
            steps: vec![],
        }];
        let report = render_gate_report(&results);
        assert!(report.contains("Overall: PASS"));
        assert!(report.contains("1/1 gates passed, 0 failed"));
    }

    #[test]
    fn unit_render_gate_report_json_structure() {
        let results = vec![GateResult {
            name: "json_gate".to_string(),
            passed: false,
            duration_ms: 77,
            failures: vec!["oops".to_string()],
            steps: vec![StepResult {
                id: "oops".to_string(),
                status: StepStatus::Fail,
                command: "exit 1".to_string(),
                log: None,
            }],
        }];

        let json = render_gate_report_json(&results).expect("json render");
        let report: VerificationReport = serde_json::from_str(&json).expect("parse json report");

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.overall, "fail");
        assert_eq!(report.gates.len(), 1);
        assert_eq!(report.gates[0].name, "json_gate");
    }

    #[test]
    fn unit_verification_report_serialization_roundtrip() {
        let report = VerificationReport {
            schema_version: 1,
            suite_id: "test_suite".to_string(),
            overall: "pass".to_string(),
            gates: vec![],
        };
        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: VerificationReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.suite_id, "test_suite");
        assert_eq!(parsed.overall, "pass");
    }

    #[test]
    fn unit_gate_builder_collects_failures() {
        let mut g = GateBuilder::new("test");
        g.add_step(StepResult {
            id: "ok".to_string(),
            status: StepStatus::Pass,
            command: "true".to_string(),
            log: None,
        });
        g.add_step(StepResult {
            id: "broken".to_string(),
            status: StepStatus::Fail,
            command: "false".to_string(),
            log: Some("err".to_string()),
        });
        g.add_step(StepResult {
            id: "skipped".to_string(),
            status: StepStatus::Skip,
            command: "exit 20".to_string(),
            log: None,
        });
        let result = g.finish();
        assert!(!result.passed);
        assert_eq!(result.failures, vec!["broken"]);
        assert_eq!(result.steps.len(), 3);
    }

    #[test]
    fn unit_list_gates_returns_registered_entries() {
        let gates = list_gates();
        assert!(!gates.is_empty());
        let ids: Vec<&str> = gates.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&"m295_operator_maturity_wave"));
        assert!(ids.contains(&"workspace_compile_check"));
        assert!(ids.contains(&"m313_e2e_core_scenario_depth"));
    }

    #[test]
    fn functional_workspace_compile_check_gate_runs() {
        // This test actually runs `cargo check --workspace` in the repo root.
        // It validates that the gate runner infrastructure works end-to-end.
        let root_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("tau-ops parent")
            .parent()
            .expect("repo root")
            .to_path_buf();

        // We only run this gate if we're in the actual repo (not a temp dir).
        if !root_dir.join("Cargo.toml").exists() {
            return;
        }

        let result = gate_workspace_compile_check(&root_dir);
        // We check the structure, not necessarily that it passed (CI env may differ).
        assert_eq!(result.name, "workspace_compile_check");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].id, "cargo_check_workspace");
    }
}
