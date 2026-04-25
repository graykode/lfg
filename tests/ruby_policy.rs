use std::time::{Duration, SystemTime};

use lfg::core::{
    ArchiveRef, ReleaseDecisionError, ReleaseDecisionEvaluator, ResolvedPackageRelease,
    ResolvedPackageReleases, ReviewDecision, ReviewPolicy,
};
use lfg::ecosystems::rubygems::RubyReleaseDecisionEvaluator;

#[test]
fn ruby_release_decision_evaluator_applies_review_policy() {
    let policy = ReviewPolicy::default();
    let evaluator = RubyReleaseDecisionEvaluator::new(&policy);
    let releases = resolved_releases("1970-01-02T00:00:00.000Z");

    assert_eq!(
        evaluator.decide(
            &releases,
            SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
        ),
        Ok(ReviewDecision::Review)
    );
}

#[test]
fn invalid_ruby_publish_time_maps_to_shared_decision_error() {
    let policy = ReviewPolicy::default();
    let evaluator = RubyReleaseDecisionEvaluator::new(&policy);

    assert_eq!(
        evaluator.decide(
            &resolved_releases("not-a-date"),
            SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
        ),
        Err(ReleaseDecisionError::MissingTargetPublishTime)
    );
}

fn resolved_releases(target_published_at: &str) -> ResolvedPackageReleases {
    ResolvedPackageReleases {
        package_name: "rack".to_owned(),
        previous: ResolvedPackageRelease {
            version: "2.2.0".to_owned(),
            published_at: "1970-01-01T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: "https://rubygems.org/gems/rack-2.2.0.gem".to_owned(),
            },
        },
        target: ResolvedPackageRelease {
            version: "3.0.0".to_owned(),
            published_at: target_published_at.to_owned(),
            archive: ArchiveRef {
                url: "https://rubygems.org/gems/rack-3.0.0.gem".to_owned(),
            },
        },
    }
}
