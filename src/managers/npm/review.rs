use std::time::SystemTime;

use crate::core::contracts::EcosystemReleaseResolver;
use crate::core::install_request::InstallRequest;
use crate::core::outcome::PackageOutcome;
use crate::core::policy::ReviewPolicy;
use crate::core::review_pipeline::evaluate_install_request;
use crate::managers::npm::policy::NpmReleaseDecisionEvaluator;

pub fn evaluate_npm_install_request<R: EcosystemReleaseResolver + ?Sized>(
    request: &InstallRequest,
    resolver: &R,
    policy: &ReviewPolicy,
    now: SystemTime,
) -> Vec<PackageOutcome> {
    let decision_evaluator = NpmReleaseDecisionEvaluator::new(policy);

    evaluate_install_request(request, resolver, &decision_evaluator, now)
}
