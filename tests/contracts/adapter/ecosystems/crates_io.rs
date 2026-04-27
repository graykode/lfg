use crate::adapter::fixtures::StaticCrateClient;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, InstallTarget, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::crates_io::CratesIoRegistryResolver;

#[test]
fn crates_io_registry_resolver_implements_common_contract() {
    let resolver = CratesIoRegistryResolver::new(StaticCrateClient, "https://crates.io");

    assert_eq!(resolver.id(), "crates-io-registry");
    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "serde".to_owned()
        }),
        Ok(ResolvedPackageReleases {
            package_name: "serde".to_owned(),
            target: ResolvedPackageRelease {
                version: "1.0.1".to_owned(),
                published_at: "1970-01-02T00:00:00+00:00".to_owned(),
                archive: ArchiveRef {
                    url: "https://crates.io/api/v1/crates/serde/1.0.1/download".to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "1.0.0".to_owned(),
                published_at: "1970-01-01T00:00:00+00:00".to_owned(),
                archive: ArchiveRef {
                    url: "https://crates.io/api/v1/crates/serde/1.0.0/download".to_owned()
                },
            },
        })
    );
}

#[test]
fn crates_io_registry_resolver_maps_fetch_failures_to_common_resolve_error() {
    let resolver = CratesIoRegistryResolver::new(StaticCrateClient, "https://crates.io");

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "missing".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable("missing".to_owned()))
    );
}
