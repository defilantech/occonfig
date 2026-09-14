//! Version string, populated at build time.

/// The version reported by `--version`.
///
/// GoReleaser injects the release tag via ldflags; a plain `cargo build` falls
/// back to the Cargo package version so the binary always reports something
/// coherent. Without the fallback, an unreleased local build would print an
/// empty string.
pub fn version() -> &'static str {
    option_env!("OCCONFIG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
}

/// The git commit the binary was built from, if the build injected one.
pub fn commit() -> Option<&'static str> {
    option_env!("OCCONFIG_COMMIT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_never_empty() {
        assert!(
            !version().is_empty(),
            "version() returned an empty string; the fallback should always \
             yield CARGO_PKG_VERSION"
        );
    }

    #[test]
    fn version_falls_back_to_cargo_package_version_when_not_injected() {
        // In a plain `cargo test` run the ldflags env var is absent, so the
        // fallback path is the one being exercised here.
        if option_env!("OCCONFIG_VERSION").is_none() {
            assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        }
    }
}
