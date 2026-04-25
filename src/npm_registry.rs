use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmRelease {
    pub version: String,
    pub published_at: String,
    pub tarball_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNpmReleases {
    pub package_name: String,
    pub target: NpmRelease,
    pub previous: NpmRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpmResolveError {
    InvalidPackument,
    MissingLatestDistTag,
    MissingTargetVersion(String),
    MissingPublishTime(String),
    MissingTarball(String),
    MissingPreviousRelease,
}

pub fn resolve_packument_releases(
    packument_json: &str,
    target_spec: &str,
) -> Result<ResolvedNpmReleases, NpmResolveError> {
    let packument: Value =
        serde_json::from_str(packument_json).map_err(|_| NpmResolveError::InvalidPackument)?;
    let (spec_package_name, explicit_version) = split_npm_spec(target_spec);
    let package_name = packument
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(spec_package_name)
        .to_owned();
    let target_version = match explicit_version {
        Some(version) => version.to_owned(),
        None => packument
            .pointer("/dist-tags/latest")
            .and_then(Value::as_str)
            .ok_or(NpmResolveError::MissingLatestDistTag)?
            .to_owned(),
    };

    let target = read_release(&packument, &target_version)?;
    let previous = previous_release(&packument, &target)?;

    Ok(ResolvedNpmReleases {
        package_name,
        target,
        previous,
    })
}

fn split_npm_spec(spec: &str) -> (&str, Option<&str>) {
    if spec.starts_with('@') {
        let Some(scope_separator) = spec.find('/') else {
            return (spec, None);
        };
        let version_separator = spec[(scope_separator + 1)..]
            .find('@')
            .map(|index| scope_separator + 1 + index);

        return match version_separator {
            Some(index) => (&spec[..index], Some(&spec[(index + 1)..])),
            None => (spec, None),
        };
    }

    match spec.rfind('@') {
        Some(index) if index > 0 => (&spec[..index], Some(&spec[(index + 1)..])),
        _ => (spec, None),
    }
}

fn previous_release(packument: &Value, target: &NpmRelease) -> Result<NpmRelease, NpmResolveError> {
    let versions = packument
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| NpmResolveError::MissingTargetVersion(target.version.clone()))?;
    let mut previous_version: Option<&str> = None;
    let mut previous_published_at: Option<&str> = None;

    for version in versions.keys() {
        if version == &target.version {
            continue;
        }

        let published_at = publish_time(packument, version)?;
        if published_at >= target.published_at.as_str() {
            continue;
        }

        if previous_published_at.is_none_or(|current| published_at > current) {
            previous_version = Some(version);
            previous_published_at = Some(published_at);
        }
    }

    let version = previous_version.ok_or(NpmResolveError::MissingPreviousRelease)?;
    read_release(packument, version)
}

fn read_release(packument: &Value, version: &str) -> Result<NpmRelease, NpmResolveError> {
    let versions = packument
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| NpmResolveError::MissingTargetVersion(version.to_owned()))?;
    if !versions.contains_key(version) {
        return Err(NpmResolveError::MissingTargetVersion(version.to_owned()));
    }

    let published_at = publish_time(packument, version)?.to_owned();
    let tarball_url = packument
        .pointer(&format!("/versions/{version}/dist/tarball"))
        .and_then(Value::as_str)
        .ok_or_else(|| NpmResolveError::MissingTarball(version.to_owned()))?
        .to_owned();

    Ok(NpmRelease {
        version: version.to_owned(),
        published_at,
        tarball_url,
    })
}

fn publish_time<'a>(packument: &'a Value, version: &str) -> Result<&'a str, NpmResolveError> {
    packument
        .pointer(&format!("/time/{version}"))
        .and_then(Value::as_str)
        .ok_or_else(|| NpmResolveError::MissingPublishTime(version.to_owned()))
}
