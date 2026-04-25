use std::time::SystemTime;

use crate::core::decide_resolved_releases_by_publish_time;
use crate::core::ResolvedPackageReleases;
use crate::core::{ReleaseDecisionError, ReleaseDecisionEvaluator};
use crate::core::{ReviewDecision, ReviewPolicy};

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
    fn id(&self) -> &'static str {
        "npm-release-policy"
    }

    fn decide(
        &self,
        releases: &ResolvedPackageReleases,
        now: SystemTime,
    ) -> Result<ReviewDecision, ReleaseDecisionError> {
        decide_resolved_releases_by_publish_time(self.policy, releases, now)
    }
}
