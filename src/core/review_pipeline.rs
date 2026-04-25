use std::time::SystemTime;

use crate::core::InstallRequest;
use crate::core::{AskReason, ReviewDecision};
use crate::core::{EcosystemReleaseResolver, ResolveError, ResolvedPackageReleases};
use crate::core::{PackageOutcome, ReviewUnavailableReason};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseDecisionError {
    MissingTargetPublishTime,
}

pub trait ReleaseDecisionEvaluator {
    fn decide(
        &self,
        releases: &ResolvedPackageReleases,
        now: SystemTime,
    ) -> Result<ReviewDecision, ReleaseDecisionError>;
}

pub fn evaluate_install_request<R, E>(
    request: &InstallRequest,
    resolver: &R,
    decision_evaluator: &E,
    now: SystemTime,
) -> Vec<PackageOutcome>
where
    R: EcosystemReleaseResolver + ?Sized,
    E: ReleaseDecisionEvaluator + ?Sized,
{
    request
        .targets
        .iter()
        .map(|target| {
            let releases = match resolver.resolve(target) {
                Ok(releases) => releases,
                Err(error) => return outcome_from_resolve_error(error),
            };
            let decision = match decision_evaluator.decide(&releases, now) {
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

fn outcome_from_resolve_error(error: ResolveError) -> PackageOutcome {
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

fn outcome_from_policy_error(error: ReleaseDecisionError) -> PackageOutcome {
    match error {
        ReleaseDecisionError::MissingTargetPublishTime => {
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        }
    }
}
