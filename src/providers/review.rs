use crate::core::{
    PackageOutcome, ProviderReviewOutcome, ReleaseReviewer, ResolvedPackageReleases,
    ReviewUnavailableReason,
};
use crate::evidence::{ArchiveDiffBuilder, ArchiveFetcher, DiffEngine};
use crate::providers::{
    parse_provider_output, write_provider_review_log, DiffReviewPromptBuilder, PromptBuilder,
    ReviewPrompt,
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

impl<T> ReviewProvider for Box<T>
where
    T: ReviewProvider + ?Sized,
{
    fn id(&self) -> &'static str {
        self.as_ref().id()
    }

    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        self.as_ref().review(prompt)
    }
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
                        let log_path = write_provider_review_log(
                            releases,
                            self.provider.id(),
                            &prompt,
                            &output,
                            &review,
                        )
                        .ok()
                        .flatten();
                        PackageOutcome::ProviderReview(ProviderReviewOutcome {
                            package_name: releases.package_name.clone(),
                            version: releases.target.version.clone(),
                            provider_id: self.provider.id().to_owned(),
                            verdict: review.verdict,
                            reason: review.reason,
                            log_path,
                        })
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
