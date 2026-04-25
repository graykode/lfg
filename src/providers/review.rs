use crate::core::{
    PackageOutcome, ReleaseReviewer, ResolvedPackageReleases, ReviewUnavailableReason,
};
use crate::evidence::{ArchiveDiffBuilder, ArchiveFetcher, DiffEngine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveDiffReviewer<F, D> {
    diff_builder: ArchiveDiffBuilder<F, D>,
}

impl<F, D> ArchiveDiffReviewer<F, D> {
    pub const fn new(fetcher: F, diff_engine: D) -> Self {
        Self {
            diff_builder: ArchiveDiffBuilder::new(fetcher, diff_engine),
        }
    }
}

impl<F, D> ReleaseReviewer for ArchiveDiffReviewer<F, D>
where
    F: ArchiveFetcher,
    D: DiffEngine,
{
    fn review(&self, releases: &ResolvedPackageReleases) -> PackageOutcome {
        match self.diff_builder.build(releases) {
            Ok(_diff) => {
                PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
            }
            Err(_) => PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure),
        }
    }
}
