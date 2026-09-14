//! Back up the user's config before every mutation.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

/// Write a timestamped backup of `path` next to it, and return the backup path.
///
/// Naming follows the convention already present in user config directories:
/// `.pre-occonfig-<name>-<YYYYMMDD-HHMMSS>.bak`, where `<name>` is the file
/// stem. Every mutating subcommand calls this before it writes; there is no
/// code path that skips it (see AGENTS.md, "User data invariants").
pub fn backup(path: &Path) -> Result<PathBuf> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("could not read {} for backup", path.display()))?;

    let dir = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("opencode");

    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let backup_path = dir.join(format!(".pre-occonfig-{stem}-{stamp}.bak"));

    fs::write(&backup_path, contents)
        .with_context(|| format!("could not write backup {}", backup_path.display()))?;

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn backup_creates_a_timestamped_sibling_with_identical_contents() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("opencode.json");
        {
            let mut f = fs::File::create(&target).unwrap();
            f.write_all(br#"{"model":"a/one"}"#).unwrap();
        }

        let backup_path = backup(&target).unwrap();

        assert!(
            backup_path.exists(),
            "backup file was not created at {}",
            backup_path.display()
        );

        let name = backup_path.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with(".pre-occonfig-opencode-"),
            "backup name {name} does not match the convention"
        );
        assert!(
            name.ends_with(".bak"),
            "backup name {name} does not end in .bak"
        );

        assert_eq!(
            fs::read_to_string(&backup_path).unwrap(),
            fs::read_to_string(&target).unwrap(),
            "backup contents differ from the original"
        );

        // The backup lives beside the target, not in a temp or global location.
        assert_eq!(backup_path.parent().unwrap(), dir.path());
    }

    #[test]
    fn backup_fails_on_a_missing_target() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.json");
        assert!(backup(&missing).is_err());
    }
}
