//! Provider/model reference validation.

use serde_json::Value;

/// Check that a `provider/model` reference resolves against the live config.
///
/// A reference is valid when its provider exists under `provider` and its
/// model exists under `provider.<p>.models`. Anything else would be written
/// into the config as a dangling reference that opencode rejects.
///
/// This runs on the write path, before a backup is taken and before the file
/// is touched, so a stale profile fails without mutating anything.
pub fn validate_ref(cfg: &Value, reference: &str) -> Result<(), String> {
    let Some((provider, model)) = reference.split_once('/') else {
        return Err(format!(
            "'{reference}' is not a provider/model reference (expected a single '/')"
        ));
    };

    if provider.is_empty() || model.is_empty() {
        return Err(format!(
            "'{reference}' has an empty provider or model segment"
        ));
    }

    let Some(providers) = cfg.get("provider").and_then(Value::as_object) else {
        return Err(format!(
            "the config declares no providers, so '{reference}' cannot resolve"
        ));
    };

    let Some(spec) = providers.get(provider) else {
        return Err(format!(
            "provider '{provider}' is not present in the config"
        ));
    };

    let has_model = spec
        .get("models")
        .and_then(Value::as_object)
        .map(|models| models.contains_key(model))
        .unwrap_or(false);

    if !has_model {
        return Err(format!(
            "model '{model}' is not defined under provider '{provider}'"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg() -> Value {
        json!({
            "provider": {
                "agw": { "models": { "flash": {}, "pro": {} } }
            }
        })
    }

    #[test]
    fn accepts_a_reference_that_resolves() {
        assert!(validate_ref(&cfg(), "agw/flash").is_ok());
    }

    #[test]
    fn rejects_a_reference_without_a_slash() {
        let err = validate_ref(&cfg(), "agwflash").unwrap_err();
        assert!(err.contains("single '/'"), "unexpected error: {err}");
    }

    #[test]
    fn rejects_a_reference_with_an_empty_segment() {
        assert!(validate_ref(&cfg(), "agw/").is_err());
        assert!(validate_ref(&cfg(), "/flash").is_err());
    }

    #[test]
    fn rejects_an_unknown_provider() {
        let err = validate_ref(&cfg(), "ghost/whatever").unwrap_err();
        assert!(
            err.contains("provider 'ghost' is not present"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_an_unknown_model_under_a_known_provider() {
        let err = validate_ref(&cfg(), "agw/nonexistent").unwrap_err();
        assert!(
            err.contains("model 'nonexistent' is not defined"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn splits_on_the_first_slash_only() {
        // A model name containing a slash is unusual but the provider segment
        // is always the part before the first '/'.
        let cfg = json!({
            "provider": { "agw": { "models": { "a/b": {} } } }
        });
        assert!(validate_ref(&cfg, "agw/a/b").is_ok());
    }
}
