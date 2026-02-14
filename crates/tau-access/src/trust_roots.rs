use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use tau_core::write_text_atomic;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public struct `TrustedKey` used across Tau components.
pub struct TrustedKey {
    pub id: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Public struct `TrustedRootRecord` used across Tau components.
pub struct TrustedRootRecord {
    pub id: String,
    pub public_key: String,
    #[serde(default)]
    pub revoked: bool,
    pub expires_unix: Option<u64>,
    pub rotated_from: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum TrustedRootFileFormat {
    List(Vec<TrustedRootRecord>),
    Wrapped { roots: Vec<TrustedRootRecord> },
    Keys { keys: Vec<TrustedRootRecord> },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Public struct `TrustMutationReport` used across Tau components.
pub struct TrustMutationReport {
    pub added: usize,
    pub updated: usize,
    pub revoked: usize,
    pub rotated: usize,
}

pub fn parse_trusted_root_spec(raw: &str) -> Result<TrustedKey> {
    let (id, public_key) = raw
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid --skill-trust-root '{raw}', expected key_id=base64_key"))?;
    let id = id.trim();
    let public_key = public_key.trim();
    if id.is_empty() || public_key.is_empty() {
        bail!("invalid --skill-trust-root '{raw}', expected key_id=base64_key");
    }
    Ok(TrustedKey {
        id: id.to_string(),
        public_key: public_key.to_string(),
    })
}

pub fn parse_trust_rotation_spec(raw: &str) -> Result<(String, TrustedKey)> {
    let (old_id, new_spec) = raw.split_once(':').ok_or_else(|| {
        anyhow!("invalid --skill-trust-rotate '{raw}', expected old_id:new_id=base64_key")
    })?;
    let old_id = old_id.trim();
    if old_id.is_empty() {
        bail!("invalid --skill-trust-rotate '{raw}', expected old_id:new_id=base64_key");
    }
    let new_key = parse_trusted_root_spec(new_spec)?;
    Ok((old_id.to_string(), new_key))
}

pub fn load_trust_root_records(path: &Path) -> Result<Vec<TrustedRootRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed = serde_json::from_str::<TrustedRootFileFormat>(&raw)
        .with_context(|| format!("failed to parse trusted root file {}", path.display()))?;

    let records = match parsed {
        TrustedRootFileFormat::List(items) => items,
        TrustedRootFileFormat::Wrapped { roots } => roots,
        TrustedRootFileFormat::Keys { keys } => keys,
    };

    Ok(records)
}

pub fn save_trust_root_records(path: &Path, records: &[TrustedRootRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    let mut payload = serde_json::to_string_pretty(&TrustedRootFileFormat::Wrapped {
        roots: records.to_vec(),
    })
    .context("failed to serialize trusted root records")?;
    payload.push('\n');
    write_text_atomic(path, &payload)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn apply_trust_root_mutation_specs(
    records: &mut Vec<TrustedRootRecord>,
    add_specs: &[String],
    revoke_ids: &[String],
    rotate_specs: &[String],
) -> Result<TrustMutationReport> {
    let mut report = TrustMutationReport::default();

    for spec in add_specs {
        let key = parse_trusted_root_spec(spec)?;
        if let Some(existing) = records.iter_mut().find(|record| record.id == key.id) {
            existing.public_key = key.public_key;
            existing.revoked = false;
            existing.rotated_from = None;
            report.updated += 1;
        } else {
            records.push(TrustedRootRecord {
                id: key.id,
                public_key: key.public_key,
                revoked: false,
                expires_unix: None,
                rotated_from: None,
            });
            report.added += 1;
        }
    }

    for id in revoke_ids {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        let record = records
            .iter_mut()
            .find(|record| record.id == id)
            .ok_or_else(|| anyhow!("cannot revoke unknown trust key id '{}'", id))?;
        if !record.revoked {
            record.revoked = true;
            report.revoked += 1;
        }
    }

    for spec in rotate_specs {
        let (old_id, new_key) = parse_trust_rotation_spec(spec)?;
        let old = records
            .iter_mut()
            .find(|record| record.id == old_id)
            .ok_or_else(|| anyhow!("cannot rotate unknown trust key id '{}'", old_id))?;
        old.revoked = true;

        if let Some(existing_new) = records.iter_mut().find(|record| record.id == new_key.id) {
            existing_new.public_key = new_key.public_key;
            existing_new.revoked = false;
            existing_new.rotated_from = Some(old_id.clone());
            report.updated += 1;
        } else {
            records.push(TrustedRootRecord {
                id: new_key.id,
                public_key: new_key.public_key,
                revoked: false,
                expires_unix: None,
                rotated_from: Some(old_id.clone()),
            });
            report.added += 1;
        }
        report.rotated += 1;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        apply_trust_root_mutation_specs, load_trust_root_records, parse_trust_rotation_spec,
        parse_trusted_root_spec, save_trust_root_records, TrustedRootRecord,
    };

    #[test]
    fn unit_parse_trusted_root_spec_accepts_key_id_and_key() {
        let key = parse_trusted_root_spec("root_a=ZmFrZV9rZXk=").expect("parse trusted root");
        assert_eq!(key.id, "root_a");
        assert_eq!(key.public_key, "ZmFrZV9rZXk=");
    }

    #[test]
    fn functional_save_and_load_trust_root_records_roundtrip() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("trust-roots.json");
        let records = vec![TrustedRootRecord {
            id: "root_a".to_string(),
            public_key: "ZmFrZV9rZXk=".to_string(),
            revoked: false,
            expires_unix: Some(123),
            rotated_from: None,
        }];

        save_trust_root_records(&path, &records).expect("save trust roots");
        let loaded = load_trust_root_records(&path).expect("load trust roots");
        assert_eq!(loaded, records);
    }

    #[test]
    fn integration_apply_trust_root_mutations_tracks_add_revoke_rotate() {
        let mut records = vec![TrustedRootRecord {
            id: "root_a".to_string(),
            public_key: "old".to_string(),
            revoked: false,
            expires_unix: None,
            rotated_from: None,
        }];

        let report = apply_trust_root_mutation_specs(
            &mut records,
            &["root_b=new".to_string()],
            &["root_a".to_string()],
            &["root_a:root_c=rotated".to_string()],
        )
        .expect("apply mutations");

        assert_eq!(report.added, 2);
        assert_eq!(report.revoked, 1);
        assert_eq!(report.rotated, 1);
        assert!(records
            .iter()
            .any(|record| record.id == "root_a" && record.revoked));
        assert!(records.iter().any(
            |record| record.id == "root_c" && record.rotated_from.as_deref() == Some("root_a")
        ));
    }

    #[test]
    fn regression_load_trust_root_records_rejects_invalid_payload() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("trust-roots-invalid.json");
        std::fs::write(&path, "{not-json").expect("write invalid payload");

        let error = load_trust_root_records(Path::new(&path))
            .expect_err("invalid trust root payload should fail");
        assert!(error
            .to_string()
            .contains("failed to parse trusted root file"));
    }

    #[test]
    fn regression_parse_trust_rotation_spec_rejects_missing_delimiter() {
        let error = parse_trust_rotation_spec("root_a=next")
            .expect_err("rotation spec without ':' should fail");
        assert!(error.to_string().contains("invalid --skill-trust-rotate"));
    }
}
