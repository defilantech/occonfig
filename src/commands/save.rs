//! `occonfig save <name>`: snapshot the current model assignments.

use anyhow::{bail, Result};
use chrono::Utc;

use crate::config;
use crate::profile::{self, Profile};

/// Build a profile from the current config contents.
pub fn snapshot(name: &str, cfg: &serde_json::Value) -> Profile {
    let models = config::models(cfg);

    let model = cfg
        .get("model")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    // Drop the synthetic top-level entry; `model` already carries it.
    let agents = models
        .into_iter()
        .filter(|(agent, _)| agent != "(top level)")
        .collect();

    Profile {
        name: name.to_string(),
        model,
        agents,
        saved_at: Utc::now().to_rfc3339(),
    }
}

/// Run the `save` subcommand against the real config file.
pub fn run(name: &str, force: bool) -> Result<()> {
    if name.trim().is_empty() {
        bail!("a profile name is required");
    }
    if name.contains(std::path::MAIN_SEPARATOR) {
        bail!("profile name '{name}' must not contain a path separator");
    }

    if profile::exists(name)? && !force {
        bail!("a profile named '{name}' already exists; pass --force to overwrite it");
    }

    let path = config::config_path()?;
    let cfg = config::load(&path)?;
    let snapshot = snapshot(name, &cfg);

    if snapshot.agents.is_empty() && snapshot.model.is_none() {
        bail!(
            "the config at {} declares no model assignments, so there is \
             nothing to save",
            path.display()
        );
    }

    let written = profile::save(&snapshot)?;
    let total = snapshot.references().len();
    println!(
        "saved profile '{name}' ({total} model assignment(s)) to {}",
        written.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn snapshot_captures_top_level_and_agents_without_duplicating() {
        let cfg = json!({
            "model": "a/one",
            "agent": {
                "build": { "model": "a/two" },
                "title": { "model": "a/three" }
            }
        });

        let p = snapshot("lab", &cfg);

        assert_eq!(p.name, "lab");
        assert_eq!(p.model.as_deref(), Some("a/one"));
        assert_eq!(p.agents.len(), 2);
        assert!(
            !p.agents.contains_key("(top level)"),
            "the synthetic top-level entry must not leak into the agent map"
        );
        assert_eq!(p.agents.get("build").map(String::as_str), Some("a/two"));
    }

    #[test]
    fn snapshot_of_a_config_without_a_top_level_model_still_works() {
        let cfg = json!({ "agent": { "build": { "model": "a/two" } } });
        let p = snapshot("lab", &cfg);
        assert!(p.model.is_none());
        assert_eq!(p.agents.len(), 1);
    }
}
