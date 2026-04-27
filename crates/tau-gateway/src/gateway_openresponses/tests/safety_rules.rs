use tau_agent_core::{default_safety_rule_set, SafetyRuleMatcher};

/// Build a rules JSON payload that includes all default rules plus optional custom additions.
/// This satisfies the immutable safety floor which requires all default rules to be present.
pub(super) fn safety_rules_json_with_defaults(
    extra_prompt_injection: &[serde_json::Value],
    extra_secret_leak: &[serde_json::Value],
) -> serde_json::Value {
    let defaults = default_safety_rule_set();
    let mut prompt_rules: Vec<serde_json::Value> = defaults
        .prompt_injection_rules
        .iter()
        .map(|r| {
            serde_json::json!({
                "rule_id": r.rule_id,
                "reason_code": r.reason_code,
                "pattern": r.pattern,
                "matcher": match r.matcher {
                    SafetyRuleMatcher::Literal => "literal",
                    SafetyRuleMatcher::Regex => "regex",
                },
                "enabled": r.enabled
            })
        })
        .collect();
    prompt_rules.extend_from_slice(extra_prompt_injection);
    let mut secret_rules: Vec<serde_json::Value> = defaults
        .secret_leak_rules
        .iter()
        .map(|r| {
            serde_json::json!({
                "rule_id": r.rule_id,
                "reason_code": r.reason_code,
                "pattern": r.pattern,
                "matcher": match r.matcher {
                    SafetyRuleMatcher::Literal => "literal",
                    SafetyRuleMatcher::Regex => "regex",
                },
                "enabled": r.enabled
            })
        })
        .collect();
    secret_rules.extend_from_slice(extra_secret_leak);
    serde_json::json!({
        "rules": {
            "prompt_injection_rules": prompt_rules,
            "secret_leak_rules": secret_rules
        }
    })
}
