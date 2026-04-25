use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lfg::builtins::{
    built_in_manager_adapters, built_in_release_decision_evaluators, built_in_release_resolvers,
    built_in_review_provider, built_in_review_provider_with_preference, AdapterConfig,
    ProgramDetector, ReviewProviderPreference,
};
use lfg::core::{InstallTarget, ReviewPolicy};
use lfg::providers::ReviewProvider;

fn serve_json_once(body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let address = listener.local_addr().expect("read local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept one request");
        let mut buffer = [0; 2048];
        let read = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });

    (format!("http://{address}"), handle)
}

#[test]
fn built_in_manager_registry_contains_manager_adapters() {
    let registry = built_in_manager_adapters().expect("built-in manager adapters register");

    assert_eq!(registry.available_ids(), vec!["npm", "pip", "uv"]);

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

    let adapter = registry.get("pip").expect("pip manager adapter");
    assert_eq!(adapter.id(), "pip");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );

    let request = adapter
        .parse_install(&["install".to_owned(), "requests".to_owned()])
        .expect("parse pip install");
    assert_eq!(request.targets[0].spec, "requests");

    let adapter = registry.get("uv").expect("uv manager adapter");
    assert_eq!(adapter.id(), "uv");
    assert_eq!(adapter.release_resolver_id(), "pypi-registry");
    assert_eq!(
        adapter.release_decision_evaluator_id(),
        "python-release-policy"
    );

    let request = adapter
        .parse_install(&["add".to_owned(), "requests".to_owned()])
        .expect("parse uv add");
    assert_eq!(request.targets[0].spec, "requests");
}

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
    let (npm_registry_base_url, npm_server) = serve_json_once(packument);
    let (pypi_registry_base_url, pypi_server) = serve_json_once(project);
    let registry = built_in_release_resolvers(AdapterConfig {
        npm_registry_base_url,
        pypi_registry_base_url,
    })
    .expect("built-in release resolvers register");

    assert_eq!(
        registry.available_ids(),
        vec!["npm-registry", "pypi-registry"]
    );

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
}

#[test]
fn built_in_release_decision_evaluator_registry_contains_release_policies() {
    let policy = ReviewPolicy::default();
    let registry =
        built_in_release_decision_evaluators(&policy).expect("built-in evaluators register");

    assert_eq!(
        registry.available_ids(),
        vec!["npm-release-policy", "python-release-policy"]
    );

    let evaluator = registry
        .get("npm-release-policy")
        .expect("npm release decision evaluator");
    assert_eq!(evaluator.id(), "npm-release-policy");

    let evaluator = registry
        .get("python-release-policy")
        .expect("python release decision evaluator");
    assert_eq!(evaluator.id(), "python-release-policy");
}

#[derive(Debug, Clone, Copy)]
struct StaticProgramDetector {
    claude: bool,
    codex: bool,
}

impl ProgramDetector for StaticProgramDetector {
    fn is_available(&self, program: &str) -> bool {
        match program {
            "claude" => self.claude,
            "codex" => self.codex,
            _ => false,
        }
    }
}

#[test]
fn built_in_review_provider_prefers_claude_before_codex() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: true,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "claude-cli");
}

#[test]
fn built_in_review_provider_uses_codex_when_claude_is_missing() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: false,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "codex-cli");
}

#[test]
fn built_in_review_provider_returns_unavailable_when_no_local_provider_exists() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::Auto,
        &StaticProgramDetector {
            claude: false,
            codex: false,
        },
    );

    assert_eq!(provider.id(), "unavailable");
}

#[test]
fn configured_review_provider_returns_unavailable_when_missing() {
    let provider = built_in_review_provider_with_preference(
        ReviewProviderPreference::ClaudeCli,
        &StaticProgramDetector {
            claude: false,
            codex: true,
        },
    );

    assert_eq!(provider.id(), "unavailable");
}

#[test]
fn built_in_review_provider_can_be_disabled_by_env() {
    let previous = env::var_os("LFG_REVIEW_PROVIDER");
    env::set_var("LFG_REVIEW_PROVIDER", "none");

    let provider = built_in_review_provider(&StaticProgramDetector {
        claude: true,
        codex: true,
    });

    match previous {
        Some(value) => env::set_var("LFG_REVIEW_PROVIDER", value),
        None => env::remove_var("LFG_REVIEW_PROVIDER"),
    }

    assert_eq!(provider.id(), "unavailable");
}
