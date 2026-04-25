use serde_json::Value;

use crate::core::InstallTarget;
use crate::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RubyGemsFetchError {
    Unavailable(String),
}

pub trait RubyGemsVersionsClient {
    fn fetch_versions(&self, gem_name: &str) -> Result<String, RubyGemsFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyGemsHttpVersionsClient {
    registry_base_url: String,
}

impl RubyGemsHttpVersionsClient {
    pub fn new(registry_base_url: impl Into<String>) -> Self {
        Self {
            registry_base_url: registry_base_url.into(),
        }
    }
}

impl RubyGemsVersionsClient for RubyGemsHttpVersionsClient {
    fn fetch_versions(&self, gem_name: &str) -> Result<String, RubyGemsFetchError> {
        let url = format!(
            "{}/api/v1/versions/{}.json",
            self.registry_base_url.trim_end_matches('/'),
            encode_gem_name_for_registry_path(gem_name)
        );

        ureq::get(&url)
            .header("Accept", "application/json")
            .call()
            .map_err(|error| RubyGemsFetchError::Unavailable(error.to_string()))?
            .body_mut()
            .read_to_string()
            .map_err(|error| RubyGemsFetchError::Unavailable(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyGemsRegistryResolver<C> {
    client: C,
    registry_base_url: String,
}

impl<C> RubyGemsRegistryResolver<C> {
    pub fn new(client: C, registry_base_url: impl Into<String>) -> Self {
        Self {
            client,
            registry_base_url: registry_base_url.into(),
        }
    }
}

impl<C: RubyGemsVersionsClient> EcosystemReleaseResolver for RubyGemsRegistryResolver<C> {
    fn id(&self) -> &'static str {
        "rubygems-registry"
    }

    fn resolve(&self, target: &InstallTarget) -> Result<ResolvedPackageReleases, ResolveError> {
        let spec = parse_gem_spec(&target.spec);
        if spec.gem_name.is_empty() {
            return Err(ResolveError::MissingTargetVersion(target.spec.clone()));
        }

        let versions = self
            .client
            .fetch_versions(spec.gem_name)
            .map_err(|error| match error {
                RubyGemsFetchError::Unavailable(message) => {
                    ResolveError::RegistryUnavailable(message)
                }
            })?;

        resolve_gem_releases(&versions, &target.spec, &self.registry_base_url)
    }
}

fn resolve_gem_releases(
    versions_json: &str,
    target_spec: &str,
    registry_base_url: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    let versions: Value =
        serde_json::from_str(versions_json).map_err(|_| ResolveError::InvalidMetadata)?;
    let spec = parse_gem_spec(target_spec);
    if spec.gem_name.is_empty() {
        return Err(ResolveError::MissingTargetVersion(target_spec.to_owned()));
    }
    let gem_name = spec.gem_name.to_owned();
    let target_version = match spec.explicit_version {
        Some(version) => version.to_owned(),
        None => versions
            .as_array()
            .and_then(|releases| releases.first())
            .and_then(|release| release.get("number"))
            .and_then(Value::as_str)
            .ok_or(ResolveError::MissingLatestDistTag)?
            .to_owned(),
    };

    let target = read_release(&versions, &gem_name, &target_version, registry_base_url)?;
    let previous = previous_release(&versions, &gem_name, &target, registry_base_url)?;

    Ok(ResolvedPackageReleases {
        package_name: gem_name,
        target,
        previous,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GemSpec<'a> {
    gem_name: &'a str,
    explicit_version: Option<&'a str>,
}

fn parse_gem_spec(spec: &str) -> GemSpec<'_> {
    match spec.rfind('@') {
        Some(index) if index > 0 => GemSpec {
            gem_name: spec[..index].trim(),
            explicit_version: Some(spec[(index + 1)..].trim()),
        },
        _ => GemSpec {
            gem_name: spec.trim(),
            explicit_version: None,
        },
    }
}

fn encode_gem_name_for_registry_path(gem_name: &str) -> String {
    gem_name.replace('/', "%2F")
}

fn previous_release(
    versions: &Value,
    gem_name: &str,
    target: &ResolvedPackageRelease,
    registry_base_url: &str,
) -> Result<ResolvedPackageRelease, ResolveError> {
    let releases = versions
        .as_array()
        .ok_or_else(|| ResolveError::MissingTargetVersion(target.version.clone()))?;
    let mut previous: Option<ResolvedPackageRelease> = None;

    for release in releases {
        if release.get("number").and_then(Value::as_str) == Some(target.version.as_str()) {
            continue;
        }

        let Some(version_number) = release.get("number").and_then(Value::as_str) else {
            continue;
        };
        let release = read_release(versions, gem_name, version_number, registry_base_url)?;
        if release.published_at >= target.published_at {
            continue;
        }

        if previous
            .as_ref()
            .is_none_or(|current| release.published_at > current.published_at)
        {
            previous = Some(release);
        }
    }

    previous.ok_or(ResolveError::MissingPreviousRelease)
}

fn read_release(
    versions: &Value,
    gem_name: &str,
    version: &str,
    registry_base_url: &str,
) -> Result<ResolvedPackageRelease, ResolveError> {
    let release = versions
        .as_array()
        .and_then(|releases| {
            releases
                .iter()
                .find(|candidate| candidate.get("number").and_then(Value::as_str) == Some(version))
        })
        .ok_or_else(|| ResolveError::MissingTargetVersion(version.to_owned()))?;
    let published_at = release
        .get("created_at")
        .or_else(|| release.get("built_at"))
        .and_then(Value::as_str)
        .ok_or_else(|| ResolveError::MissingPublishTime(version.to_owned()))?
        .to_owned();
    let archive_url = format!(
        "{}/gems/{}-{}.gem",
        registry_base_url.trim_end_matches('/'),
        encode_gem_name_for_registry_path(gem_name),
        version
    );

    Ok(ResolvedPackageRelease {
        version: version.to_owned(),
        published_at,
        archive: ArchiveRef { url: archive_url },
    })
}
