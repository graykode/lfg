use std::env;
use std::path::Path;
use std::time::Duration;

use crate::core::{EcosystemReleaseResolver, ManagerIntegrationAdapter};
use crate::core::{Registry, RegistryError};
use crate::core::{ReleaseDecisionEvaluator, ReviewPolicy};
use crate::ecosystems::crates_io::{
    CratesIoHttpCrateClient, CratesIoRegistryResolver, RustReleaseDecisionEvaluator,
};
use crate::ecosystems::npm::{
    NpmHttpPackumentClient, NpmRegistryResolver, NpmReleaseDecisionEvaluator,
};
use crate::ecosystems::pypi::{
    PypiHttpProjectClient, PypiRegistryResolver, PythonReleaseDecisionEvaluator,
};
use crate::managers::cargo::CargoManagerAdapter;
use crate::managers::npm::NpmManagerAdapter;
use crate::managers::pip::PipManagerAdapter;
use crate::managers::pnpm::PnpmManagerAdapter;
use crate::managers::uv::UvManagerAdapter;
use crate::managers::yarn::YarnManagerAdapter;
use crate::providers::{CommandReviewProvider, ReviewProvider, UnavailableReviewProvider};

pub type ManagerAdapterRegistry = Registry<Box<dyn ManagerIntegrationAdapter>>;
pub type ReleaseResolverRegistry = Registry<Box<dyn EcosystemReleaseResolver>>;
pub type ReleaseDecisionEvaluatorRegistry<'a> = Registry<Box<dyn ReleaseDecisionEvaluator + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfig {
    pub crates_io_registry_base_url: String,
    pub npm_registry_base_url: String,
    pub pypi_registry_base_url: String,
}

