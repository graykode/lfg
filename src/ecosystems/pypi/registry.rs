use serde_json::Value;

use crate::core::InstallTarget;
use crate::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PypiFetchError {
    Unavailable(String),
}

pub trait PypiProjectClient {
    fn fetch_project(&self, package_name: &str) -> Result<String, PypiFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PypiHttpProjectClient {
    registry_base_url: String,
}

impl PypiHttpProjectClient {
    pub fn new(registry_base_url: impl Into<String>) -> Self {
        Self {
            registry_base_url: registry_base_url.into(),
        }
    }
}

impl PypiProjectClient for PypiHttpProjectClient {
    fn fetch_project(&self, package_name: &str) -> Result<String, PypiFetchError> {
        let url = format!(
            "{}/pypi/{}/json",
            self.registry_base_url.trim_end_matches('/'),
            encode_project_name_for_registry_path(package_name)
        );

        ureq::get(&url)
            .header("Accept", "application/json")
            .call()
            .map_err(|error| PypiFetchError::Unavailable(error.to_string()))?
            .body_mut()
            .read_to_string()
            .map_err(|error| PypiFetchError::Unavailable(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PypiRegistryResolver<C> {
    client: C,
}

impl<C> PypiRegistryResolver<C> {
    pub const fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C: PypiProjectClient> EcosystemReleaseResolver for PypiRegistryResolver<C> {
    fn id(&self) -> &'static str {
        "pypi-registry"
    }

    fn resolve(&self, target: &InstallTarget) -> Result<ResolvedPackageReleases, ResolveError> {
        let spec = parse_pypi_spec(&target.spec);
        if spec.package_name.is_empty() || spec.has_unsupported_constraint {
            return Err(ResolveError::MissingTargetVersion(target.spec.clone()));
        }

        let project =
            self.client
                .fetch_project(spec.package_name)
                .map_err(|error| match error {
                    PypiFetchError::Unavailable(message) => {
                        ResolveError::RegistryUnavailable(message)
                    }
                })?;

        resolve_project_releases(&project, &target.spec)
    }
}

fn resolve_project_releases(
    project_json: &str,
    target_spec: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    let project: Value =
        serde_json::from_str(project_json).map_err(|_| ResolveError::InvalidMetadata)?;
    let spec = parse_pypi_spec(target_spec);
    if spec.package_name.is_empty() || spec.has_unsupported_constraint {
        return Err(ResolveError::MissingTargetVersion(target_spec.to_owned()));
    }
    let package_name = project
        .pointer("/info/name")
        .and_then(Value::as_str)
        .unwrap_or(spec.package_name)
        .to_owned();
    let target_version = match spec.explicit_version {
        Some(version) => version.to_owned(),
        None => project
            .pointer("/info/version")
            .and_then(Value::as_str)
            .ok_or(ResolveError::MissingLatestDistTag)?
            .to_owned(),
    };

    let target = read_release(&project, &target_version)?;
    let previous = previous_release(&project, &target)?;

    Ok(ResolvedPackageReleases {
        package_name,
        target,
        previous,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PypiSpec<'a> {
    package_name: &'a str,
    explicit_version: Option<&'a str>,
    has_unsupported_constraint: bool,
}

fn parse_pypi_spec(spec: &str) -> PypiSpec<'_> {
    let spec = spec.split(';').next().unwrap_or(spec).trim();
    let (name, explicit_version, has_unsupported_constraint) = match spec.find("==") {
        Some(index) => (&spec[..index], Some(spec[(index + 2)..].trim()), false),
        None => {
            let version_operator = spec.find(['<', '>', '!', '~', '=']);
            let name_end = version_operator.unwrap_or(spec.len());
            (&spec[..name_end], None, version_operator.is_some())
        }
    };
    let name = name.split('[').next().unwrap_or(name).trim();

    PypiSpec {
        package_name: name,
        explicit_version,
        has_unsupported_constraint,
    }
}

fn encode_project_name_for_registry_path(package_name: &str) -> String {
    package_name.replace('/', "%2F")
}

fn previous_release(
    project: &Value,
    target: &ResolvedPackageRelease,
) -> Result<ResolvedPackageRelease, ResolveError> {
    let releases = project
        .get("releases")
        .and_then(Value::as_object)
        .ok_or_else(|| ResolveError::MissingTargetVersion(target.version.clone()))?;
    let mut previous: Option<ResolvedPackageRelease> = None;

    for version in releases.keys() {
        if version == &target.version {
            continue;
        }

        let release = read_release(project, version)?;
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

fn read_release(project: &Value, version: &str) -> Result<ResolvedPackageRelease, ResolveError> {
    let releases = project
        .get("releases")
        .and_then(Value::as_object)
        .ok_or_else(|| ResolveError::MissingTargetVersion(version.to_owned()))?;
    let files = releases
        .get(version)
        .and_then(Value::as_array)
        .ok_or_else(|| ResolveError::MissingTargetVersion(version.to_owned()))?;
    let sdist = files
        .iter()
        .find(|file| {
            file.get("packagetype").and_then(Value::as_str) == Some("sdist")
                && file
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(is_supported_source_archive_url)
        })
        .ok_or_else(|| ResolveError::MissingTarball(version.to_owned()))?;
    let published_at = sdist
        .get("upload_time_iso_8601")
        .and_then(Value::as_str)
        .ok_or_else(|| ResolveError::MissingPublishTime(version.to_owned()))?
        .to_owned();
    let archive_url = sdist
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| ResolveError::MissingTarball(version.to_owned()))?
        .to_owned();

    Ok(ResolvedPackageRelease {
        version: version.to_owned(),
        published_at,
        archive: ArchiveRef { url: archive_url },
    })
}

fn is_supported_source_archive_url(url: &str) -> bool {
    url.ends_with(".tar.gz") || url.ends_with(".tgz")
}
