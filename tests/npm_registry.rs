use lfg::install_request::InstallTarget;
use lfg::npm_registry::{
    resolve_packument_releases, NpmFetchError, NpmPackumentClient, NpmRegistryError,
    NpmRegistryResolver, NpmRelease, NpmResolveError, ResolvedNpmReleases,
};

const PACKUMENT: &str = r#"{
  "name": "left-pad",
  "dist-tags": {
    "latest": "1.1.0"
  },
  "time": {
    "created": "2026-04-20T00:00:00.000Z",
    "modified": "2026-04-24T00:00:00.000Z",
    "1.0.0": "2026-04-21T00:00:00.000Z",
    "1.2.0": "2026-04-22T00:00:00.000Z",
    "1.1.0": "2026-04-23T00:00:00.000Z"
  },
  "versions": {
    "1.0.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz"
      }
    },
    "1.1.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz"
      }
    },
    "1.2.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.2.0.tgz"
      }
    }
  }
}"#;

#[test]
fn resolves_latest_target_and_previous_published_release() {
    assert_eq!(
        resolve_packument_releases(PACKUMENT, "left-pad"),
        Ok(ResolvedNpmReleases {
            package_name: "left-pad".to_owned(),
            target: NpmRelease {
                version: "1.1.0".to_owned(),
                published_at: "2026-04-23T00:00:00.000Z".to_owned(),
                tarball_url: "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz".to_owned(),
            },
            previous: NpmRelease {
                version: "1.2.0".to_owned(),
                published_at: "2026-04-22T00:00:00.000Z".to_owned(),
                tarball_url: "https://registry.npmjs.org/left-pad/-/left-pad-1.2.0.tgz".to_owned(),
            },
        })
    );
}

#[test]
fn explicit_version_overrides_latest_dist_tag() {
    let resolved = resolve_packument_releases(PACKUMENT, "left-pad@1.2.0")
        .expect("explicit version should resolve");

    assert_eq!(resolved.target.version, "1.2.0");
    assert_eq!(resolved.previous.version, "1.0.0");
}

#[test]
fn reports_missing_previous_release() {
    let packument = r#"{
      "name": "one-version",
      "dist-tags": { "latest": "1.0.0" },
      "time": { "1.0.0": "2026-04-21T00:00:00.000Z" },
      "versions": {
        "1.0.0": {
          "dist": {
            "tarball": "https://registry.npmjs.org/one-version/-/one-version-1.0.0.tgz"
          }
        }
      }
    }"#;

    assert_eq!(
        resolve_packument_releases(packument, "one-version"),
        Err(NpmResolveError::MissingPreviousRelease)
    );
}

#[test]
fn reports_missing_target_version() {
    assert_eq!(
        resolve_packument_releases(PACKUMENT, "left-pad@9.9.9"),
        Err(NpmResolveError::MissingTargetVersion("9.9.9".to_owned()))
    );
}

struct StaticPackumentClient {
    expected_package_name: String,
    packument: String,
}

impl NpmPackumentClient for StaticPackumentClient {
    fn fetch_packument(&self, package_name: &str) -> Result<String, NpmFetchError> {
        if package_name != self.expected_package_name {
            return Err(NpmFetchError::Unavailable(format!(
                "unexpected package name: {package_name}"
            )));
        }

        Ok(self.packument.clone())
    }
}

#[test]
fn resolver_fetches_packument_by_package_name_and_resolves_target_spec() {
    let resolver = NpmRegistryResolver::new(StaticPackumentClient {
        expected_package_name: "left-pad".to_owned(),
        packument: PACKUMENT.to_owned(),
    });

    let resolved = resolver
        .resolve_target(&InstallTarget {
            spec: "left-pad@1.2.0".to_owned(),
        })
        .expect("target should resolve");

    assert_eq!(resolved.target.version, "1.2.0");
    assert_eq!(resolved.previous.version, "1.0.0");
}

#[test]
fn resolver_uses_scoped_package_name_without_version_for_fetch() {
    let scoped_packument = PACKUMENT.replace("\"left-pad\"", "\"@scope/pkg\"");
    let resolver = NpmRegistryResolver::new(StaticPackumentClient {
        expected_package_name: "@scope/pkg".to_owned(),
        packument: scoped_packument,
    });

    let resolved = resolver
        .resolve_target(&InstallTarget {
            spec: "@scope/pkg@1.1.0".to_owned(),
        })
        .expect("scoped target should resolve");

    assert_eq!(resolved.package_name, "@scope/pkg");
    assert_eq!(resolved.target.version, "1.1.0");
}

#[test]
fn resolver_maps_fetch_errors() {
    let resolver = NpmRegistryResolver::new(StaticPackumentClient {
        expected_package_name: "other-package".to_owned(),
        packument: PACKUMENT.to_owned(),
    });

    assert_eq!(
        resolver.resolve_target(&InstallTarget {
            spec: "left-pad".to_owned()
        }),
        Err(NpmRegistryError::Fetch(NpmFetchError::Unavailable(
            "unexpected package name: left-pad".to_owned()
        )))
    );
}
