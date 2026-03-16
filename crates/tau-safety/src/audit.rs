//! Safety audit logging for compliance and debugging.
//!
//! Records all safety check outcomes for post-hoc analysis.

use serde::Serialize;

use crate::{SafetyMode, SafetyStage};

/// A single safety audit log entry.
#[derive(Debug, Clone, Serialize)]
pub struct SafetyAuditEntry {
    pub timestamp_ms: u64,
    pub session_id: String,
    pub stage: SafetyStage,
    pub action: SafetyAction,
    pub rule_matched: Option<String>,
    pub content_snippet: String,
    pub severity: SafetySeverity,
}

/// The action taken by the safety system.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyAction {
    Allowed,
    Warned,
    Redacted,
    Blocked,
}

impl From<SafetyMode> for SafetyAction {
    fn from(mode: SafetyMode) -> Self {
        match mode {
            SafetyMode::Warn => SafetyAction::Warned,
            SafetyMode::Redact => SafetyAction::Redacted,
            SafetyMode::Block => SafetyAction::Blocked,
        }
    }
}

/// Severity level for audit entries.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetySeverity {
    Info,
    Warning,
    Critical,
}

/// In-memory safety audit log with bounded capacity.
pub struct SafetyAuditLog {
    entries: Vec<SafetyAuditEntry>,
    max_entries: usize,
}

impl SafetyAuditLog {
    /// Create a new audit log with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Append an audit entry, evicting the oldest if at capacity.
    pub fn record(&mut self, entry: SafetyAuditEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Returns all audit entries.
    pub fn entries(&self) -> &[SafetyAuditEntry] {
        &self.entries
    }

    /// Returns entries filtered by stage.
    pub fn entries_for_stage(&self, stage: SafetyStage) -> Vec<&SafetyAuditEntry> {
        self.entries.iter().filter(|e| e.stage == stage).collect()
    }

    /// Returns entries filtered by severity.
    pub fn entries_by_severity(&self, severity: &SafetySeverity) -> Vec<&SafetyAuditEntry> {
        self.entries
            .iter()
            .filter(|e| &e.severity == severity)
            .collect()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no entries recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// PII detection patterns and redaction.
pub struct PiiDetector {
    patterns: Vec<PiiPattern>,
}

/// A single PII detection pattern.
pub struct PiiPattern {
    pub category: PiiCategory,
    pub regex: regex::Regex,
    pub redaction_token: String,
}

/// Categories of PII that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiCategory {
    Email,
    PhoneNumber,
    SocialSecurityNumber,
    CreditCard,
    IpAddress,
}

/// A single PII detection result.
#[derive(Debug, Clone)]
pub struct PiiDetection {
    pub category: PiiCategory,
    pub start: usize,
    pub end: usize,
}

impl PiiDetector {
    /// Create a detector with default PII patterns.
    pub fn default_patterns() -> Self {
        Self {
            patterns: vec![
                PiiPattern {
                    category: PiiCategory::Email,
                    regex: regex::Regex::new(
                        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
                    )
                    .expect("valid email regex"),
                    redaction_token: "[EMAIL_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::PhoneNumber,
                    regex: regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b")
                        .expect("valid phone regex"),
                    redaction_token: "[PHONE_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::SocialSecurityNumber,
                    regex: regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
                        .expect("valid SSN regex"),
                    redaction_token: "[SSN_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::CreditCard,
                    regex: regex::Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b")
                        .expect("valid CC regex"),
                    redaction_token: "[CC_REDACTED]".to_string(),
                },
                PiiPattern {
                    category: PiiCategory::IpAddress,
                    regex: regex::Regex::new(
                        r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b",
                    )
                    .expect("valid IP regex"),
                    redaction_token: "[IP_REDACTED]".to_string(),
                },
            ],
        }
    }

    /// Scan text for PII and return redacted version plus detections.
    pub fn scan_and_redact(&self, text: &str) -> (String, Vec<PiiDetection>) {
        let mut detections = Vec::new();
        let mut result = text.to_string();

        for pattern in &self.patterns {
            let ranges: Vec<(usize, usize)> = pattern
                .regex
                .find_iter(&result)
                .map(|m| (m.start(), m.end()))
                .collect();
            for &(start, end) in ranges.iter().rev() {
                detections.push(PiiDetection {
                    category: pattern.category,
                    start,
                    end,
                });
                result.replace_range(start..end, &pattern.redaction_token);
            }
        }

        (result, detections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_log_capacity() {
        let mut log = SafetyAuditLog::new(3);
        for i in 0..5 {
            log.record(SafetyAuditEntry {
                timestamp_ms: i,
                session_id: "s1".to_string(),
                stage: SafetyStage::InboundMessage,
                action: SafetyAction::Allowed,
                rule_matched: None,
                content_snippet: format!("entry {}", i),
                severity: SafetySeverity::Info,
            });
        }
        assert_eq!(log.len(), 3);
        assert_eq!(log.entries()[0].timestamp_ms, 2);
    }

    #[test]
    fn pii_email_redaction() {
        let detector = PiiDetector::default_patterns();
        let (redacted, detections) = detector.scan_and_redact("Email me at user@example.com ok?");
        assert!(redacted.contains("[EMAIL_REDACTED]"));
        assert!(!redacted.contains("user@example.com"));
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].category, PiiCategory::Email);
    }

    #[test]
    fn pii_ssn_redaction() {
        let detector = PiiDetector::default_patterns();
        let (redacted, detections) = detector.scan_and_redact("SSN is 123-45-6789");
        assert!(redacted.contains("[SSN_REDACTED]"));
        assert!(!redacted.contains("123-45-6789"));
        assert_eq!(detections.len(), 1);
    }

    #[test]
    fn pii_no_false_positives_on_clean_text() {
        let detector = PiiDetector::default_patterns();
        let (redacted, detections) = detector.scan_and_redact("Hello, this is clean text.");
        assert_eq!(redacted, "Hello, this is clean text.");
        assert!(detections.is_empty());
    }

    #[test]
    fn safety_action_from_mode() {
        assert_eq!(SafetyAction::from(SafetyMode::Warn), SafetyAction::Warned);
        assert_eq!(
            SafetyAction::from(SafetyMode::Redact),
            SafetyAction::Redacted
        );
        assert_eq!(SafetyAction::from(SafetyMode::Block), SafetyAction::Blocked);
    }
}
