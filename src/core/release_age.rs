use std::time::{Duration, SystemTime};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::core::{
    ReleaseDecisionError, ReleaseFacts, ResolvedPackageReleases, ReviewDecision, ReviewPolicy,
};

pub fn release_facts_from_resolved_releases(
    releases: &ResolvedPackageReleases,
    now: SystemTime,
) -> Result<ReleaseFacts, ReleaseDecisionError> {
    let published_at = OffsetDateTime::parse(&releases.target.published_at, &Rfc3339)
        .map_err(|_| ReleaseDecisionError::MissingTargetPublishTime)?;
    let now = OffsetDateTime::from(now);
    let Some(age_seconds) = now
        .unix_timestamp()
        .checked_sub(published_at.unix_timestamp())
    else {
        return Err(ReleaseDecisionError::MissingTargetPublishTime);
    };
    if age_seconds < 0 {
        return Err(ReleaseDecisionError::MissingTargetPublishTime);
    }

    Ok(ReleaseFacts {
        target_age: Some(Duration::from_secs(age_seconds as u64)),
        has_previous_release: true,
    })
}

pub fn decide_resolved_releases_by_publish_time(
    policy: &ReviewPolicy,
    releases: &ResolvedPackageReleases,
    now: SystemTime,
) -> Result<ReviewDecision, ReleaseDecisionError> {
    let facts = release_facts_from_resolved_releases(releases, now)?;
    Ok(policy.decide(&facts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ArchiveRef, ResolvedPackageRelease, ResolvedPackageReleases};

    fn resolved_releases(target_published_at: &str) -> ResolvedPackageReleases {
        ResolvedPackageReleases {
            package_name: "demo".to_owned(),
            target: ResolvedPackageRelease {
                version: "1.1.0".to_owned(),
                published_at: target_published_at.to_owned(),
                archive: ArchiveRef {
                    url: "https://example.test/demo-1.1.0.tar.gz".to_owned(),
                },
            },
            previous: ResolvedPackageRelease {
                version: "1.0.0".to_owned(),
                published_at: "1970-01-01T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://example.test/demo-1.0.0.tar.gz".to_owned(),
                },
            },
        }
    }

    #[test]
    fn converts_target_publish_time_to_release_facts() {
        assert_eq!(
            release_facts_from_resolved_releases(
                &resolved_releases("1970-01-02T00:00:00.000Z"),
                SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
            ),
            Ok(ReleaseFacts {
                target_age: Some(Duration::from_secs(60 * 60)),
                has_previous_release: true,
            })
        );
    }

    #[test]
    fn invalid_or_future_publish_time_returns_shared_error() {
        assert_eq!(
            release_facts_from_resolved_releases(
                &resolved_releases("not-a-date"),
                SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
            ),
            Err(ReleaseDecisionError::MissingTargetPublishTime)
        );
        assert_eq!(
            release_facts_from_resolved_releases(
                &resolved_releases("1970-01-03T00:00:00.000Z"),
                SystemTime::UNIX_EPOCH + Duration::from_secs(25 * 60 * 60),
            ),
            Err(ReleaseDecisionError::MissingTargetPublishTime)
        );
    }
}
