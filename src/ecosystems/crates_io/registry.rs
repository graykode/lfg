use serde_json::Value;

use crate::core::InstallTarget;
use crate::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CratesIoFetchError {
    Unavailable(String),
}

pub trait CratesIoCrateClient {
    fn fetch_crate(&self, crate_name: &str) -> Result<String, CratesIoFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CratesIoHttpCrateClient {
    registry_base_url: String,
}

impl CratesIoHttpCrateClient {
    pub fn new(registry_base_url: impl Into<String>) -> Self {
        Self {
            registry_base_url: registry_base_url.into(),
        }
    }
}

impl CratesIoCrateClient for CratesIoHttpCrateClient {
    fn fetch_crate(&self, crate_name: &str) -> Result<String, CratesIoFetchError> {
        let url = format!(
            "{}/api/v1/crates/{}",
            self.registry_base_url.trim_end_matches('/'),
            encode_crate_name_for_registry_path(crate_name)
        );

        ureq::get(&url)
            .header("Accept", "application/json")
            .call()
            .map_err(|error| CratesIoFetchError::Unavailable(error.to_string()))?
            .body_mut()
            .read_to_string()
            .map_err(|error| CratesIoFetchError::Unavailable(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CratesIoRegistryResolver<C> {
    client: C,
    registry_base_url: String,
}

impl<C> CratesIoRegistryResolver<C> {
    pub fn new(client: C, registry_base_url: impl Into<String>) -> Self {
        Self {
            client,
            registry_base_url: registry_base_url.into(),
        }
    }
}

impl<C: CratesIoCrateClient> EcosystemReleaseResolver for CratesIoRegistryResolver<C> {
    fn id(&self) -> &'static str {
        "crates-io-registry"
    }

    fn resolve(&self, target: &InstallTarget) -> Result<ResolvedPackageReleases, ResolveError> {
        let spec = parse_cargo_crate_spec(&target.spec);
        if spec.crate_name.is_empty() {
            return Err(ResolveError::MissingTargetVersion(target.spec.clone()));
        }

        let metadata = self
            .client
            .fetch_crate(spec.crate_name)
            .map_err(|error| match error {
                CratesIoFetchError::Unavailable(message) => {
                    ResolveError::RegistryUnavailable(message)
                }
            })?;

        resolve_crate_releases(&metadata, &target.spec, &self.registry_base_url)
    }
}

fn resolve_crate_releases(
    metadata_json: &str,
    target_spec: &str,
    registry_base_url: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    let metadata: Value =
        serde_json::from_str(metadata_json).map_err(|_| ResolveError::InvalidMetadata)?;
    let spec = parse_cargo_crate_spec(target_spec);
    if spec.crate_name.is_empty() {
        return Err(ResolveError::MissingTargetVersion(target_spec.to_owned()));
    }
    let package_name = metadata
        .pointer("/crate/id")
        .and_then(Value::as_str)
        .unwrap_or(spec.crate_name)
        .to_owned();
    let target_version = match spec.explicit_version {
        Some(version) => version.to_owned(),
        None => metadata
            .pointer("/crate/max_version")
            .or_else(|| metadata.pointer("/crate/newest_version"))
            .and_then(Value::as_str)
            .ok_or(ResolveError::MissingLatestDistTag)?
            .to_owned(),
    };

    let target = read_release(&metadata, &package_name, &target_version, registry_base_url)?;
    let previous = previous_release(&metadata, &package_name, &target, registry_base_url)?;

    Ok(ResolvedPackageReleases {
        package_name,
        target,
        previous,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CargoCrateSpec<'a> {
    crate_name: &'a str,
    explicit_version: Option<&'a str>,
}

fn parse_cargo_crate_spec(spec: &str) -> CargoCrateSpec<'_> {
    match spec.rfind('@') {
        Some(index) if index > 0 => CargoCrateSpec {
            crate_name: spec[..index].trim(),
            explicit_version: Some(spec[(index + 1)..].trim()),
        },
        _ => CargoCrateSpec {
            crate_name: spec.trim(),
            explicit_version: None,
        },
    }
}

fn encode_crate_name_for_registry_path(crate_name: &str) -> String {
    crate_name.replace('/', "%2F")
}

fn previous_release(
    metadata: &Value,
    crate_name: &str,
    target: &ResolvedPackageRelease,
    registry_base_url: &str,
) -> Result<ResolvedPackageRelease, ResolveError> {
    let versions = metadata
        .get("versions")
        .and_then(Value::as_array)
        .ok_or_else(|| ResolveError::MissingTargetVersion(target.version.clone()))?;
    let mut previous: Option<ResolvedPackageRelease> = None;

    for version in versions {
        if version.get("num").and_then(Value::as_str) == Some(target.version.as_str()) {
            continue;
        }

        let Some(version_number) = version.get("num").and_then(Value::as_str) else {
            continue;
        };
        let release = read_release(metadata, crate_name, version_number, registry_base_url)?;
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
    metadata: &Value,
    crate_name: &str,
    version: &str,
    registry_base_url: &str,
) -> Result<ResolvedPackageRelease, ResolveError> {
    let version_metadata = metadata
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|versions| {
            versions
                .iter()
                .find(|candidate| candidate.get("num").and_then(Value::as_str) == Some(version))
        })
        .ok_or_else(|| ResolveError::MissingTargetVersion(version.to_owned()))?;
    let published_at = version_metadata
        .get("created_at")
        .and_then(Value::as_str)
        .ok_or_else(|| ResolveError::MissingPublishTime(version.to_owned()))?
        .to_owned();
    let archive_url = crate_download_url(version_metadata, crate_name, version, registry_base_url);

    Ok(ResolvedPackageRelease {
        version: version.to_owned(),
        published_at,
        archive: ArchiveRef { url: archive_url },
    })
}

fn crate_download_url(
    version_metadata: &Value,
    crate_name: &str,
    version: &str,
    registry_base_url: &str,
) -> String {
    if let Some(download_path) = version_metadata.get("dl_path").and_then(Value::as_str) {
        if download_path.starts_with("http://") || download_path.starts_with("https://") {
            return download_path.to_owned();
        }

        return format!(
            "{}{}",
            registry_base_url.trim_end_matches('/'),
            download_path
        );
    }

    format!(
        "{}/api/v1/crates/{}/{}/download",
        registry_base_url.trim_end_matches('/'),
        encode_crate_name_for_registry_path(crate_name),
        version
    )
}
