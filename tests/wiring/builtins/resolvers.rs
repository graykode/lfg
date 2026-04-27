use super::support::serve_json_once;
use lfg::builtins::{built_in_release_resolvers, AdapterConfig};
use lfg::core::InstallTarget;

#[test]
fn built_in_release_resolver_registry_contains_configured_ecosystem_resolvers() {
    let packument = r#"{
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
    }"#;
    let project = r#"{
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
    }"#;
    let crate_metadata = r#"{
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
    }"#;
    let gem_versions = r#"[
      {
        "number": "3.0.0",
        "created_at": "1970-01-02T00:00:00.000Z"
      },
      {
        "number": "2.2.0",
        "created_at": "1970-01-01T00:00:00.000Z"
      }
    ]"#;
    let (crates_io_registry_base_url, crates_io_server) = serve_json_once(crate_metadata);
    let (rubygems_registry_base_url, rubygems_server) = serve_json_once(gem_versions);
    let (npm_registry_base_url, npm_server) = serve_json_once(packument);
    let (pypi_registry_base_url, pypi_server) = serve_json_once(project);
    let registry = built_in_release_resolvers(AdapterConfig {
        crates_io_registry_base_url,
        npm_registry_base_url,
        pypi_registry_base_url,
        rubygems_registry_base_url,
    })
    .expect("built-in release resolvers register");

    assert_eq!(
        registry.available_ids(),
        vec![
            "crates-io-registry",
            "npm-registry",
            "pypi-registry",
            "rubygems-registry"
        ]
    );

    let resolver = registry
        .get("crates-io-registry")
        .expect("crates.io registry resolver");
    assert_eq!(resolver.id(), "crates-io-registry");

    let releases = resolver
        .resolve(&InstallTarget {
            spec: "serde".to_owned(),
        })
        .expect("resolve crate release");

    assert_eq!(releases.package_name, "serde");
    assert_eq!(releases.target.version, "1.0.1");
    assert_eq!(releases.previous.version, "1.0.0");

    let resolver = registry
        .get("rubygems-registry")
        .expect("RubyGems registry resolver");
    assert_eq!(resolver.id(), "rubygems-registry");

    let releases = resolver
        .resolve(&InstallTarget {
            spec: "rack".to_owned(),
        })
        .expect("resolve gem release");

    assert_eq!(releases.package_name, "rack");
    assert_eq!(releases.target.version, "3.0.0");
    assert_eq!(releases.previous.version, "2.2.0");

    let resolver = registry.get("npm-registry").expect("npm registry resolver");
    assert_eq!(resolver.id(), "npm-registry");

    let releases = resolver
        .resolve(&InstallTarget {
            spec: "left-pad".to_owned(),
        })
        .expect("resolve npm release");

    assert_eq!(releases.package_name, "left-pad");
    assert_eq!(releases.target.version, "1.1.0");
    assert_eq!(releases.previous.version, "1.0.0");

    let resolver = registry
        .get("pypi-registry")
        .expect("pypi registry resolver");
    assert_eq!(resolver.id(), "pypi-registry");

    let releases = resolver
        .resolve(&InstallTarget {
            spec: "requests".to_owned(),
        })
        .expect("resolve pypi release");

    assert_eq!(releases.package_name, "requests");
    assert_eq!(releases.target.version, "2.32.3");
    assert_eq!(releases.previous.version, "2.32.2");

    let request = npm_server.join().expect("npm server thread completes");
    assert!(request.starts_with("GET /left-pad HTTP/1.1\r\n"));
    let request = pypi_server.join().expect("pypi server thread completes");
    assert!(request.starts_with("GET /pypi/requests/json HTTP/1.1\r\n"));
    let request = crates_io_server
        .join()
        .expect("crates.io server thread completes");
    assert!(request.starts_with("GET /api/v1/crates/serde HTTP/1.1\r\n"));
    let request = rubygems_server
        .join()
        .expect("RubyGems server thread completes");
    assert!(request.starts_with("GET /api/v1/versions/rack.json HTTP/1.1\r\n"));
}
