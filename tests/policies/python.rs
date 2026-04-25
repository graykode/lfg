use std::time::{Duration, SystemTime};

use lfg::core::{
    ArchiveRef, ReleaseDecisionError, ReleaseDecisionEvaluator, ResolvedPackageRelease,
    ResolvedPackageReleases, ReviewDecision, ReviewPolicy, SkipReason,
};
use lfg::ecosystems::pypi::PythonReleaseDecisionEvaluator;

fn resolved_releases(target_published_at: &str) -> ResolvedPackageReleases {
    ResolvedPackageReleases {
        package_name: "requests".to_owned(),
        target: ResolvedPackageRelease {
            version: "2.32.3".to_owned(),
            published_at: target_published_at.to_owned(),
            archive: ArchiveRef {
                url: "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz".to_owned(),
            },
        },
        previous: ResolvedPackageRelease {
            version: "2.32.2".to_owned(),
            published_at: "1970-01-01T00:00:00.000000Z".to_owned(),
            archive: ArchiveRef {
                url: "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz".to_owned(),
            },
        },
    }
}

#[test]
fn python_release_decision_evaluator_applies_review_policy() {
    let policy = ReviewPolicy::default();
    let evaluator = PythonReleaseDecisionEvaluator::new(&policy);

    assert_eq!(
        evaluator.decide(
            &resolved_releases("1970-01-02T00:00:00.000000Z"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
        ),
        Ok(ReviewDecision::Review)
    );
    assert_eq!(
        evaluator.decide(
            &resolved_releases("1970-01-02T00:00:00.000000Z"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(50 * 60 * 60),
        ),
        Ok(ReviewDecision::Skip(SkipReason::OlderThanThreshold))
    );
}

#[test]
fn invalid_python_publish_time_maps_to_shared_decision_error() {
    let policy = ReviewPolicy::default();
    let evaluator = PythonReleaseDecisionEvaluator::new(&policy);

    assert_eq!(
        evaluator.decide(
            &resolved_releases("not-a-date"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
        ),
        Err(ReleaseDecisionError::MissingTargetPublishTime)
    );
}
