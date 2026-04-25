use lfg::core::InstallTarget;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::pypi::{PypiFetchError, PypiProjectClient, PypiRegistryResolver};

const PROJECT: &str = r#"{
  "info": {
    "name": "requests",
    "version": "2.32.3"
  },
  "releases": {
    "2.32.1": [
      {
        "packagetype": "sdist",
        "url": "https://files.pythonhosted.org/packages/requests-2.32.1.tar.gz",
        "upload_time_iso_8601": "2026-04-21T00:00:00.000000Z"
      }
    ],
    "2.32.2": [
      {
        "packagetype": "bdist_wheel",
        "url": "https://files.pythonhosted.org/packages/requests-2.32.2-py3-none-any.whl",
        "upload_time_iso_8601": "2026-04-22T00:00:00.000000Z"
      },
      {
        "packagetype": "sdist",
        "url": "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz",
        "upload_time_iso_8601": "2026-04-22T00:00:00.000000Z"
      }
    ],
    "2.32.3": [
      {
        "packagetype": "sdist",
        "url": "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz",
        "upload_time_iso_8601": "2026-04-23T00:00:00.000000Z"
      }
    ]
  }
}"#;

#[test]
fn resolves_latest_target_and_previous_pypi_sdist_release() {
    assert_eq!(
        resolve_fixture("requests", PROJECT, "requests"),
        Ok(ResolvedPackageReleases {
            package_name: "requests".to_owned(),
            target: ResolvedPackageRelease {
                version: "2.32.3".to_owned(),
                published_at: "2026-04-23T00:00:00.000000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz"
                        .to_owned(),
                },
            },
            previous: ResolvedPackageRelease {
                version: "2.32.2".to_owned(),
                published_at: "2026-04-22T00:00:00.000000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz"
                        .to_owned(),
                },
            },
        })
    );
}

#[test]
fn explicit_pinned_version_overrides_latest_project_version() {
    let resolved = resolve_fixture("requests", PROJECT, "requests==2.32.2")
        .expect("explicit version should resolve");

    assert_eq!(resolved.target.version, "2.32.2");
    assert_eq!(resolved.previous.version, "2.32.1");
}

#[test]
fn resolver_fetches_project_by_package_name_without_version_or_extras() {
    let resolver = PypiRegistryResolver::new(StaticProjectClient {
        expected_package_name: "requests".to_owned(),
        project: PROJECT.to_owned(),
    });

    let resolved = resolver
        .resolve(&InstallTarget {
            spec: "requests[security]==2.32.3".to_owned(),
        })
        .expect("target should resolve");

    assert_eq!(resolved.package_name, "requests");
    assert_eq!(resolved.target.version, "2.32.3");
}

#[test]
fn reports_missing_previous_release() {
    let project = r#"{
      "info": { "name": "single", "version": "1.0.0" },
      "releases": {
        "1.0.0": [
          {
            "packagetype": "sdist",
            "url": "https://files.pythonhosted.org/packages/single-1.0.0.tar.gz",
            "upload_time_iso_8601": "2026-04-21T00:00:00.000000Z"
          }
        ]
      }
    }"#;

    assert_eq!(
        resolve_fixture("single", project, "single"),
        Err(ResolveError::MissingPreviousRelease)
    );
}

#[test]
fn reports_missing_target_version() {
    assert_eq!(
        resolve_fixture("requests", PROJECT, "requests==9.9.9"),
        Err(ResolveError::MissingTargetVersion("9.9.9".to_owned()))
    );
}

#[test]
fn reports_range_requirement_as_unresolved_target_version() {
    let resolver = PypiRegistryResolver::new(StaticProjectClient {
        expected_package_name: "requests".to_owned(),
        project: PROJECT.to_owned(),
    });

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "requests<3".to_owned()
        }),
        Err(ResolveError::MissingTargetVersion("requests<3".to_owned()))
    );
}

#[test]
fn reports_non_source_release_as_missing_tarball() {
    let project = r#"{
      "info": { "name": "wheel-only", "version": "1.0.0" },
      "releases": {
        "1.0.0": [
          {
            "packagetype": "bdist_wheel",
            "url": "https://files.pythonhosted.org/packages/wheel_only-1.0.0.whl",
            "upload_time_iso_8601": "2026-04-21T00:00:00.000000Z"
          }
        ]
      }
    }"#;

    assert_eq!(
        resolve_fixture("wheel-only", project, "wheel-only"),
        Err(ResolveError::MissingTarball("1.0.0".to_owned()))
    );
}

#[test]
fn resolver_maps_fetch_errors() {
    let resolver = PypiRegistryResolver::new(StaticProjectClient {
        expected_package_name: "other-package".to_owned(),
        project: PROJECT.to_owned(),
    });

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "requests".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable(
            "unexpected package name: requests".to_owned()
        ))
    );
}

struct StaticProjectClient {
    expected_package_name: String,
    project: String,
}

impl PypiProjectClient for StaticProjectClient {
    fn fetch_project(&self, package_name: &str) -> Result<String, PypiFetchError> {
        if package_name != self.expected_package_name {
            return Err(PypiFetchError::Unavailable(format!(
                "unexpected package name: {package_name}"
            )));
        }

        Ok(self.project.clone())
    }
}

fn resolve_fixture(
    package_name: &str,
    project: &str,
    target_spec: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    PypiRegistryResolver::new(StaticProjectClient {
        expected_package_name: package_name.to_owned(),
        project: project.to_owned(),
    })
    .resolve(&InstallTarget {
        spec: target_spec.to_owned(),
    })
}
