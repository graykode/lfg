use crate::core::Verdict;
use crate::core::{AskReason, ReviewDecision, SkipReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewUnavailableReason {
    RegistryFailure,
    ProviderFailure,
    ProviderTimeout,
    DiffFailure,
    MalformedProviderOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageOutcome {
    Skipped(SkipReason),
    PolicyAsk(AskReason),
    ReviewUnavailable(ReviewUnavailableReason),
    ProviderVerdict(Verdict),
}

impl PackageOutcome {
    pub const fn from_policy_decision(decision: ReviewDecision) -> Option<Self> {
        match decision {
            ReviewDecision::Review => None,
            ReviewDecision::Skip(reason) => Some(Self::Skipped(reason)),
            ReviewDecision::Ask(reason) => Some(Self::PolicyAsk(reason)),
        }
    }
}

pub fn aggregate_verdicts(outcomes: &[PackageOutcome]) -> Verdict {
    let mut has_ask = false;

    for outcome in outcomes {
        match outcome {
            PackageOutcome::ProviderVerdict(Verdict::Block) => return Verdict::Block,
            PackageOutcome::ProviderVerdict(Verdict::Ask)
            | PackageOutcome::PolicyAsk(_)
            | PackageOutcome::ReviewUnavailable(_) => has_ask = true,
            PackageOutcome::ProviderVerdict(Verdict::Pass) | PackageOutcome::Skipped(_) => {}
        }
    }

    if has_ask {
        Verdict::Ask
    } else {
        Verdict::Pass
    }
}

pub fn has_provider_pass(outcomes: &[PackageOutcome]) -> bool {
    outcomes
        .iter()
        .any(|outcome| matches!(outcome, PackageOutcome::ProviderVerdict(Verdict::Pass)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Verdict;
    use crate::core::{AskReason, ReviewDecision, SkipReason};

    #[test]
    fn skipped_review_passes_install() {
        let verdict =
            aggregate_verdicts(&[PackageOutcome::Skipped(SkipReason::OlderThanThreshold)]);

        assert_eq!(verdict, Verdict::Pass);
    }

    #[test]
    fn policy_ask_returns_ask() {
        let verdict =
            aggregate_verdicts(&[PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn provider_failure_returns_ask() {
        let verdict = aggregate_verdicts(&[PackageOutcome::ReviewUnavailable(
            ReviewUnavailableReason::ProviderFailure,
        )]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn malformed_provider_output_returns_ask() {
        let verdict = aggregate_verdicts(&[PackageOutcome::ReviewUnavailable(
            ReviewUnavailableReason::MalformedProviderOutput,
        )]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn provider_ask_returns_ask() {
        let verdict = aggregate_verdicts(&[PackageOutcome::ProviderVerdict(Verdict::Ask)]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn diff_failure_returns_ask() {
        let verdict = aggregate_verdicts(&[PackageOutcome::ReviewUnavailable(
            ReviewUnavailableReason::DiffFailure,
        )]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn provider_block_blocks_install() {
        let verdict = aggregate_verdicts(&[PackageOutcome::ProviderVerdict(Verdict::Block)]);

        assert_eq!(verdict, Verdict::Block);
    }

    #[test]
    fn detects_provider_pass_outcome() {
        assert!(has_provider_pass(&[PackageOutcome::ProviderVerdict(
            Verdict::Pass
        )]));
        assert!(!has_provider_pass(&[PackageOutcome::Skipped(
            SkipReason::OlderThanThreshold
        )]));
    }

    #[test]
    fn any_block_wins_over_ask_and_pass() {
        let verdict = aggregate_verdicts(&[
            PackageOutcome::ProviderVerdict(Verdict::Pass),
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime),
            PackageOutcome::ProviderVerdict(Verdict::Block),
        ]);

        assert_eq!(verdict, Verdict::Block);
    }

    #[test]
    fn ask_wins_when_no_package_blocks() {
        let verdict = aggregate_verdicts(&[
            PackageOutcome::Skipped(SkipReason::OlderThanThreshold),
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderTimeout),
        ]);

        assert_eq!(verdict, Verdict::Ask);
    }

    #[test]
    fn policy_skip_maps_to_package_outcome() {
        assert_eq!(
            PackageOutcome::from_policy_decision(ReviewDecision::Skip(
                SkipReason::OlderThanThreshold
            )),
            Some(PackageOutcome::Skipped(SkipReason::OlderThanThreshold))
        );
    }

    #[test]
    fn policy_ask_maps_to_package_outcome() {
        assert_eq!(
            PackageOutcome::from_policy_decision(ReviewDecision::Ask(
                AskReason::MissingTargetPublishTime
            )),
            Some(PackageOutcome::PolicyAsk(
                AskReason::MissingTargetPublishTime
            ))
        );
    }

    #[test]
    fn policy_review_requires_later_review_outcome() {
        assert_eq!(
            PackageOutcome::from_policy_decision(ReviewDecision::Review),
            None
        );
    }
}
