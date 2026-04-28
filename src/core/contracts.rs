use crate::core::{InstallRequest, InstallTarget, RealCommand};

pub trait ManagerIntegrationAdapter {
    fn id(&self) -> &'static str;
    fn release_resolver_id(&self) -> &'static str;
    fn release_decision_evaluator_id(&self) -> &'static str;
    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError>;
    fn real_command(&self, request: &InstallRequest) -> RealCommand;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAdapterError {
    MissingCommand,
    MissingPackage,
    MissingRequirementsFile,
    InvalidManifest(String),
    ManifestUnavailable(String),
    RequirementsFileUnavailable(String),
    UnsupportedManagerOption(String),
    UnsupportedRequirement(String),
    UnsupportedCommand(String),
}

pub trait EcosystemReleaseResolver {
    fn id(&self) -> &'static str;
    fn resolve(&self, target: &InstallTarget) -> Result<ResolvedPackageReleases, ResolveError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    RegistryUnavailable(String),
    InvalidMetadata,
    MissingLatestDistTag,
    MissingTargetVersion(String),
    MissingPublishTime(String),
    MissingTarball(String),
    MissingPreviousRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveRef {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackageRelease {
    pub version: String,
    pub published_at: String,
    pub archive: ArchiveRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackageReleases {
    pub package_name: String,
    pub target: ResolvedPackageRelease,
    pub previous: ResolvedPackageRelease,
}
