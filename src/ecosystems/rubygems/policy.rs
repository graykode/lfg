use std::time::SystemTime;

use crate::core::decide_resolved_releases_by_publish_time;
use crate::core::ResolvedPackageReleases;
use crate::core::{ReleaseDecisionError, ReleaseDecisionEvaluator};
use crate::core::{ReviewDecision, ReviewPolicy};

#[derive(Debug, Clone, Copy)]
pub struct RubyReleaseDecisionEvaluator<'a> {
    policy: &'a ReviewPolicy,
}

impl<'a> RubyReleaseDecisionEvaluator<'a> {
    pub const fn new(policy: &'a ReviewPolicy) -> Self {
        Self { policy }
    }
}

impl ReleaseDecisionEvaluator for RubyReleaseDecisionEvaluator<'_> {
    fn id(&self) -> &'static str {
        "ruby-release-policy"
    }

    fn decide(
        &self,
        releases: &ResolvedPackageReleases,
        now: SystemTime,
    ) -> Result<ReviewDecision, ReleaseDecisionError> {
        decide_resolved_releases_by_publish_time(self.policy, releases, now)
    }
}
