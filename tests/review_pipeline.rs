use std::collections::BTreeMap;
use std::time::SystemTime;

use lfg::core::{evaluate_install_request, ReleaseDecisionError, ReleaseDecisionEvaluator};
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::core::{AskReason, ReviewDecision, SkipReason};
use lfg::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager};
use lfg::core::{PackageOutcome, ReviewUnavailableReason};

#[derive(Debug, Clone)]
struct StaticResolver {
    results: BTreeMap<String, Result<ResolvedPackageReleases, ResolveError>>,
}

impl EcosystemReleaseResolver for StaticResolver {
    fn id(&self) -> &'static str {
        "static"
    }

    fn resolve(&self, target: &InstallTarget) -> Result<ResolvedPackageReleases, ResolveError> {
        self.results
            .get(&target.spec)
            .cloned()
            .expect("test target should be configured")
    }
}

#[derive(Debug, Clone)]
struct StaticDecisionEvaluator {
    results: BTreeMap<String, Result<ReviewDecision, ReleaseDecisionError>>,
}

impl ReleaseDecisionEvaluator for StaticDecisionEvaluator {
    fn id(&self) -> &'static str {
        "static"
    }

    fn decide(
        &self,
        releases: &ResolvedPackageReleases,
        _now: SystemTime,
    ) -> Result<ReviewDecision, ReleaseDecisionError> {
        self.results
            .get(&releases.package_name)
            .cloned()
            .expect("test package should be configured")
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

fn releases(package_name: &str) -> ResolvedPackageReleases {
    ResolvedPackageReleases {
        package_name: package_name.to_owned(),
        previous: ResolvedPackageRelease {
            version: "1.0.0".to_owned(),
            published_at: "1970-01-01T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: format!("memory://{package_name}-1.0.0.tgz"),
            },
        },
        target: ResolvedPackageRelease {
            version: "1.1.0".to_owned(),
            published_at: "1970-01-02T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: format!("memory://{package_name}-1.1.0.tgz"),
            },
        },
    }
}

#[test]
fn maps_skip_and_review_decisions_without_manager_specific_logic() {
    let resolver = StaticResolver {
        results: BTreeMap::from([
            ("old-package".to_owned(), Ok(releases("old-package"))),
            ("recent-package".to_owned(), Ok(releases("recent-package"))),
        ]),
    };
    let evaluator = StaticDecisionEvaluator {
        results: BTreeMap::from([
            (
                "old-package".to_owned(),
                Ok(ReviewDecision::Skip(SkipReason::OlderThanThreshold)),
            ),
            ("recent-package".to_owned(), Ok(ReviewDecision::Review)),
        ]),
    };

    assert_eq!(
        evaluate_install_request(
            &install_request(&["old-package", "recent-package"]),
            &resolver,
            &evaluator,
            SystemTime::UNIX_EPOCH,
        ),
        vec![
            PackageOutcome::Skipped(SkipReason::OlderThanThreshold),
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure),
        ]
    );
}

#[test]
fn maps_resolver_and_policy_failures_to_fail_to_ask_outcomes() {
    let resolver = StaticResolver {
        results: BTreeMap::from([
            (
                "missing-previous".to_owned(),
                Err(ResolveError::MissingPreviousRelease),
            ),
            (
                "bad-metadata".to_owned(),
                Err(ResolveError::RegistryUnavailable("offline".to_owned())),
            ),
            ("bad-time".to_owned(), Ok(releases("bad-time"))),
        ]),
    };
    let evaluator = StaticDecisionEvaluator {
        results: BTreeMap::from([(
            "bad-time".to_owned(),
            Err(ReleaseDecisionError::MissingTargetPublishTime),
        )]),
    };

    assert_eq!(
        evaluate_install_request(
            &install_request(&["missing-previous", "bad-metadata", "bad-time"]),
            &resolver,
            &evaluator,
            SystemTime::UNIX_EPOCH,
        ),
        vec![
            PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease),
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::RegistryFailure),
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime),
        ]
    );
}
