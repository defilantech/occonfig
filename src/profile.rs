//! Profile type and on-disk storage.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A named set of model assignments.
///
/// A profile carries model assignments only. It never records `provider`,
/// `mcp`, `plugin`, `tools`, or `permission`; those are global configuration
/// and rewriting them because the user switched models would be a surprise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// The top-level `model` key, if the config had one when saved.
    pub model: Option<String>,
    /// Agent name to model reference.
    pub agents: BTreeMap<String, String>,
    /// RFC 3339 timestamp of when the profile was saved.
    pub saved_at: String,
}

impl Profile {
    /// Every model reference the profile carries, top-level first.
    pub fn references(&self) -> Vec<&str> {
        let mut out = Vec::new();
        if let Some(model) = self.model.as_deref() {
            out.push(model);
        }
        out.extend(self.agents.values().map(String::as_str));
        out
    }
}

/// The directory profiles are stored in, beside the opencode config.
pub fn dir() -> Result<PathBuf> {
    Ok(crate::config::config_dir()?.join("profiles"))
}

/// The on-disk path for a named profile.
pub fn path_for(name: &str) -> Result<PathBuf> {
    Ok(dir()?.join(format!("{name}.json")))
}

/// Write a profile, creating the profiles directory if needed.
pub fn save(profile: &Profile) -> Result<PathBuf> {
    let dir = dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;

    let path = path_for(&profile.name)?;
    let json = serde_json::to_string_pretty(profile).context("could not serialize the profile")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

/// Read a named profile.
pub fn load(name: &str) -> Result<Profile> {
    let path = path_for(name)?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("no profile named '{name}' at {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("profile '{name}' at {} is not valid", path.display()))
}

/// Does a profile with this name already exist?
pub fn exists(name: &str) -> Result<bool> {
    Ok(path_for(name)?.exists())
}

/// Every stored profile, sorted by name. A malformed profile file is reported
/// as an error rather than silently skipped: a profile the user cannot load is
/// something they need to know about.
pub fn list() -> Result<Vec<Profile>> {
    let dir = dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names: Vec<String> = Vec::new();
    for entry in
        std::fs::read_dir(&dir).with_context(|| format!("could not read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();

    let mut out = Vec::new();
    for name in names {
        out.push(load(&name)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Profile {
        let mut agents = BTreeMap::new();
        agents.insert("build".to_string(), "a/two".to_string());
        Profile {
            name: "lab".to_string(),
            model: Some("a/one".to_string()),
            agents,
            saved_at: "2026-09-13T16:07:26Z".to_string(),
        }
    }

    #[test]
    fn profile_round_trips_through_json() {
        let original = sample();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn references_includes_top_level_first() {
        let profile = sample();
        let refs = profile.references();
        assert_eq!(refs, vec!["a/one", "a/two"]);
    }

    #[test]
    fn references_handles_a_profile_without_a_top_level_model() {
        let mut p = sample();
        p.model = None;
        assert_eq!(p.references(), vec!["a/two"]);
    }
}
