//! `occonfig set-model <provider/model>`: the one-liner.

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::config;
use crate::validate;

/// Rewrite the top-level model, and every agent that already carries a `model`
/// key, returning the list of changes made.
///
/// An agent with no `model` key inherits the top-level model. Writing one in
/// pins it, silently changing which model that agent uses. Such agents are
/// skipped unless `add_agents` is set.
pub fn apply(
    cfg: &mut Value,
    reference: &str,
    agents: bool,
    top_level: bool,
    add_agents: bool,
) -> Result<Vec<String>> {
    let mut changes = Vec::new();

    if top_level {
        let previous = cfg.get("model").and_then(Value::as_str).unwrap_or("(none)");
        if previous != reference {
            changes.push(format!("(top level): {previous} -> {reference}"));
        }
        cfg["model"] = Value::String(reference.to_string());
    }

    if agents {
        let Some(agents_block) = cfg.get_mut("agent").and_then(Value::as_object_mut) else {
            if top_level {
                return Ok(changes);
            }
            bail!("the config has no `agent` block to rewrite");
        };

        let mut skipped = Vec::new();
        for (name, agent) in agents_block.iter_mut() {
            let Some(current) = agent.get("model").and_then(Value::as_str) else {
                if add_agents {
                    agent["model"] = Value::String(reference.to_string());
                    changes.push(format!("{name}: (new model) -> {reference}"));
                } else {
                    skipped.push(name.clone());
                }
                continue;
            };
            if current == reference {
                continue;
            }
            changes.push(format!("{name}: {current} -> {reference}"));
            agent["model"] = Value::String(reference.to_string());
        }

        if !skipped.is_empty() {
            eprintln!(
                "note: {} agent(s) carry no model and inherit the top-level one; \
                 left untouched: {}",
                skipped.len(),
                skipped.join(", ")
            );
            eprintln!("      pass --add-agents to pin them explicitly.");
        }
    }

    Ok(changes)
}

/// Run the `set-model` subcommand.
pub fn run(reference: &str, agents_only: bool, add_agents: bool, dry_run: bool) -> Result<()> {
    let path = config::config_path()?;
    let mut cfg = config::load(&path)?;

    validate::validate_ref(&cfg, reference).map_err(|err| anyhow::anyhow!("{err}"))?;

    let changes = if agents_only {
        apply(&mut cfg, reference, true, false, add_agents)?
    } else {
        apply(&mut cfg, reference, true, true, add_agents)?
    };

    if changes.is_empty() {
        println!("every assignment already points at {reference}");
        return Ok(());
    }

    if dry_run {
        println!(
            "would change {} assignment(s) to {reference}:",
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

    println!("set {} assignment(s) to {reference}", changes.len());
    println!("backup: {}", backup_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn apply_sets_top_level_and_agents_by_default() {
        let mut cfg = json!({
            "model": "a/old",
            "agent": {
                "build": { "model": "a/old" },
                "writer": { "model": "a/writer", "temperature": 0.7 }
            }
        });

        let changes = apply(&mut cfg, "a/new", true, true, false).unwrap();

        assert_eq!(cfg["model"], json!("a/new"));
        assert_eq!(cfg["agent"]["build"]["model"], json!("a/new"));
        assert_eq!(cfg["agent"]["writer"]["model"], json!("a/new"));
        // The sibling key survives.
        assert_eq!(cfg["agent"]["writer"]["temperature"], json!(0.7));
        assert_eq!(changes.len(), 3);
    }

    #[test]
    fn apply_agents_only_leaves_the_top_level_model_alone() {
        let mut cfg = json!({
            "model": "a/old",
            "agent": { "build": { "model": "a/old" } }
        });

        apply(&mut cfg, "a/new", true, false, false).unwrap();

        assert_eq!(cfg["model"], json!("a/old"));
        assert_eq!(cfg["agent"]["build"]["model"], json!("a/new"));
    }

    #[test]
    fn apply_top_level_only_leaves_agents_alone() {
        let mut cfg = json!({
            "model": "a/old",
            "agent": { "build": { "model": "a/old" } }
        });

        apply(&mut cfg, "a/new", false, true, false).unwrap();

        assert_eq!(cfg["model"], json!("a/new"));
        assert_eq!(cfg["agent"]["build"]["model"], json!("a/old"));
    }

    #[test]
    fn apply_reports_no_changes_when_already_on_the_target() {
        let mut cfg = json!({
            "model": "a/new",
            "agent": { "build": { "model": "a/new" } }
        });
        let changes = apply(&mut cfg, "a/new", true, true, false).unwrap();
        assert!(changes.is_empty(), "expected no changes, got {changes:?}");
    }

    #[test]
    fn apply_errors_when_agents_requested_but_none_defined() {
        let mut cfg = json!({ "model": "a/old" });
        assert!(apply(&mut cfg, "a/new", true, false, false).is_err());
    }

    #[test]
    fn apply_does_not_pin_an_agent_that_carries_no_model() {
        // Regression: an agent with no `model` key inherits the top-level
        // model. Writing one in pins it and silently changes which model that
        // agent runs, which is a rewrite the user never asked for.
        let mut cfg = json!({
            "model": "a/old",
            "agent": {
                "build": { "model": "a/old" },
                "plan": { "options": { "maxOutputTokens": 32768 } }
            }
        });

        let changes = apply(&mut cfg, "a/new", true, true, false).unwrap();

        assert!(
            cfg["agent"]["plan"].get("model").is_none(),
            "an agent with no model key must not be pinned by set-model"
        );
        assert_eq!(
            cfg["agent"]["plan"]["options"]["maxOutputTokens"],
            json!(32768),
            "the agent's existing keys must survive"
        );
        // Only the top level and the agent that already had a model changed.
        assert_eq!(changes.len(), 2, "unexpected changes: {changes:?}");
    }

    #[test]
    fn apply_pins_a_modelless_agent_only_when_add_agents_is_set() {
        let mut cfg = json!({
            "model": "a/old",
            "agent": {
                "plan": { "options": { "maxOutputTokens": 32768 } }
            }
        });

        apply(&mut cfg, "a/new", true, true, true).unwrap();

        assert_eq!(cfg["agent"]["plan"]["model"], json!("a/new"));
        assert_eq!(
            cfg["agent"]["plan"]["options"]["maxOutputTokens"],
            json!(32768)
        );
    }
}
