//! Locate, load, and read the user's opencode configuration.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_json::Value;

/// The environment variable opencode uses to relocate its config directory.
const CONFIG_DIR_ENV: &str = "OPENCODE_CONFIG_DIR";
/// The environment variable opencode uses to point at a specific config file.
const CONFIG_FILE_ENV: &str = "OPENCODE_CONFIG";

/// Locate the opencode config directory, honoring the same precedence opencode
/// does: `OPENCODE_CONFIG_DIR` first, then the platform default.
pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os(CONFIG_DIR_ENV) {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    let home = dirs::home_dir().context("could not determine the home directory")?;
    Ok(home.join(".config").join("opencode"))
}

/// Locate the opencode config file, honoring `OPENCODE_CONFIG` (an explicit
/// file path) before the directory-based lookup.
pub fn config_path() -> Result<PathBuf> {
    if let Some(file) = std::env::var_os(CONFIG_FILE_ENV) {
        if !file.is_empty() {
            return Ok(PathBuf::from(file));
        }
    }
    Ok(config_dir()?.join("opencode.json"))
}

/// Load and parse the config, preserving object key order.
///
/// `serde_json` is compiled with `preserve_order` for exactly this reason:
/// without it, keys within an object are emitted alphabetically on write, so a
/// save would silently reorder the user's `agent` block. See AGENTS.md.
pub fn load(path: &PathBuf) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not valid JSON", path.display()))?;
    if !value.is_object() {
        bail!(
            "{} does not contain a JSON object at the top level",
            path.display()
        );
    }
    Ok(value)
}

/// The model assignments currently in effect: the top-level `model` key plus
/// every `agent.<name>.model`.
///
/// Agent names are sorted (this is a `BTreeMap`) because the caller uses this
/// for reporting and comparison, not for rewriting the file. The on-disk key
/// order is preserved separately by `preserve_order` in [`load`].
pub fn models(cfg: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();

    if let Some(model) = cfg.get("model").and_then(Value::as_str) {
        out.insert("(top level)".to_string(), model.to_string());
    }

    if let Some(agents) = cfg.get("agent").and_then(Value::as_object) {
        for (name, agent) in agents {
            if let Some(model) = agent.get("model").and_then(Value::as_str) {
                out.insert(name.clone(), model.to_string());
            }
        }
    }

    out
}

/// Every provider/model reference the config declares, as `provider/model`
/// strings. Used by `doctor` to check the config's own internal consistency.
pub fn provider_refs(cfg: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(providers) = cfg.get("provider").and_then(Value::as_object) {
        for (provider, spec) in providers {
            if let Some(models) = spec.get("models").and_then(Value::as_object) {
                for model in models.keys() {
                    refs.push(format!("{provider}/{model}"));
                }
            }
        }
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn models_reads_top_level_and_agents() {
        let cfg = json!({
            "model": "a/one",
            "agent": {
                "build": { "model": "a/two", "temperature": 0.3 },
                "title": { "model": "a/three" },
                "no-model": { "description": "has no model key" }
            }
        });

        let got = models(&cfg);
        assert_eq!(got.get("(top level)").map(String::as_str), Some("a/one"));
        assert_eq!(got.get("build").map(String::as_str), Some("a/two"));
        assert_eq!(got.get("title").map(String::as_str), Some("a/three"));
        assert!(
            !got.contains_key("no-model"),
            "an agent without a model key must not appear"
        );
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn models_is_empty_for_a_config_without_models() {
        let cfg = json!({ "mcp": {} });
        assert!(models(&cfg).is_empty());
    }

    #[test]
    fn provider_refs_enumerates_every_provider_model_pair() {
        let cfg = json!({
            "provider": {
                "p1": { "models": { "m1": {}, "m2": {} } },
                "p2": { "models": { "m3": {} } }
            }
        });
        let mut got = provider_refs(&cfg);
        got.sort();
        assert_eq!(got, vec!["p1/m1", "p1/m2", "p2/m3"]);
    }
}
