//! Legacy CLI alias normalization helpers for M22 prompt-optimization rename.

const LEGACY_ALIAS_MAP: [(&str, &str); 8] = [
    ("--train-config", "--prompt-optimization-config"),
    ("--train-store-sqlite", "--prompt-optimization-store-sqlite"),
    ("--train-json", "--prompt-optimization-json"),
    (
        "--training-proxy-server",
        "--prompt-optimization-proxy-server",
    ),
    ("--training-proxy-bind", "--prompt-optimization-proxy-bind"),
    (
        "--training-proxy-upstream-url",
        "--prompt-optimization-proxy-upstream-url",
    ),
    (
        "--training-proxy-state-dir",
        "--prompt-optimization-proxy-state-dir",
    ),
    (
        "--training-proxy-timeout-ms",
        "--prompt-optimization-proxy-timeout-ms",
    ),
];

fn warning_for_alias(legacy: &str, canonical: &str) -> String {
    format!(
        "deprecated CLI alias '{}' detected; use '{}' instead.",
        legacy, canonical
    )
}

/// Normalize legacy prompt-optimization alias flags and return warning strings.
pub fn normalize_legacy_training_aliases(args: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut normalized = Vec::with_capacity(args.len());
    let mut warnings = Vec::new();

    for arg in args {
        let mut replacement: Option<String> = None;
        let mut warning: Option<String> = None;

        for (legacy, canonical) in LEGACY_ALIAS_MAP {
            if arg == legacy {
                replacement = Some(canonical.to_string());
                warning = Some(warning_for_alias(legacy, canonical));
                break;
            }
            if let Some(value) = arg.strip_prefix(&format!("{legacy}=")) {
                replacement = Some(format!("{canonical}={value}"));
                warning = Some(warning_for_alias(legacy, canonical));
                break;
            }
        }

        if let Some(updated) = replacement {
            normalized.push(updated);
        } else {
            normalized.push(arg);
        }
        if let Some(message) = warning {
            warnings.push(message);
        }
    }

    (normalized, warnings)
}

#[cfg(test)]
mod tests {
    use super::normalize_legacy_training_aliases;

    #[test]
    fn unit_normalize_legacy_training_aliases_maps_flags_and_emits_stable_warnings() {
        let (normalized, warnings) = normalize_legacy_training_aliases(vec![
            "tau-rs".to_string(),
            "--train-config".to_string(),
            ".tau/train.json".to_string(),
            "--training-proxy-server".to_string(),
            "--training-proxy-bind=127.0.0.1:9988".to_string(),
        ]);

        assert_eq!(
            normalized,
            vec![
                "tau-rs",
                "--prompt-optimization-config",
                ".tau/train.json",
                "--prompt-optimization-proxy-server",
                "--prompt-optimization-proxy-bind=127.0.0.1:9988",
            ]
        );
        assert_eq!(
            warnings,
            vec![
                String::from(
                    "deprecated CLI alias '--train-config' detected; use '--prompt-optimization-config' instead."
                ),
                String::from(
                    "deprecated CLI alias '--training-proxy-server' detected; use '--prompt-optimization-proxy-server' instead."
                ),
                String::from(
                    "deprecated CLI alias '--training-proxy-bind' detected; use '--prompt-optimization-proxy-bind' instead."
                ),
            ]
        );
    }
}
