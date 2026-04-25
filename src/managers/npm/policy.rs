use std::time::{Duration, SystemTime};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::core::ResolvedPackageReleases;
use crate::core::{ReleaseDecisionError, ReleaseDecisionEvaluator};
use crate::core::{ReleaseFacts, ReviewDecision, ReviewPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
enum NpmPolicyError {
    InvalidTargetPublishTime(String),
    FutureTargetPublishTime(String),
}

fn release_facts_from_resolved_npm_releases(
    releases: &ResolvedPackageReleases,
    now: SystemTime,
) -> Result<ReleaseFacts, NpmPolicyError> {
    let published_at = parse_target_publish_time(&releases.target.published_at)?;
    let now = OffsetDateTime::from(now);
    let Some(age_seconds) = now
        .unix_timestamp()
        .checked_sub(published_at.unix_timestamp())
    else {
        return Err(NpmPolicyError::FutureTargetPublishTime(
            releases.target.published_at.clone(),
        ));
    };
    if age_seconds < 0 {
        return Err(NpmPolicyError::FutureTargetPublishTime(
            releases.target.published_at.clone(),
        ));
    }

    Ok(ReleaseFacts {
        target_age: Some(Duration::from_secs(age_seconds as u64)),
        has_previous_release: true,
    })
}

fn decide_resolved_npm_releases(
    policy: &ReviewPolicy,
    releases: &ResolvedPackageReleases,
    now: SystemTime,
) -> Result<ReviewDecision, NpmPolicyError> {
    let facts = release_facts_from_resolved_npm_releases(releases, now)?;
    Ok(policy.decide(&facts))
}

#[derive(Debug, Clone, Copy)]
pub struct NpmReleaseDecisionEvaluator<'a> {
    policy: &'a ReviewPolicy,
}

impl<'a> NpmReleaseDecisionEvaluator<'a> {
    pub const fn new(policy: &'a ReviewPolicy) -> Self {
        Self { policy }
    }
}

impl ReleaseDecisionEvaluator for NpmReleaseDecisionEvaluator<'_> {
    fn decide(
        &self,
        releases: &ResolvedPackageReleases,
        now: SystemTime,
    ) -> Result<ReviewDecision, ReleaseDecisionError> {
        decide_resolved_npm_releases(self.policy, releases, now).map_err(|error| match error {
            NpmPolicyError::InvalidTargetPublishTime(_)
            | NpmPolicyError::FutureTargetPublishTime(_) => {
                ReleaseDecisionError::MissingTargetPublishTime
            }
        })
    }
}

fn parse_target_publish_time(published_at: &str) -> Result<OffsetDateTime, NpmPolicyError> {
    OffsetDateTime::parse(published_at, &Rfc3339)
        .map_err(|_| NpmPolicyError::InvalidTargetPublishTime(published_at.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases, SkipReason};

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
}
