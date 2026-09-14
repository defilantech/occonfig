//! `occonfig current`: show the active model and drift.

use anyhow::Result;
use serde_json::Value;

use crate::config;

/// A summary of the model assignments currently in effect.
#[derive(Debug, PartialEq, Eq)]
pub struct Current {
    pub top_level: Option<String>,
    pub agents_on_top_level: usize,
    pub agents_total: usize,
    /// Agents whose model differs from the top-level one.
    pub drifted: Vec<(String, String)>,
}

/// Compute the current state from a config value.
pub fn summarize(cfg: &Value) -> Current {
    let top_level = cfg.get("model").and_then(Value::as_str).map(str::to_string);

    let mut agents_total = 0usize;
    let mut agents_on_top_level = 0usize;
    let mut drifted = Vec::new();

    if let Some(agents) = cfg.get("agent").and_then(Value::as_object) {
        for (name, agent) in agents {
            let Some(model) = agent.get("model").and_then(Value::as_str) else {
                continue;
            };
            agents_total += 1;
            match top_level.as_deref() {
                Some(top) if top == model => agents_on_top_level += 1,
                Some(_) => drifted.push((name.clone(), model.to_string())),
                None => drifted.push((name.clone(), model.to_string())),
            }
        }
    }

    drifted.sort();
    Current {
        top_level,
        agents_on_top_level,
        agents_total,
        drifted,
    }
}

/// Run the `current` subcommand.
pub fn run() -> Result<()> {
    let path = config::config_path()?;
    let cfg = config::load(&path)?;
    let now = summarize(&cfg);

    match now.top_level.as_deref() {
        Some(model) => println!("top level: {model}"),
        None => println!("top level: (none set)"),
    }
    println!(
        "agents:    {} of {} on the top-level model",
        now.agents_on_top_level, now.agents_total
    );

    if !now.drifted.is_empty() {
        println!("differing:");
        for (name, model) in &now.drifted {
            println!("  {name}: {model}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn summarize_counts_agents_on_and_off_the_top_level_model() {
        let cfg = json!({
            "model": "a/one",
            "agent": {
                "build": { "model": "a/one" },
                "writer": { "model": "a/other" },
                "reader": { "model": "a/one" }
            }
        });

        let now = summarize(&cfg);
        assert_eq!(now.top_level.as_deref(), Some("a/one"));
        assert_eq!(now.agents_total, 3);
        assert_eq!(now.agents_on_top_level, 2);
        assert_eq!(
            now.drifted,
            vec![("writer".to_string(), "a/other".to_string())]
        );
    }

    #[test]
    fn summarize_ignores_agents_without_a_model_key() {
        let cfg = json!({
            "model": "a/one",
            "agent": {
                "build": { "model": "a/one" },
                "toolish": { "description": "no model" }
            }
        });
        let now = summarize(&cfg);
        assert_eq!(now.agents_total, 1);
    }

    #[test]
    fn summarize_without_a_top_level_model_reports_every_agent_as_drifted() {
        let cfg = json!({
            "agent": { "build": { "model": "a/two" } }
        });
        let now = summarize(&cfg);
        assert!(now.top_level.is_none());
        assert_eq!(now.agents_on_top_level, 0);
        assert_eq!(now.drifted.len(), 1);
    }
}
