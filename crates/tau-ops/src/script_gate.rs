//! CI gate for detecting unregistered scripts.
//!
//! Walks a scripts directory and reports any `.sh` files not present in a
//! known registered-names list, skipping an archive subdirectory.

use std::path::Path;

/// Check for `.sh` files in `scripts_dir` (excluding `archive_dir`) that are
/// not present in `registered_names`. Returns the list of unregistered script
/// file names (stem only, without path or extension).
pub fn check_unregistered_scripts(
    scripts_dir: &Path,
    archive_dir: &Path,
    registered_names: &[&str],
) -> Vec<String> {
    let mut orphans = Vec::new();
    let entries = match std::fs::read_dir(scripts_dir) {
        Ok(e) => e,
        Err(_) => return orphans,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Skip the archive directory
        if path.is_dir() {
            if let (Ok(canon_path), Ok(canon_archive)) =
                (path.canonicalize(), archive_dir.canonicalize())
            {
                if canon_path == canon_archive {
                    continue;
                }
            }
            // Recurse into non-archive subdirectories
            let sub = check_unregistered_scripts(&path, archive_dir, registered_names);
            orphans.extend(sub);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("sh") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if !registered_names.contains(&stem) {
                    orphans.push(stem.to_string());
                }
            }
        }
    }
    orphans.sort();
    orphans
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn unit_check_unregistered_scripts_finds_orphans() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts = tmp.path().join("scripts");
        let archive = scripts.join("archive");
        fs::create_dir_all(&archive).unwrap();

        // Registered script
        fs::write(scripts.join("deploy.sh"), "#!/bin/bash\necho deploy").unwrap();
        // Unregistered script
        fs::write(scripts.join("orphan.sh"), "#!/bin/bash\necho orphan").unwrap();
        // Archived script (should be skipped)
        fs::write(archive.join("old.sh"), "#!/bin/bash\necho old").unwrap();
        // Non-sh file (should be skipped)
        fs::write(scripts.join("readme.txt"), "info").unwrap();

        let registered = &["deploy"];
        let orphans = check_unregistered_scripts(&scripts, &archive, registered);
        assert_eq!(orphans, vec!["orphan".to_string()]);
    }

    #[test]
    fn unit_check_unregistered_scripts_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts = tmp.path().join("empty_scripts");
        let archive = tmp.path().join("archive");
        fs::create_dir_all(&scripts).unwrap();
        fs::create_dir_all(&archive).unwrap();

        let orphans = check_unregistered_scripts(&scripts, &archive, &[]);
        assert!(orphans.is_empty());
    }

    #[test]
    fn unit_check_unregistered_scripts_nonexistent_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts = tmp.path().join("nonexistent");
        let archive = tmp.path().join("archive");

        let orphans = check_unregistered_scripts(&scripts, &archive, &[]);
        assert!(orphans.is_empty());
    }
}
