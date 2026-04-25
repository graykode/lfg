use std::time::SystemTime;

use crate::adapters::{EcosystemReleaseResolver, ResolveError};
use crate::install_request::InstallRequest;
use crate::npm_policy::{decide_resolved_npm_releases, NpmPolicyError};
use crate::orchestrator::{PackageOutcome, ReviewUnavailableReason};
use crate::policy::{AskReason, ReviewDecision, ReviewPolicy};

pub fn evaluate_npm_install_request<R: EcosystemReleaseResolver>(
    request: &InstallRequest,
    resolver: &R,
    policy: &ReviewPolicy,
    now: SystemTime,
) -> Vec<PackageOutcome> {
    request
        .targets
        .iter()
        .map(|target| {
            let releases = match resolver.resolve(target) {
                Ok(releases) => releases,
                Err(error) => return outcome_from_registry_error(error),
            };
            let decision = match decide_resolved_npm_releases(policy, &releases, now) {
                Ok(decision) => decision,
                Err(error) => return outcome_from_policy_error(error),
            };

            outcome_from_policy_decision(decision)
        })
        .collect()
}

fn outcome_from_policy_decision(decision: ReviewDecision) -> PackageOutcome {
    match PackageOutcome::from_policy_decision(decision) {
        Some(outcome) => outcome,
        None => PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure),
    }
}

fn outcome_from_registry_error(error: ResolveError) -> PackageOutcome {
    match error {
        ResolveError::MissingPreviousRelease => {
            PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)
        }
        ResolveError::MissingPublishTime(_) => {
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        }
        ResolveError::RegistryUnavailable(_)
        | ResolveError::InvalidMetadata
        | ResolveError::MissingLatestDistTag
        | ResolveError::MissingTargetVersion(_)
        | ResolveError::MissingTarball(_) => {
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::RegistryFailure)
        }
    }
}

fn outcome_from_policy_error(error: NpmPolicyError) -> PackageOutcome {
    match error {
        NpmPolicyError::InvalidTargetPublishTime(_)
        | NpmPolicyError::FutureTargetPublishTime(_) => {
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        }
    }
}
