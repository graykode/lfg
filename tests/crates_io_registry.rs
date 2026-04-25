use lfg::core::InstallTarget;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::crates_io::{
    CratesIoCrateClient, CratesIoFetchError, CratesIoRegistryResolver,
};

const CRATE_METADATA: &str = r#"{
  "crate": {
    "id": "serde",
    "max_version": "1.0.228"
  },
  "versions": [
    {
      "num": "1.0.228",
      "created_at": "2026-04-23T00:00:00+00:00",
      "dl_path": "/api/v1/crates/serde/1.0.228/download"
    },
    {
      "num": "1.0.227",
      "created_at": "2026-04-22T00:00:00+00:00",
      "dl_path": "/api/v1/crates/serde/1.0.227/download"
    },
    {
      "num": "1.0.226",
      "created_at": "2026-04-21T00:00:00+00:00",
      "dl_path": "/api/v1/crates/serde/1.0.226/download"
    }
  ]
}"#;

#[test]
fn resolves_latest_target_and_previous_crate_release() {
    assert_eq!(
        resolve_fixture("serde", CRATE_METADATA, "serde"),
        Ok(ResolvedPackageReleases {
            package_name: "serde".to_owned(),
            target: ResolvedPackageRelease {
                version: "1.0.228".to_owned(),
                published_at: "2026-04-23T00:00:00+00:00".to_owned(),
                archive: ArchiveRef {
                    url: "https://crates.io/api/v1/crates/serde/1.0.228/download".to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "1.0.227".to_owned(),
                published_at: "2026-04-22T00:00:00+00:00".to_owned(),
                archive: ArchiveRef {
                    url: "https://crates.io/api/v1/crates/serde/1.0.227/download".to_owned()
                },
            },
        })
    );
}

#[test]
fn explicit_pinned_version_overrides_latest_crate_version() {
    let resolved = resolve_fixture("serde", CRATE_METADATA, "serde@1.0.227")
        .expect("explicit version should resolve");

    assert_eq!(resolved.target.version, "1.0.227");
    assert_eq!(resolved.previous.version, "1.0.226");
}

#[test]
fn resolver_fetches_crate_by_name_without_version() {
    let resolver = CratesIoRegistryResolver::new(
        StaticCrateClient {
            expected_crate_name: "serde".to_owned(),
            metadata: CRATE_METADATA.to_owned(),
        },
        "https://crates.io",
    );

    let resolved = resolver
        .resolve(&InstallTarget {
            spec: "serde@1.0.228".to_owned(),
        })
        .expect("crate should resolve");

    assert_eq!(resolved.package_name, "serde");
    assert_eq!(resolved.target.version, "1.0.228");
}

#[test]
fn reports_missing_previous_release() {
    let metadata = r#"{
      "crate": { "id": "single", "max_version": "1.0.0" },
      "versions": [
        {
          "num": "1.0.0",
          "created_at": "2026-04-21T00:00:00+00:00",
          "dl_path": "/api/v1/crates/single/1.0.0/download"
        }
      ]
    }"#;

    assert_eq!(
        resolve_fixture("single", metadata, "single"),
        Err(ResolveError::MissingPreviousRelease)
    );
}

#[test]
fn reports_missing_target_version() {
    assert_eq!(
        resolve_fixture("serde", CRATE_METADATA, "serde@9.9.9"),
        Err(ResolveError::MissingTargetVersion("9.9.9".to_owned()))
    );
}

#[test]
fn resolver_maps_fetch_errors() {
    let resolver = CratesIoRegistryResolver::new(
        StaticCrateClient {
            expected_crate_name: "other-crate".to_owned(),
            metadata: CRATE_METADATA.to_owned(),
        },
        "https://crates.io",
    );

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "serde".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable(
            "unexpected crate name: serde".to_owned()
        ))
    );
}

struct StaticCrateClient {
    expected_crate_name: String,
    metadata: String,
}

impl CratesIoCrateClient for StaticCrateClient {
    fn fetch_crate(&self, crate_name: &str) -> Result<String, CratesIoFetchError> {
        if crate_name != self.expected_crate_name {
            return Err(CratesIoFetchError::Unavailable(format!(
                "unexpected crate name: {crate_name}"
            )));
        }

        Ok(self.metadata.clone())
    }
}

fn resolve_fixture(
    crate_name: &str,
    metadata: &str,
    target_spec: &str,
) -> Result<ResolvedPackageReleases, ResolveError> {
    CratesIoRegistryResolver::new(
        StaticCrateClient {
            expected_crate_name: crate_name.to_owned(),
            metadata: metadata.to_owned(),
        },
        "https://crates.io",
    )
    .resolve(&InstallTarget {
        spec: target_spec.to_owned(),
    })
}
