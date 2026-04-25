use std::time::{Duration, SystemTime};

use lfg::core::{
    ArchiveRef, ReleaseDecisionError, ReleaseDecisionEvaluator, ResolvedPackageRelease,
    ResolvedPackageReleases, ReviewDecision, ReviewPolicy,
};
use lfg::ecosystems::crates_io::RustReleaseDecisionEvaluator;

#[test]
fn rust_release_decision_evaluator_applies_review_policy() {
    let policy = ReviewPolicy::new(Duration::from_secs(24 * 60 * 60));
    let evaluator = RustReleaseDecisionEvaluator::new(&policy);
    let releases = ResolvedPackageReleases {
        package_name: "serde".to_owned(),
        previous: ResolvedPackageRelease {
            version: "1.0.227".to_owned(),
            published_at: "1970-01-01T00:00:00+00:00".to_owned(),
            archive: ArchiveRef {
                url: "https://crates.io/api/v1/crates/serde/1.0.227/download".to_owned(),
            },
        },
        target: ResolvedPackageRelease {
            version: "1.0.228".to_owned(),
            published_at: "1970-01-02T00:00:00+00:00".to_owned(),
            archive: ArchiveRef {
                url: "https://crates.io/api/v1/crates/serde/1.0.228/download".to_owned(),
            },
        },
    };
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);

    assert_eq!(evaluator.decide(&releases, now), Ok(ReviewDecision::Review));
}

#[test]
fn invalid_rust_publish_time_maps_to_shared_decision_error() {
    let policy = ReviewPolicy::default();
    let evaluator = RustReleaseDecisionEvaluator::new(&policy);
    let releases = ResolvedPackageReleases {
        package_name: "serde".to_owned(),
        previous: ResolvedPackageRelease {
            version: "1.0.227".to_owned(),
            published_at: "2026-04-23T00:00:00+00:00".to_owned(),
            archive: ArchiveRef {
                url: "https://crates.io/api/v1/crates/serde/1.0.227/download".to_owned(),
            },
        },
        target: ResolvedPackageRelease {
            version: "1.0.228".to_owned(),
            published_at: "not a timestamp".to_owned(),
            archive: ArchiveRef {
                url: "https://crates.io/api/v1/crates/serde/1.0.228/download".to_owned(),
            },
        },
    };

    assert_eq!(
        evaluator.decide(&releases, SystemTime::UNIX_EPOCH),
        Err(ReleaseDecisionError::MissingTargetPublishTime)
    );
}
