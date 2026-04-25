use std::time::{Duration, SystemTime};

use lfg::core::install_request::{InstallOperation, InstallRequest, InstallTarget, PackageManager};
use lfg::core::outcome::{PackageOutcome, ReviewUnavailableReason};
use lfg::core::policy::{AskReason, ReviewPolicy, SkipReason};
use lfg::managers::npm::registry::{NpmFetchError, NpmPackumentClient, NpmRegistryResolver};
use lfg::managers::npm::review::evaluate_npm_install_request;

const OLD_PACKUMENT: &str = r#"{
  "name": "old-package",
  "dist-tags": { "latest": "1.1.0" },
  "time": {
    "1.0.0": "1970-01-01T00:00:00.000Z",
    "1.1.0": "1970-01-02T00:00:00.000Z"
  },
  "versions": {
    "1.0.0": {
      "dist": { "tarball": "https://registry.npmjs.org/old-package/-/old-package-1.0.0.tgz" }
    },
    "1.1.0": {
      "dist": { "tarball": "https://registry.npmjs.org/old-package/-/old-package-1.1.0.tgz" }
    }
  }
}"#;

const RECENT_PACKUMENT: &str = r#"{
  "name": "recent-package",
  "dist-tags": { "latest": "1.1.0" },
  "time": {
    "1.0.0": "1970-01-01T00:00:00.000Z",
    "1.1.0": "1970-01-02T00:00:00.000Z"
  },
  "versions": {
    "1.0.0": {
      "dist": { "tarball": "https://registry.npmjs.org/recent-package/-/recent-package-1.0.0.tgz" }
    },
    "1.1.0": {
      "dist": { "tarball": "https://registry.npmjs.org/recent-package/-/recent-package-1.1.0.tgz" }
    }
  }
}"#;

const ONE_VERSION_PACKUMENT: &str = r#"{
  "name": "one-version",
  "dist-tags": { "latest": "1.0.0" },
  "time": { "1.0.0": "1970-01-02T00:00:00.000Z" },
  "versions": {
    "1.0.0": {
      "dist": { "tarball": "https://registry.npmjs.org/one-version/-/one-version-1.0.0.tgz" }
    }
  }
}"#;

struct FixturePackumentClient;

impl NpmPackumentClient for FixturePackumentClient {
    fn fetch_packument(&self, package_name: &str) -> Result<String, NpmFetchError> {
        match package_name {
            "old-package" => Ok(OLD_PACKUMENT.to_owned()),
            "recent-package" => Ok(RECENT_PACKUMENT.to_owned()),
            "one-version" => Ok(ONE_VERSION_PACKUMENT.to_owned()),
            other => Err(NpmFetchError::Unavailable(format!(
                "missing fixture: {other}"
            ))),
        }
    }
}

fn install_request(specs: &[&str]) -> InstallRequest {
    InstallRequest {
        manager: PackageManager::Npm,
        operation: InstallOperation::Install,
        targets: specs
            .iter()
            .map(|spec| InstallTarget {
                spec: (*spec).to_owned(),
            })
            .collect(),
        manager_args: specs.iter().map(|spec| (*spec).to_owned()).collect(),
    }
}

#[test]
fn old_npm_release_maps_to_skipped_outcome() {
    let resolver = NpmRegistryResolver::new(FixturePackumentClient);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(50 * 60 * 60);

    assert_eq!(
        evaluate_npm_install_request(
            &install_request(&["old-package"]),
            &resolver,
            &ReviewPolicy::default(),
            now,
        ),
        vec![PackageOutcome::Skipped(SkipReason::OlderThanThreshold)]
    );
}

#[test]
fn recent_npm_release_waits_for_diff_review() {
    let resolver = NpmRegistryResolver::new(FixturePackumentClient);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);

    assert_eq!(
        evaluate_npm_install_request(
            &install_request(&["recent-package"]),
            &resolver,
            &ReviewPolicy::default(),
            now,
        ),
        vec![PackageOutcome::ReviewUnavailable(
            ReviewUnavailableReason::DiffFailure
        )]
    );
}

#[test]
fn missing_previous_release_maps_to_policy_ask() {
    let resolver = NpmRegistryResolver::new(FixturePackumentClient);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);

    assert_eq!(
        evaluate_npm_install_request(
            &install_request(&["one-version"]),
            &resolver,
            &ReviewPolicy::default(),
            now,
        ),
        vec![PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)]
    );
}

#[test]
fn registry_fetch_error_maps_to_ask_outcome() {
    let resolver = NpmRegistryResolver::new(FixturePackumentClient);
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);

    assert_eq!(
        evaluate_npm_install_request(
            &install_request(&["missing-package"]),
            &resolver,
            &ReviewPolicy::default(),
            now,
        ),
        vec![PackageOutcome::ReviewUnavailable(
            ReviewUnavailableReason::RegistryFailure
        )]
    );
}
