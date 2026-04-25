use std::time::SystemTime;

use crate::adapters::EcosystemReleaseResolver;
use crate::install_request::InstallRequest;
use crate::npm_policy::NpmReleaseDecisionEvaluator;
use crate::orchestrator::PackageOutcome;
use crate::policy::ReviewPolicy;
use crate::review_pipeline::evaluate_install_request;

pub fn evaluate_npm_install_request<R: EcosystemReleaseResolver + ?Sized>(
    request: &InstallRequest,
    resolver: &R,
    policy: &ReviewPolicy,
    now: SystemTime,
) -> Vec<PackageOutcome> {
    let decision_evaluator = NpmReleaseDecisionEvaluator::new(policy);

    evaluate_install_request(request, resolver, &decision_evaluator, now)
}
