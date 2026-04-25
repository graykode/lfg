use crate::core::{
    PackageOutcome, ReleaseReviewer, ResolvedPackageReleases, ReviewUnavailableReason,
};
use crate::evidence::{ArchiveDiffBuilder, ArchiveFetcher, DiffEngine};
use crate::providers::{
    parse_provider_output, DiffReviewPromptBuilder, PromptBuilder, ReviewPrompt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Failure(String),
    Timeout,
    Unavailable(String),
}

pub trait ReviewProvider {
    fn id(&self) -> &'static str;
    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnavailableReviewProvider;

impl ReviewProvider for UnavailableReviewProvider {
    fn id(&self) -> &'static str {
        "unavailable"
    }

    fn review(&self, _prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        Err(ProviderError::Unavailable(
            "no review provider is configured".to_owned(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveDiffReviewer<F, D, P = UnavailableReviewProvider> {
    diff_builder: ArchiveDiffBuilder<F, D>,
    prompt_builder: DiffReviewPromptBuilder,
    provider: P,
}

impl<F, D> ArchiveDiffReviewer<F, D> {
    pub const fn new(fetcher: F, diff_engine: D) -> Self {
        Self {
            diff_builder: ArchiveDiffBuilder::new(fetcher, diff_engine),
            prompt_builder: DiffReviewPromptBuilder,
            provider: UnavailableReviewProvider,
        }
    }
}

impl<F, D, P> ArchiveDiffReviewer<F, D, P> {
    pub const fn with_provider(fetcher: F, diff_engine: D, provider: P) -> Self {
        Self {
            diff_builder: ArchiveDiffBuilder::new(fetcher, diff_engine),
            prompt_builder: DiffReviewPromptBuilder,
            provider,
        }
    }
}

impl<F, D, P> ReleaseReviewer for ArchiveDiffReviewer<F, D, P>
where
    F: ArchiveFetcher,
    D: DiffEngine,
    P: ReviewProvider,
{
    fn review(&self, releases: &ResolvedPackageReleases) -> PackageOutcome {
        match self.diff_builder.build(releases) {
            Ok(diff) => {
                let prompt = self.prompt_builder.build(releases, &diff);

                match self.provider.review(&prompt) {
                    Ok(output) => {
                        let review = parse_provider_output(&output);
                        PackageOutcome::ProviderVerdict(review.verdict)
                    }
                    Err(ProviderError::Timeout) => {
                        PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderTimeout)
                    }
                    Err(ProviderError::Failure(_) | ProviderError::Unavailable(_)) => {
                        PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
                    }
                }
            }
            Err(_) => PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure),
        }
    }
}
