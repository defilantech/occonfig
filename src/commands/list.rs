//! `occonfig list`: enumerate saved profiles.

use anyhow::Result;

use crate::profile;

/// A one-line summary of what a profile pins.
pub fn summarize(profile: &profile::Profile) -> String {
    let top = profile.model.as_deref().unwrap_or("(no top-level model)");
    format!("{top}; {} agent(s)", profile.agents.len())
}

/// Run the `list` subcommand.
pub fn run() -> Result<()> {
    let profiles = profile::list()?;

    if profiles.is_empty() {
        println!("no profiles saved yet");
        println!("run `occonfig save <name>` to capture the current model assignments");
        return Ok(());
    }

    for p in &profiles {
        println!("{:<20} {}  {}", p.name, p.saved_at, summarize(p));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn summarize_reports_the_top_level_model_and_agent_count() {
        let mut agents = BTreeMap::new();
        agents.insert("build".to_string(), "a/two".to_string());
        let p = profile::Profile {
            name: "lab".to_string(),
            model: Some("a/one".to_string()),
            agents,
            saved_at: "2026-09-13T00:00:00Z".to_string(),
        };
        assert_eq!(summarize(&p), "a/one; 1 agent(s)");
    }

    #[test]
    fn summarize_says_so_when_there_is_no_top_level_model() {
        let p = profile::Profile {
            name: "sparse".to_string(),
            model: None,
            agents: BTreeMap::new(),
            saved_at: "2026-09-13T00:00:00Z".to_string(),
        };
        assert_eq!(summarize(&p), "(no top-level model); 0 agent(s)");
    }
}
