use crate::adapter::fixtures::StaticVersionsClient;
use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, InstallTarget, ResolveError, ResolvedPackageRelease,
    ResolvedPackageReleases,
};
use lfg::ecosystems::rubygems::RubyGemsRegistryResolver;

#[test]
fn rubygems_registry_resolver_implements_common_contract() {
    let resolver = RubyGemsRegistryResolver::new(StaticVersionsClient, "https://rubygems.org");

    assert_eq!(resolver.id(), "rubygems-registry");
    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "rack".to_owned()
        }),
        Ok(ResolvedPackageReleases {
            package_name: "rack".to_owned(),
            target: ResolvedPackageRelease {
                version: "3.0.0".to_owned(),
                published_at: "1970-01-02T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://rubygems.org/gems/rack-3.0.0.gem".to_owned()
                },
            },
            previous: ResolvedPackageRelease {
                version: "2.2.0".to_owned(),
                published_at: "1970-01-01T00:00:00.000Z".to_owned(),
                archive: ArchiveRef {
                    url: "https://rubygems.org/gems/rack-2.2.0.gem".to_owned()
                },
            },
        })
    );
}

#[test]
fn rubygems_registry_resolver_maps_fetch_failures_to_common_resolve_error() {
    let resolver = RubyGemsRegistryResolver::new(StaticVersionsClient, "https://rubygems.org");

    assert_eq!(
        resolver.resolve(&InstallTarget {
            spec: "missing".to_owned()
        }),
        Err(ResolveError::RegistryUnavailable("missing".to_owned()))
    );
}
