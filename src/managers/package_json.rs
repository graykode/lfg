use std::collections::BTreeSet;
use std::fs;

use serde_json::Value;

use crate::core::{InstallTarget, ManagerAdapterError};

const PACKAGE_JSON_PATH: &str = "package.json";
const DEPENDENCY_SECTIONS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

pub(crate) fn package_json_install_targets() -> Result<Vec<InstallTarget>, ManagerAdapterError> {
    let content = fs::read_to_string(PACKAGE_JSON_PATH)
        .map_err(|_| ManagerAdapterError::ManifestUnavailable(PACKAGE_JSON_PATH.to_owned()))?;

    parse_package_json_install_targets(&content)
}

pub(crate) fn parse_package_json_install_targets(
    content: &str,
) -> Result<Vec<InstallTarget>, ManagerAdapterError> {
    let manifest: Value = serde_json::from_str(content)
        .map_err(|_| ManagerAdapterError::InvalidManifest(PACKAGE_JSON_PATH.to_owned()))?;
    let mut specs = BTreeSet::new();

    for section in DEPENDENCY_SECTIONS {
        let Some(dependencies) = manifest.get(*section) else {
            continue;
        };
        let dependencies = dependencies
            .as_object()
            .ok_or_else(|| ManagerAdapterError::InvalidManifest(PACKAGE_JSON_PATH.to_owned()))?;

        for (package_name, requirement) in dependencies {
            let requirement = requirement.as_str().ok_or_else(|| {
                ManagerAdapterError::InvalidManifest(PACKAGE_JSON_PATH.to_owned())
            })?;
            let Some(spec) = package_json_dependency_spec(package_name, requirement)? else {
                continue;
            };
            specs.insert(spec);
        }
    }

    Ok(specs
        .into_iter()
        .map(|spec| InstallTarget { spec })
        .collect())
}

fn package_json_dependency_spec(
    package_name: &str,
    requirement: &str,
) -> Result<Option<String>, ManagerAdapterError> {
    let requirement = requirement.trim();
    if requirement.is_empty() || requirement == "*" {
        return Ok(Some(package_name.to_owned()));
    }

    if is_local_dependency(requirement) {
        return Ok(None);
    }

    if let Some(alias) = requirement.strip_prefix("npm:") {
        return Ok(Some(npm_alias_dependency_spec(alias)));
    }

    if is_remote_dependency(requirement) {
        return Err(ManagerAdapterError::UnsupportedRequirement(format!(
            "{package_name}@{requirement}"
        )));
    }

    if is_exact_registry_version(requirement) {
        return Ok(Some(format!("{package_name}@{requirement}")));
    }

    Ok(Some(package_name.to_owned()))
}

fn is_local_dependency(requirement: &str) -> bool {
    requirement.starts_with("file:")
        || requirement.starts_with("link:")
        || requirement.starts_with("workspace:")
        || requirement.starts_with("portal:")
        || requirement.starts_with("patch:")
        || requirement.starts_with('.')
        || requirement.starts_with('/')
}

fn is_remote_dependency(requirement: &str) -> bool {
    requirement.starts_with("git:")
        || requirement.starts_with("git+")
        || requirement.starts_with("github:")
        || requirement.starts_with("http:")
        || requirement.starts_with("https:")
        || requirement.starts_with("ssh:")
        || requirement.contains("://")
}

fn is_exact_registry_version(requirement: &str) -> bool {
    requirement
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
        && !requirement.contains(['^', '~', '<', '>', '=', '*', 'x', 'X', '|', ' '])
}

fn npm_alias_dependency_spec(alias: &str) -> String {
    let (package_name, version) = split_npm_spec(alias);
    match version {
        Some(version) if is_exact_registry_version(version) => {
            format!("{package_name}@{version}")
        }
        _ => package_name.to_owned(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_registry_dependencies_from_package_json() {
        let targets = parse_package_json_install_targets(
            r#"{
              "dependencies": {
                "left-pad": "1.3.0",
                "react": "^18.2.0"
              },
              "devDependencies": {
                "@scope/tool": "~2.0.0"
              },
              "optionalDependencies": {
                "alias": "npm:real-package@1.2.3"
              }
            }"#,
        )
        .expect("package.json should parse");

        assert_eq!(
            targets,
            vec![
                InstallTarget {
                    spec: "@scope/tool".to_owned(),
                },
                InstallTarget {
                    spec: "left-pad@1.3.0".to_owned(),
                },
                InstallTarget {
                    spec: "react".to_owned(),
                },
                InstallTarget {
                    spec: "real-package@1.2.3".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn skips_local_package_json_dependencies() {
        let targets = parse_package_json_install_targets(
            r#"{
              "dependencies": {
                "workspace-lib": "workspace:*",
                "local-lib": "file:../local-lib",
                "link-lib": "link:../link-lib"
              }
            }"#,
        )
        .expect("package.json should parse");

        assert!(targets.is_empty());
    }

    #[test]
    fn rejects_remote_package_json_dependencies_that_skip_registry_metadata() {
        assert_eq!(
            parse_package_json_install_targets(
                r#"{
                  "dependencies": {
                    "remote-lib": "github:owner/repo"
                  }
                }"#,
            ),
            Err(ManagerAdapterError::UnsupportedRequirement(
                "remote-lib@github:owner/repo".to_owned()
            ))
        );
    }
}
