use std::time::SystemTime;

use crate::core::evaluate_install_request;
use crate::core::EcosystemReleaseResolver;
use crate::core::InstallRequest;
use crate::core::PackageOutcome;
use crate::core::ReviewPolicy;
use crate::ecosystems::npm::NpmReleaseDecisionEvaluator;

pub fn evaluate_npm_install_request<R: EcosystemReleaseResolver + ?Sized>(
    request: &InstallRequest,
    resolver: &R,
    policy: &ReviewPolicy,
    now: SystemTime,
) -> Vec<PackageOutcome> {
    let decision_evaluator = NpmReleaseDecisionEvaluator::new(policy);

    evaluate_install_request(request, resolver, &decision_evaluator, now)
}
