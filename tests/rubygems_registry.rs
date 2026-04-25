use lfg::core::InstallTarget;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::rubygems::{
    RubyGemsFetchError, RubyGemsRegistryResolver, RubyGemsVersionsClient,
};

const VERSIONS: &str = r#"[
  {
    "number": "3.0.0",
    "created_at": "2026-04-23T00:00:00.000Z"
  },
  {
    "number": "2.2.0",
    "created_at": "2026-04-22T00:00:00.000Z"
  },
  {
    "number": "2.1.0",
    "created_at": "2026-04-21T00:00:00.000Z"
  }
]"#;

#[test]
fn resolves_latest_target_and_previous_gem_release() {
    assert_eq!(
        resolve_fixture("rack", VERSIONS, "rack"),
        Ok(ResolvedPackageReleases {
            package_name: "rack".to_owned(),
            target: ResolvedPackageRelease {
                version: "3.0.0".to_owned(),
                published_at: "2026-04-23T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://rubygems.org/gems/rack-3.0.0.gem".to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "2.2.0".to_owned(),
                published_at: "2026-04-22T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://rubygems.org/gems/rack-2.2.0.gem".to_owned()
                },
            },
        })
    );
}

#[test]
fn explicit_version_overrides_latest_gem_version() {
    let resolved =
        resolve_fixture("rack", VERSIONS, "rack@2.2.0").expect("explicit version should resolve");

    assert_eq!(resolved.target.version, "2.2.0");
    assert_eq!(resolved.previous.version, "2.1.0");
}

#[test]
fn resolver_fetches_gem_by_name_without_version() {
    let resolver = RubyGemsRegistryResolver::new(
        StaticVersionsClient {
            expected_gem_name: "rack".to_owned(),
            versions: VERSIONS.to_owned(),
        },
        "https://rubygems.org",
    );

    let resolved = resolver
        .resolve(&InstallTarget {
            spec: "rack@3.0.0".to_owned(),
        })
        .expect("gem should resolve");

    assert_eq!(resolved.package_name, "rack");
    assert_eq!(resolved.target.version, "3.0.0");
}

#[test]
fn reports_missing_previous_release() {
    let versions = r#"[
      {
        "number": "1.0.0",
        "created_at": "2026-04-21T00:00:00.000Z"
      }
    ]"#;

    assert_eq!(
        resolve_fixture("single", versions, "single"),
        Err(ResolveError::MissingPreviousRelease)
    );
}

#[test]
fn reports_missing_target_version() {
    assert_eq!(
        resolve_fixture("rack", VERSIONS, "rack@9.9.9"),
        Err(ResolveError::MissingTargetVersion("9.9.9".to_owned()))
    );
}

#[test]
fn resolver_maps_fetch_errors() {
    let resolver = RubyGemsRegistryResolver::new(
        StaticVersionsClient {
            expected_gem_name: "other-gem".to_owned(),
            versions: VERSIONS.to_owned(),
        },
        "https://rubygems.org",
    );

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "rack".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable(
            "unexpected gem name: rack".to_owned()
        ))
    );
}

struct StaticVersionsClient {
    expected_gem_name: String,
    versions: String,
}

impl RubyGemsVersionsClient for StaticVersionsClient {
    fn fetch_versions(&self, gem_name: &str) -> Result<String, RubyGemsFetchError> {
        if gem_name != self.expected_gem_name {
            return Err(RubyGemsFetchError::Unavailable(format!(
                "unexpected gem name: {gem_name}"
            )));
        }

        Ok(self.versions.clone())
    }
}

fn resolve_fixture(
    gem_name: &str,
    versions: &str,
    target_spec: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    RubyGemsRegistryResolver::new(
        StaticVersionsClient {
            expected_gem_name: gem_name.to_owned(),
            versions: versions.to_owned(),
        },
        "https://rubygems.org",
    )
    .resolve(&InstallTarget {
        spec: target_spec.to_owned(),
    })
}
