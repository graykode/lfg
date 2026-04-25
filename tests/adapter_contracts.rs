use lfg::core::contracts::{
    ArchiveRef, EcosystemReleaseResolver, ManagerAdapterError, ManagerIntegrationAdapter,
    ResolveError, ResolvedPackageRelease, ResolvedPackageReleases,
};
use lfg::core::install_request::{InstallOperation, InstallTarget, PackageManager};
use lfg::managers::npm::adapter::NpmManagerAdapter;
use lfg::managers::npm::registry::{NpmFetchError, NpmPackumentClient, NpmRegistryResolver};

struct StaticPackumentClient;

impl NpmPackumentClient for StaticPackumentClient {
    fn fetch_packument(&self, package_name: &str) -> Result<String, NpmFetchError> {
        if package_name != "left-pad" {
            return Err(NpmFetchError::Unavailable(package_name.to_owned()));
        }

        Ok(r#"{
          "name": "left-pad",
          "dist-tags": { "latest": "1.1.0" },
          "time": {
            "1.0.0": "1970-01-01T00:00:00.000Z",
            "1.1.0": "1970-01-02T00:00:00.000Z"
          },
          "versions": {
            "1.0.0": {
              "dist": { "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz" }
            },
            "1.1.0": {
              "dist": { "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz" }
            }
          }
        }"#
        .to_owned())
    }
}

#[test]
fn npm_manager_adapter_implements_common_contract() {
    let adapter = NpmManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "left-pad".to_owned()])
        .expect("npm install should parse");

    assert_eq!(adapter.id(), "npm");
    assert_eq!(request.manager, PackageManager::Npm);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}

#[test]
fn npm_manager_adapter_uses_common_parse_errors() {
    let adapter = NpmManagerAdapter;

    assert_eq!(
        adapter.parse_install(&["run".to_owned(), "build".to_owned()]),
        Err(ManagerAdapterError::UnsupportedCommand("run".to_owned()))
    );
}

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
