use crate::adapter::fixtures::StaticPackumentClient;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, InstallTarget, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::npm::NpmRegistryResolver;

#[test]
fn npm_registry_resolver_implements_common_contract() {
    let resolver = NpmRegistryResolver::new(StaticPackumentClient);

    assert_eq!(resolver.id(), "npm-registry");
    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "left-pad".to_owned()
        }),
        Ok(ResolvedPackageReleases {
            package_name: "left-pad".to_owned(),
            target: ResolvedPackageRelease {
                version: "1.1.0".to_owned(),
                published_at: "1970-01-02T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz".to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "1.0.0".to_owned(),
                published_at: "1970-01-01T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz".to_owned()
                },
            },
        })
    );
}

#[test]
fn npm_registry_resolver_maps_fetch_failures_to_common_resolve_error() {
    let resolver = NpmRegistryResolver::new(StaticPackumentClient);

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "missing".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable("missing".to_owned()))
    );
}
