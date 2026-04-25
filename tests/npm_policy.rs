use std::time::{Duration, SystemTime};

use lfg::adapters::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases};
use lfg::npm_policy::{
    decide_resolved_npm_releases, release_facts_from_resolved_npm_releases, NpmPolicyError,
};
use lfg::policy::{ReleaseFacts, ReviewDecision, ReviewPolicy, SkipReason};

fn resolved_releases(target_published_at: &str) -> ResolvedPackageReleases {
    ResolvedPackageReleases {
        package_name: "left-pad".to_owned(),
        target: ResolvedPackageRelease {
            version: "1.1.0".to_owned(),
            published_at: target_published_at.to_owned(),
            archive: ArchiveRef {
                url: "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz".to_owned(),
            },
        },
        previous: ResolvedPackageRelease {
            version: "1.0.0".to_owned(),
            published_at: "1970-01-01T00:00:00.000Z".to_owned(),
            archive: ArchiveRef {
                url: "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz".to_owned(),
            },
        },
    }
}

#[test]
fn converts_target_publish_time_to_release_facts() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);
    let releases = resolved_releases("1970-01-02T00:00:00.000Z");

    assert_eq!(
        release_facts_from_resolved_npm_releases(&releases, now),
        Ok(ReleaseFacts {
            target_age: Some(Duration::from_secs(60 * 60)),
            has_previous_release: true,
        })
    );
}

#[test]
fn recent_npm_release_requires_review() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);
    let releases = resolved_releases("1970-01-02T00:00:00.000Z");

    assert_eq!(
        decide_resolved_npm_releases(&ReviewPolicy::default(), &releases, now),
        Ok(ReviewDecision::Review)
    );
}

#[test]
fn old_npm_release_skips_review() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(50 * 60 * 60);
    let releases = resolved_releases("1970-01-02T00:00:00.000Z");

    assert_eq!(
        decide_resolved_npm_releases(&ReviewPolicy::default(), &releases, now),
        Ok(ReviewDecision::Skip(SkipReason::OlderThanThreshold))
    );
}

#[test]
fn invalid_target_publish_time_returns_typed_error() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);
    let releases = resolved_releases("not-a-date");

    assert_eq!(
        decide_resolved_npm_releases(&ReviewPolicy::default(), &releases, now),
        Err(NpmPolicyError::InvalidTargetPublishTime(
            "not-a-date".to_owned()
        ))
    );
}

#[test]
fn future_target_publish_time_returns_typed_error() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60);
    let releases = resolved_releases("1970-01-03T00:00:00.000Z");

    assert_eq!(
        decide_resolved_npm_releases(&ReviewPolicy::default(), &releases, now),
        Err(NpmPolicyError::FutureTargetPublishTime(
            "1970-01-03T00:00:00.000Z".to_owned()
        ))
    );
}
