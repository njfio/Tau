use std::path::PathBuf;

use anyhow::Result;
use kamn_sdk::{
    initialize_browser_did, render_browser_did_init_report, BrowserDidInitRequest, DidMethod,
};
use serde::Serialize;

#[derive(Debug, Clone)]
/// Public struct `DeploymentWasmBrowserDidConfig` used across Tau components.
pub struct DeploymentWasmBrowserDidConfig {
    pub method: DidMethod,
    pub network: String,
    pub subject: String,
    pub entropy: String,
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
/// Public struct `DeploymentWasmBrowserDidReport` used across Tau components.
pub struct DeploymentWasmBrowserDidReport {
    pub runtime: String,
    pub interoperability_profile: String,
    pub method: String,
    pub network: String,
    pub subject: String,
    pub did: String,
    pub key_id: String,
    pub fingerprint: String,
    pub output_path: Option<String>,
    pub persisted: bool,
}

pub fn initialize_deployment_wasm_browser_did(
    config: &DeploymentWasmBrowserDidConfig,
) -> Result<DeploymentWasmBrowserDidReport> {
    let init = initialize_browser_did(&BrowserDidInitRequest {
        method: config.method,
        network: config.network.clone(),
        subject: config.subject.clone(),
        entropy: config.entropy.clone(),
    })?;

    #[cfg(not(target_arch = "wasm32"))]
    let mut persisted = false;
    #[cfg(target_arch = "wasm32")]
    let persisted = false;

    if let Some(path) = &config.output_path {
        #[cfg(not(target_arch = "wasm32"))]
        {
            kamn_sdk::write_browser_did_init_report(path, &init)?;
            persisted = true;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
        }
    }

    Ok(DeploymentWasmBrowserDidReport {
        runtime: init.runtime,
        interoperability_profile: init.interoperability_profile,
        method: init.identity.method.as_str().to_string(),
        network: init.identity.network,
        subject: init.identity.subject,
        did: init.identity.did,
        key_id: init.identity.key_id,
        fingerprint: init.identity.fingerprint,
        output_path: config
            .output_path
            .as_ref()
            .map(|path| path.display().to_string()),
        persisted,
    })
}

pub fn render_deployment_wasm_browser_did_report(
    report: &DeploymentWasmBrowserDidReport,
) -> String {
    format!(
        "deployment wasm browser did: runtime={} profile={} method={} did={} key_id={} fingerprint={} output_path={} persisted={}",
        report.runtime,
        report.interoperability_profile,
        report.method,
        report.did,
        report.key_id,
        report.fingerprint,
        report.output_path.as_deref().unwrap_or("none"),
        report.persisted
    )
}

pub fn render_deployment_wasm_browser_did_summary(
    config: &DeploymentWasmBrowserDidConfig,
) -> Result<String> {
    let init = initialize_browser_did(&BrowserDidInitRequest {
        method: config.method,
        network: config.network.clone(),
        subject: config.subject.clone(),
        entropy: config.entropy.clone(),
    })?;
    Ok(render_browser_did_init_report(&init))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        initialize_deployment_wasm_browser_did, render_deployment_wasm_browser_did_report,
        DeploymentWasmBrowserDidConfig,
    };
    use kamn_sdk::DidMethod;

    #[test]
    fn unit_initialize_deployment_wasm_browser_did_generates_identity() {
        let report = initialize_deployment_wasm_browser_did(&DeploymentWasmBrowserDidConfig {
            method: DidMethod::Key,
            network: "tau-devnet".to_string(),
            subject: "agent".to_string(),
            entropy: "seed-01".to_string(),
            output_path: None,
        })
        .expect("initialize browser did");

        assert_eq!(report.method, "key");
        assert!(report.did.starts_with("did:key:"));
        assert!(!report.persisted);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn functional_initialize_deployment_wasm_browser_did_writes_output_file() {
        let temp = tempdir().expect("tempdir");
        let output = temp.path().join("browser-did.json");
        let report = initialize_deployment_wasm_browser_did(&DeploymentWasmBrowserDidConfig {
            method: DidMethod::Web,
            network: "edge.tau.local".to_string(),
            subject: "operator".to_string(),
            entropy: "seed-02".to_string(),
            output_path: Some(output.clone()),
        })
        .expect("initialize browser did");

        assert!(report.persisted);
        assert!(output.exists());
    }

    #[test]
    fn integration_render_deployment_wasm_browser_did_report_contains_core_fields() {
        let report = initialize_deployment_wasm_browser_did(&DeploymentWasmBrowserDidConfig {
            method: DidMethod::Key,
            network: "tau-devnet".to_string(),
            subject: "agent".to_string(),
            entropy: "seed-03".to_string(),
            output_path: None,
        })
        .expect("initialize browser did");
        let rendered = render_deployment_wasm_browser_did_report(&report);

        assert!(rendered.contains("deployment wasm browser did"));
        assert!(rendered.contains("method=key"));
        assert!(rendered.contains("fingerprint="));
    }

    #[test]
    fn regression_initialize_deployment_wasm_browser_did_rejects_invalid_subject() {
        let error = initialize_deployment_wasm_browser_did(&DeploymentWasmBrowserDidConfig {
            method: DidMethod::Key,
            network: "tau-devnet".to_string(),
            subject: "bad subject".to_string(),
            entropy: "seed-04".to_string(),
            output_path: None,
        })
        .expect_err("invalid subject should fail");
        assert!(error
            .to_string()
            .contains("subject contains unsupported characters"));
    }
}
