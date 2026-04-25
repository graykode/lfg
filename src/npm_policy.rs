use std::time::{Duration, SystemTime};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::adapters::ResolvedPackageReleases;
use crate::policy::{ReleaseFacts, ReviewDecision, ReviewPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpmPolicyError {
    InvalidTargetPublishTime(String),
    FutureTargetPublishTime(String),
}

pub fn release_facts_from_resolved_npm_releases(
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

pub fn decide_resolved_npm_releases(
    policy: &ReviewPolicy,
    releases: &ResolvedPackageReleases,
    now: SystemTime,
) -> Result<ReviewDecision, NpmPolicyError> {
    let facts = release_facts_from_resolved_npm_releases(releases, now)?;
    Ok(policy.decide(&facts))
}

fn parse_target_publish_time(published_at: &str) -> Result<OffsetDateTime, NpmPolicyError> {
    OffsetDateTime::parse(published_at, &Rfc3339)
        .map_err(|_| NpmPolicyError::InvalidTargetPublishTime(published_at.to_owned()))
}
