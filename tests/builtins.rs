use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lfg::builtins::{
    built_in_manager_adapters, built_in_release_decision_evaluators, built_in_release_resolvers,
    AdapterConfig,
};
use lfg::core::{InstallTarget, ReviewPolicy};

fn serve_packument_once(packument: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let address = listener.local_addr().expect("read local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept one request");
        let mut buffer = [0; 2048];
        let read = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            packument.len(),
            packument
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });

    (format!("http://{address}"), handle)
}

#[test]
fn built_in_manager_registry_contains_npm_adapter() {
    let registry = built_in_manager_adapters().expect("built-in manager adapters register");

    assert_eq!(registry.available_ids(), vec!["npm"]);

    let adapter = registry.get("npm").expect("npm manager adapter");
    assert_eq!(adapter.id(), "npm");
    assert_eq!(adapter.release_resolver_id(), "npm-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "npm-release-policy"
    );

    let request = adapter
        .parse_install(&["install".to_owned(), "left-pad".to_owned()])
        .expect("parse npm install");
    assert_eq!(request.targets[0].spec, "left-pad");
}

#[test]
fn built_in_release_resolver_registry_contains_configured_npm_registry_resolver() {
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
    let (registry_base_url, server) = serve_packument_once(packument);
    let registry = built_in_release_resolvers(AdapterConfig {
        npm_registry_base_url: registry_base_url,
    })
    .expect("built-in release resolvers register");

    assert_eq!(registry.available_ids(), vec!["npm-registry"]);

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

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /left-pad HTTP/1.1\r\n"));
}

#[test]
fn built_in_release_decision_evaluator_registry_contains_npm_policy() {
    let policy = ReviewPolicy::default();
    let registry =
        built_in_release_decision_evaluators(&policy).expect("built-in evaluators register");

    assert_eq!(registry.available_ids(), vec!["npm-release-policy"]);

    let evaluator = registry
        .get("npm-release-policy")
        .expect("npm release decision evaluator");
    assert_eq!(evaluator.id(), "npm-release-policy");
}