impl AdapterConfig {
    pub fn from_env() -> Self {
        Self {
            crates_io_registry_base_url: env::var("LFG_CRATES_IO_REGISTRY_URL")
                .unwrap_or_else(|_| "https://crates.io".to_owned()),
            npm_registry_base_url: env::var("LFG_NPM_REGISTRY_URL")
                .unwrap_or_else(|_| "https://registry.npmjs.org".to_owned()),
            pypi_registry_base_url: env::var("LFG_PYPI_REGISTRY_URL")
                .unwrap_or_else(|_| "https://pypi.org".to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyConfig {
    pub review_age_threshold: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyConfigError {
    InvalidReviewAgeThreshold,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            review_age_threshold: Duration::from_secs(24 * 60 * 60),
        }
    }
}

impl PolicyConfig {
    pub fn from_env() -> Result<Self, PolicyConfigError> {
        let mut config = Self::default();

        if let Some(threshold) = configured_review_age_threshold()? {
            config.review_age_threshold = threshold;
        }

        Ok(config)
    }

    pub fn review_policy(&self) -> ReviewPolicy {
        ReviewPolicy::new(self.review_age_threshold)
    }
}

fn configured_review_age_threshold() -> Result<Option<Duration>, PolicyConfigError> {
    let value = match env::var("LFG_REVIEW_AGE_THRESHOLD_SECONDS") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(PolicyConfigError::InvalidReviewAgeThreshold);
        }
    };

    let seconds = value
        .parse::<u64>()
        .map_err(|_| PolicyConfigError::InvalidReviewAgeThreshold)?;
    if seconds == 0 {
        return Err(PolicyConfigError::InvalidReviewAgeThreshold);
    }

    Ok(Some(Duration::from_secs(seconds)))
}

pub trait ProgramDetector {
    fn is_available(&self, program: &str) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewProviderPreference {
    Auto,
    None,
    ClaudeCli,
    CodexCli,
}

impl ReviewProviderPreference {
    fn from_env() -> Self {
        match env::var("LFG_REVIEW_PROVIDER").ok().as_deref() {
            Some("auto") | Some("") | None => Self::Auto,
            Some("claude") | Some("claude-cli") => Self::ClaudeCli,
            Some("codex") | Some("codex-cli") => Self::CodexCli,
            Some("none") => Self::None,
            Some(_) => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathProgramDetector;

impl ProgramDetector for PathProgramDetector {
    fn is_available(&self, program: &str) -> bool {
        let program_path = Path::new(program);
        if program_path.components().count() > 1 {
            return program_path.is_file();
        }

        env::var_os("PATH")
            .into_iter()
            .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
            .any(|path| path.join(program).is_file())
    }
}

pub fn built_in_manager_adapters() -> Result<ManagerAdapterRegistry, RegistryError> {
    let mut registry = Registry::new();
    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(CargoManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(NpmManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(PipManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(PnpmManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(UvManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    let adapter: Box<dyn ManagerIntegrationAdapter> = Box::new(YarnManagerAdapter);
    let id = adapter.id();

    registry.register(id, adapter)?;

    Ok(registry)
}

pub fn built_in_release_decision_evaluators<'a>(
    policy: &'a ReviewPolicy,
) -> Result<ReleaseDecisionEvaluatorRegistry<'a>, RegistryError> {
    let mut registry = Registry::new();
    let evaluator: Box<dyn ReleaseDecisionEvaluator + 'a> =
        Box::new(NpmReleaseDecisionEvaluator::new(policy));
    let id = evaluator.id();

    registry.register(id, evaluator)?;

    let evaluator: Box<dyn ReleaseDecisionEvaluator + 'a> =
        Box::new(PythonReleaseDecisionEvaluator::new(policy));
    let id = evaluator.id();

    registry.register(id, evaluator)?;

    let evaluator: Box<dyn ReleaseDecisionEvaluator + 'a> =
        Box::new(RustReleaseDecisionEvaluator::new(policy));
    let id = evaluator.id();

    registry.register(id, evaluator)?;

    Ok(registry)
}

pub fn built_in_review_provider(detector: &dyn ProgramDetector) -> Box<dyn ReviewProvider> {
    built_in_review_provider_with_preference(ReviewProviderPreference::from_env(), detector)
}

pub fn built_in_review_provider_with_preference(
    preference: ReviewProviderPreference,
    detector: &dyn ProgramDetector,
) -> Box<dyn ReviewProvider> {
    match preference {
        ReviewProviderPreference::Auto => {
            if detector.is_available("claude") {
                claude_review_provider()
            } else if detector.is_available("codex") {
                codex_review_provider()
            } else {
                unavailable_review_provider()
            }
        }
        ReviewProviderPreference::None => unavailable_review_provider(),
        ReviewProviderPreference::ClaudeCli => {
            if detector.is_available("claude") {
                claude_review_provider()
            } else {
                unavailable_review_provider()
            }
        }
        ReviewProviderPreference::CodexCli => {
            if detector.is_available("codex") {
                codex_review_provider()
            } else {
                unavailable_review_provider()
            }
        }
    }
}

fn claude_review_provider() -> Box<dyn ReviewProvider> {
    Box::new(CommandReviewProvider::new(
        "claude-cli",
        "claude",
        [
            "-p",
            "--output-format",
            "text",
            "--no-session-persistence",
            "--tools",
            "",
        ],
    ))
}

fn codex_review_provider() -> Box<dyn ReviewProvider> {
    Box::new(CommandReviewProvider::new(
        "codex-cli",
        "codex",
        [
            "exec",
            "--skip-git-repo-check",
            "--sandbox",
            "read-only",
            "-",
        ],
    ))
}

fn unavailable_review_provider() -> Box<dyn ReviewProvider> {
    Box::new(UnavailableReviewProvider)
}

pub fn built_in_release_resolvers(
    config: AdapterConfig,
) -> Result<ReleaseResolverRegistry, RegistryError> {
    let mut registry = Registry::new();
    let resolver: Box<dyn EcosystemReleaseResolver> = Box::new(CratesIoRegistryResolver::new(
        CratesIoHttpCrateClient::new(config.crates_io_registry_base_url.clone()),
        config.crates_io_registry_base_url,
    ));
    let id = resolver.id();

    registry.register(id, resolver)?;

    let resolver: Box<dyn EcosystemReleaseResolver> = Box::new(NpmRegistryResolver::new(
        NpmHttpPackumentClient::new(config.npm_registry_base_url),
    ));
    let id = resolver.id();

    registry.register(id, resolver)?;

    let resolver: Box<dyn EcosystemReleaseResolver> = Box::new(PypiRegistryResolver::new(
        PypiHttpProjectClient::new(config.pypi_registry_base_url),
    ));
    let id = resolver.id();

    registry.register(id, resolver)?;

    Ok(registry)
}
