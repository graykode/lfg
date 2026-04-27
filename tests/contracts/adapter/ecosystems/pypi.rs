use crate::adapter::fixtures::StaticProjectClient;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, InstallTarget, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::pypi::PypiRegistryResolver;

#[test]
fn pypi_registry_resolver_implements_common_contract() {
    let resolver = PypiRegistryResolver::new(StaticProjectClient);

    assert_eq!(resolver.id(), "pypi-registry");
    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "requests".to_owned()
        }),
        Ok(ResolvedPackageReleases {
            package_name: "requests".to_owned(),
            target: ResolvedPackageRelease {
                version: "2.32.3".to_owned(),
                published_at: "1970-01-02T00:00:00.000000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz"
                        .to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "2.32.2".to_owned(),
                published_at: "1970-01-01T00:00:00.000000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz"
                        .to_owned()
                },
            },
        })
    );
}

#[test]
fn pypi_registry_resolver_maps_fetch_failures_to_common_resolve_error() {
    let resolver = PypiRegistryResolver::new(StaticProjectClient);

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "missing".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable("missing".to_owned()))
    );
}
