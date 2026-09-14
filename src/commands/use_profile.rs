//! `occonfig use <name>`: apply a saved profile to the live config.

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::config;
use crate::profile::Profile;
use crate::validate;

/// Apply a profile to `cfg`, returning the list of changes made.
///
/// Only the keys the profile carries are rewritten. `provider`, `mcp`,
/// `plugin`, `tools`, and `permission` are never touched.
///
/// An agent named in the profile that does not exist in the config is skipped
/// unless `add_agents` is set, so a profile cannot silently introduce agents
/// the user never defined.
pub fn apply(cfg: &mut Value, profile: &Profile, add_agents: bool) -> Result<Vec<String>> {
    let mut changes = Vec::new();

    if let Some(model) = profile.model.as_deref() {
        let previous = cfg.get("model").and_then(Value::as_str).unwrap_or("(none)");
        if previous != model {
            changes.push(format!("(top level): {previous} -> {model}"));
        }
        cfg["model"] = Value::String(model.to_string());
    }

    let Some(agents) = cfg.get_mut("agent").and_then(Value::as_object_mut) else {
        if !profile.agents.is_empty() {
            bail!(
                "the config has no `agent` block, so the profile's {} agent \
                 assignment(s) cannot be applied",
                profile.agents.len()
            );
        }
        return Ok(changes);
    };

    let mut skipped = Vec::new();
    for (name, model) in &profile.agents {
        match agents.get_mut(name) {
            Some(agent) => {
                let previous = agent
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("(none)");
                if previous != model {
                    changes.push(format!("{name}: {previous} -> {model}"));
                }
                agent["model"] = Value::String(model.clone());
            }
            None => skipped.push(name.clone()),
        }
    }

    if !skipped.is_empty() {
        if add_agents {
            for name in &skipped {
                let model = &profile.agents[name];
                agents.insert(name.clone(), serde_json::json!({ "model": model }));
                changes.push(format!("{name}: (new agent) -> {model}"));
            }
        } else {
            eprintln!(
                "note: {} agent(s) in the profile are not defined in the config \
                 and were skipped: {}",
                skipped.len(),
                skipped.join(", ")
            );
            eprintln!("      pass --add-agents to create them.");
        }
    }

    Ok(changes)
}

/// Run the `use` subcommand against the real config file.
pub fn run(name: &str, add_agents: bool, dry_run: bool) -> Result<()> {
    let path = config::config_path()?;
    let profile = crate::profile::load(name)?;
    let mut cfg = config::load(&path)?;

    // Validate before backup and before write, so a stale profile fails
    // without mutating the file. See AGENTS.md, "User data invariants".
    for reference in profile.references() {
        if let Err(err) = validate::validate_ref(&cfg, reference) {
            bail!(
                "profile '{name}' references a model that is not in your config: {err}\n\
                 the config was not modified."
            );
        }
    }

    let changes = apply(&mut cfg, &profile, add_agents)?;

    if changes.is_empty() {
        println!("already using profile '{name}'; nothing to change");
        return Ok(());
    }

    if dry_run {
        println!(
            "would change {} assignment(s) for profile '{name}':",
            changes.len()
        );
        for change in &changes {
            println!("  {change}");
        }
        return Ok(());
    }

    let backup_path = crate::backup::backup(&path)?;
    let json = serde_json::to_string_pretty(&cfg).context("could not serialize the config")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("could not write {}", path.display()))?;

    println!("applied profile '{name}' ({} change(s))", changes.len());
    for change in &changes {
        println!("  {change}");
    }
    println!("backup: {}", backup_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn profile() -> Profile {
        Profile {
            name: "lab".to_string(),
            model: Some("a/new".to_string()),
            agents: [("build".to_string(), "a/new".to_string())]
                .into_iter()
                .collect(),
            saved_at: "2026-09-13T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn apply_rewrites_only_the_profile_keys() {
        let mut cfg = json!({
            "model": "a/old",
            "agent": {
                "build": { "model": "a/old", "temperature": 0.3 },
                "writer": { "model": "a/writer" }
            },
            "mcp": { "keep": "me" }
        });

        let changes = apply(&mut cfg, &profile(), false).unwrap();

        assert_eq!(cfg["model"], json!("a/new"));
        assert_eq!(cfg["agent"]["build"]["model"], json!("a/new"));
        // The sibling key on the agent survives.
        assert_eq!(cfg["agent"]["build"]["temperature"], json!(0.3));
        // An agent the profile does not mention is untouched.
        assert_eq!(cfg["agent"]["writer"]["model"], json!("a/writer"));
        // Global blocks are untouched.
        assert_eq!(cfg["mcp"], json!({ "keep": "me" }));
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn apply_skips_unknown_agents_by_default() {
        let mut cfg = json!({
            "agent": { "build": { "model": "a/old" } }
        });
        let mut p = profile();
        p.agents.insert("ghost".to_string(), "a/new".to_string());

        apply(&mut cfg, &p, false).unwrap();

        assert!(
            cfg["agent"].get("ghost").is_none(),
            "an agent absent from the config must not be created without --add-agents"
        );
    }

    #[test]
    fn apply_creates_unknown_agents_with_add_agents() {
        let mut cfg = json!({
            "agent": { "build": { "model": "a/old" } }
        });
        let mut p = profile();
        p.agents.insert("ghost".to_string(), "a/new".to_string());

        apply(&mut cfg, &p, true).unwrap();

        assert_eq!(cfg["agent"]["ghost"]["model"], json!("a/new"));
    }

    #[test]
    fn apply_reports_no_changes_when_already_current() {
        let mut cfg = json!({
            "model": "a/new",
            "agent": { "build": { "model": "a/new" } }
        });
        let changes = apply(&mut cfg, &profile(), false).unwrap();
        assert!(changes.is_empty(), "expected no changes, got {changes:?}");
    }
}
