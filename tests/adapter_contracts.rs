use lfg::core::{
    ArchiveRef, EcosystemReleaseResolver, ManagerAdapterError, ManagerIntegrationAdapter,
    ResolveError, ResolvedPackageRelease, ResolvedPackageReleases,
};
use lfg::core::{InstallOperation, InstallTarget, PackageManager};
use lfg::ecosystems::crates_io::{
    CratesIoCrateClient, CratesIoFetchError, CratesIoRegistryResolver,
};
use lfg::ecosystems::npm::{NpmFetchError, NpmPackumentClient, NpmRegistryResolver};
use lfg::ecosystems::pypi::{PypiFetchError, PypiProjectClient, PypiRegistryResolver};
use lfg::ecosystems::rubygems::{
    RubyGemsFetchError, RubyGemsRegistryResolver, RubyGemsVersionsClient,
};
use lfg::managers::cargo::CargoManagerAdapter;
use lfg::managers::gem::GemManagerAdapter;
use lfg::managers::npm::NpmManagerAdapter;
use lfg::managers::pip::PipManagerAdapter;
use lfg::managers::pnpm::PnpmManagerAdapter;
use lfg::managers::uv::UvManagerAdapter;
use lfg::managers::yarn::YarnManagerAdapter;

struct StaticPackumentClient;

struct StaticCrateClient;

impl CratesIoCrateClient for StaticCrateClient {
    fn fetch_crate(&self, crate_name: &str) -> Result<String, CratesIoFetchError> {
        if crate_name != "serde" {
            return Err(CratesIoFetchError::Unavailable(crate_name.to_owned()));
        }

        Ok(r#"{
          "crate": { "id": "serde", "max_version": "1.0.1" },
          "versions": [
            {
              "num": "1.0.1",
              "created_at": "1970-01-02T00:00:00+00:00",
              "dl_path": "/api/v1/crates/serde/1.0.1/download"
            },
            {
              "num": "1.0.0",
              "created_at": "1970-01-01T00:00:00+00:00",
              "dl_path": "/api/v1/crates/serde/1.0.0/download"
            }
          ]
        }"#
        .to_owned())
    }
}

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

struct StaticProjectClient;

struct StaticVersionsClient;

impl RubyGemsVersionsClient for StaticVersionsClient {
    fn fetch_versions(&self, gem_name: &str) -> Result<String, RubyGemsFetchError> {
        if gem_name != "rack" {
            return Err(RubyGemsFetchError::Unavailable(gem_name.to_owned()));
        }

        Ok(r#"[
          {
            "number": "3.0.0",
            "created_at": "1970-01-02T00:00:00.000Z"
          },
          {
            "number": "2.2.0",
            "created_at": "1970-01-01T00:00:00.000Z"
          }
        ]"#
        .to_owned())
    }
}

impl PypiProjectClient for StaticProjectClient {
    fn fetch_project(&self, package_name: &str) -> Result<String, PypiFetchError> {
        if package_name != "requests" {
            return Err(PypiFetchError::Unavailable(package_name.to_owned()));
        }

        Ok(r#"{
          "info": { "name": "requests", "version": "2.32.3" },
          "releases": {
            "2.32.2": [
              {
                "packagetype": "sdist",
                "url": "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz",
                "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
              }
            ],
            "2.32.3": [
              {
                "packagetype": "sdist",
                "url": "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz",
                "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
              }
            ]
          }
        }"#
        .to_owned())
    }
}

#[test]
fn cargo_manager_adapter_implements_common_contract() {
    let adapter = CargoManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "serde".to_owned()])
        .expect("cargo add should parse");

    assert_eq!(adapter.id(), "cargo");
    assert_eq!(adapter.release_resolver_id(), "crates-io-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "rust-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Cargo);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "serde".to_owned()
        }]
    );
}

#[test]
fn gem_manager_adapter_implements_common_contract() {
    let adapter = GemManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "rack".to_owned()])
        .expect("gem install should parse");

    assert_eq!(adapter.id(), "gem");
    assert_eq!(adapter.release_resolver_id(), "rubygems-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "ruby-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Gem);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "rack".to_owned()
        }]
    );
}

#[test]
fn npm_manager_adapter_implements_common_contract() {
    let adapter = NpmManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "left-pad".to_owned()])
        .expect("npm install should parse");

    assert_eq!(adapter.id(), "npm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
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
fn pip_manager_adapter_implements_common_contract() {
    let adapter = PipManagerAdapter;

    let request = adapter
        .parse_install(&["install".to_owned(), "requests".to_owned()])
        .expect("pip install should parse");

    assert_eq!(adapter.id(), "pip");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Pip);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests".to_owned()
        }]
    );
}

#[test]
fn pnpm_manager_adapter_implements_common_contract() {
    let adapter = PnpmManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("pnpm add should parse");

    assert_eq!(adapter.id(), "pnpm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Pnpm);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}

#[test]
fn uv_manager_adapter_implements_common_contract() {
    let adapter = UvManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "requests".to_owned()])
        .expect("uv add should parse");

    assert_eq!(adapter.id(), "uv");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Uv);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests".to_owned()
        }]
    );
}

#[test]
fn yarn_manager_adapter_implements_common_contract() {
    let adapter = YarnManagerAdapter;

    let request = adapter
        .parse_install(&["add".to_owned(), "left-pad".to_owned()])
        .expect("yarn add should parse");

    assert_eq!(adapter.id(), "yarn");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );
    assert_eq!(request.manager, PackageManager::Yarn);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "left-pad".to_owned()
        }]
    );
}

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
