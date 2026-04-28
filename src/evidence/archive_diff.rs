use crate::core::{ArchiveRef, ResolvedPackageReleases};
use crate::evidence::{read_source_archive_tree, ArchiveError};
use crate::evidence::{DiffEngine, DiffError, SourceDiff};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveFetchError {
    Unavailable(String),
}

pub trait ArchiveFetcher {
    fn fetch(&self, archive: &ArchiveRef) -> Result<Vec<u8>, ArchiveFetchError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpArchiveFetcher;

impl ArchiveFetcher for HttpArchiveFetcher {
    fn fetch(&self, archive: &ArchiveRef) -> Result<Vec<u8>, ArchiveFetchError> {
        ureq::get(&archive.url)
            .call()
            .map_err(|error| ArchiveFetchError::Unavailable(error.to_string()))?
            .body_mut()
            .read_to_vec()
            .map_err(|error| ArchiveFetchError::Unavailable(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveDiffError {
    Fetch(ArchiveFetchError),
    Archive(ArchiveError),
    Diff(DiffError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveDiffBuilder<F, D> {
    fetcher: F,
    diff_engine: D,
}

impl<F, D> ArchiveDiffBuilder<F, D> {
    pub const fn new(fetcher: F, diff_engine: D) -> Self {
        Self {
            fetcher,
            diff_engine,
        }
    }
}

impl<F: ArchiveFetcher, D: DiffEngine> ArchiveDiffBuilder<F, D> {
    pub fn build(
        &self,
        releases: &ResolvedPackageReleases,
    ) -> Result<SourceDiff, ArchiveDiffError> {
        let previous_archive = self
            .fetcher
            .fetch(&releases.previous.archive)
            .map_err(ArchiveDiffError::Fetch)?;
        let target_archive = self
            .fetcher
            .fetch(&releases.target.archive)
            .map_err(ArchiveDiffError::Fetch)?;

        let previous_tree =
            read_source_archive_tree(&previous_archive, &releases.previous.archive.url)
                .map_err(ArchiveDiffError::Archive)?;
        let target_tree = read_source_archive_tree(&target_archive, &releases.target.archive.url)
            .map_err(ArchiveDiffError::Archive)?;

        self.diff_engine
            .diff(&previous_tree, &target_tree)
            .map_err(ArchiveDiffError::Diff)
    }
}
