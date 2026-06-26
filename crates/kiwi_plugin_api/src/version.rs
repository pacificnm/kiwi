/// Semver of this `kiwi_plugin_api` crate.
///
/// Plugin crates should depend on a matching major version. Kiwi compares this
/// value against the API version a plugin was built against during load.
pub const API_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Dynamic-library symbol exported by plugins for initialization.
pub const PLUGIN_INIT_SYMBOL: &str = "kiwi_plugin_init";

/// Alias kept for documentation examples.
pub const DEFAULT_PLUGIN_INIT_SYMBOL: &str = PLUGIN_INIT_SYMBOL;

/// Returns `true` when `plugin_api_version` matches this crate's major version.
#[must_use]
pub fn api_version_compatible(plugin_api_version: &str) -> bool {
    semver_major(plugin_api_version) == semver_major(API_VERSION)
}

/// Returns `true` when `plugin_min_kiwi` is satisfied by `running_kiwi_version`.
#[must_use]
pub fn kiwi_version_compatible(plugin_min_kiwi: &str, running_kiwi_version: &str) -> bool {
    semver_at_least(running_kiwi_version, plugin_min_kiwi)
}

fn semver_major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

fn semver_at_least(current: &str, minimum: &str) -> bool {
    let Some((current_major, current_minor, current_patch)) = parse_semver_triplet(current) else {
        return false;
    };
    let Some((min_major, min_minor, min_patch)) = parse_semver_triplet(minimum) else {
        return false;
    };

    (current_major, current_minor, current_patch) >= (min_major, min_minor, min_patch)
}

fn parse_semver_triplet(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_version_matches_crate_version() {
        assert!(api_version_compatible(API_VERSION));
    }

    #[test]
    fn api_version_rejects_different_major() {
        assert!(!api_version_compatible("99.0.0"));
    }

    #[test]
    fn kiwi_version_compares_semver_triplets() {
        assert!(kiwi_version_compatible("0.1.0", "0.1.5"));
        assert!(kiwi_version_compatible("0.1.0", "0.1.0"));
        assert!(!kiwi_version_compatible("0.2.0", "0.1.9"));
    }
}
