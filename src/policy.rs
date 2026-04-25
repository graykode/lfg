use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseFacts {
    pub target_age: Option<Duration>,
    pub has_previous_release: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewDecision {
    Review,
    Skip(SkipReason),
    Ask(AskReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    OlderThanThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskReason {
    MissingTargetPublishTime,
    MissingPreviousRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewPolicy {
    review_age_threshold: Duration,
}

impl Default for ReviewPolicy {
    fn default() -> Self {
        Self {
            review_age_threshold: Duration::from_secs(24 * 60 * 60),
        }
    }
}

impl ReviewPolicy {
    pub fn decide(&self, facts: &ReleaseFacts) -> ReviewDecision {
        let Some(target_age) = facts.target_age else {
            return ReviewDecision::Ask(AskReason::MissingTargetPublishTime);
        };

        if !facts.has_previous_release {
            return ReviewDecision::Ask(AskReason::MissingPreviousRelease);
        }

        if target_age <= self.review_age_threshold {
            ReviewDecision::Review
        } else {
            ReviewDecision::Skip(SkipReason::OlderThanThreshold)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn recent_release_requires_review() {
        let policy = ReviewPolicy::default();
        let facts = ReleaseFacts {
            target_age: Some(Duration::from_secs(60)),
            has_previous_release: true,
        };

        assert_eq!(policy.decide(&facts), ReviewDecision::Review);
    }

    #[test]
    fn old_release_skips_review() {
        let policy = ReviewPolicy::default();
        let facts = ReleaseFacts {
            target_age: Some(Duration::from_secs(25 * 60 * 60)),
            has_previous_release: true,
        };

        assert_eq!(
            policy.decide(&facts),
            ReviewDecision::Skip(SkipReason::OlderThanThreshold)
        );
    }

    #[test]
    fn missing_target_publish_time_returns_ask() {
        let policy = ReviewPolicy::default();
        let facts = ReleaseFacts {
            target_age: None,
            has_previous_release: true,
        };

        assert_eq!(
            policy.decide(&facts),
            ReviewDecision::Ask(AskReason::MissingTargetPublishTime)
        );
    }

    #[test]
    fn missing_previous_release_returns_ask() {
        let policy = ReviewPolicy::default();
        let facts = ReleaseFacts {
            target_age: Some(Duration::from_secs(60)),
            has_previous_release: false,
        };

        assert_eq!(
            policy.decide(&facts),
            ReviewDecision::Ask(AskReason::MissingPreviousRelease)
        );
    }
}
