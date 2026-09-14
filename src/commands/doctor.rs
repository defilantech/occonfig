//! `occonfig doctor`: validate the config and flag stale profiles.

use anyhow::Result;

use crate::config;
use crate::profile;
use crate::validate;

/// One problem found by doctor. Kept as data so the logic is unit-testable
/// without capturing stdout.
#[derive(Debug, PartialEq, Eq)]
pub enum Finding {
    /// The profile references a model that the config does not declare.
    StaleProfile {
        profile: String,
        reference: String,
        reason: String,
    },
    /// The config declares no providers at all.
    NoProviders,
}

impl Finding {
    /// A human-readable line for this finding.
    pub fn render(&self) -> String {
        match self {
            Finding::StaleProfile {
                profile,
                reference,
                reason,
            } => format!("profile '{profile}' references '{reference}': {reason}"),
            Finding::NoProviders => {
                "the config declares no providers, so no reference can resolve".to_string()
            }
        }
    }
}

/// Check every stored profile against the live config.
pub fn diagnose(cfg: &serde_json::Value, profiles: &[profile::Profile]) -> Vec<Finding> {
    let mut findings = Vec::new();

    if cfg.get("provider").and_then(|p| p.as_object()).is_none() {
        findings.push(Finding::NoProviders);
        return findings;
    }

    for profile in profiles {
        for reference in profile.references() {
            if let Err(reason) = validate::validate_ref(cfg, reference) {
                findings.push(Finding::StaleProfile {
                    profile: profile.name.clone(),
                    reference: reference.to_string(),
                    reason,
                });
            }
        }
    }

    findings
}

/// Run the `doctor` subcommand.
pub fn run() -> Result<()> {
    let path = config::config_path()?;
    let cfg = config::load(&path)?;
    println!("config: {}", path.display());

    let profiles = profile::list()?;
    println!("profiles: {} found", profiles.len());

    let findings = diagnose(&cfg, &profiles);
    if findings.is_empty() {
        println!("ok: every model reference resolves");
        return Ok(());
    }

    println!("{} problem(s) found:", findings.len());
    for finding in &findings {
        println!("  {}", finding.render());
    }
    // Findings are a report, not a failure: the user asked what is wrong, and
    // exiting non-zero here would make `doctor` awkward to chain in a script.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn profile(name: &str, model: &str) -> profile::Profile {
        let mut agents = BTreeMap::new();
        agents.insert("build".to_string(), model.to_string());
        profile::Profile {
            name: name.to_string(),
            model: Some(model.to_string()),
            agents,
            saved_at: "2026-09-13T00:00:00Z".to_string(),
        }
    }

    fn cfg() -> serde_json::Value {
        json!({
            "provider": { "agw": { "models": { "flash": {} } } }
        })
    }

    #[test]
    fn reports_a_stale_profile_reference() {
        let profiles = vec![profile("old", "removed/gone")];
        let findings = diagnose(&cfg(), &profiles);

        assert_eq!(findings.len(), 2, "expected one finding per reference");
        match &findings[0] {
            Finding::StaleProfile {
                profile, reference, ..
            } => {
                assert_eq!(profile, "old");
                assert_eq!(reference, "removed/gone");
            }
            other => panic!("expected a stale-profile finding, got {other:?}"),
        }
    }

    #[test]
    fn reports_nothing_for_profiles_that_resolve() {
        let profiles = vec![profile("good", "agw/flash")];
        assert!(diagnose(&cfg(), &profiles).is_empty());
    }

    #[test]
    fn reports_no_providers_when_the_config_has_none() {
        let findings = diagnose(&json!({}), &[]);
        assert_eq!(findings, vec![Finding::NoProviders]);
    }

    #[test]
    fn a_stale_profile_is_not_reported_as_clean() {
        // The discriminating assertion: the finding must exist, so a doctor
        // that silently claims everything is fine fails this test.
        let profiles = vec![profile("old", "removed/gone")];
        let findings = diagnose(&cfg(), &profiles);
        assert!(
            !findings.is_empty(),
            "a stale profile must produce at least one finding"
        );
    }
}
