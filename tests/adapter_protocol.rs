use serde_json::json;

use lfg::core::{
    AdapterCapability, AdapterCapabilityKind, AdapterProtocolArchiveRef, AdapterProtocolError,
    AdapterProtocolErrorCode, AdapterProtocolInstallOperation, AdapterProtocolInstallRequest,
    AdapterProtocolInstallTarget, AdapterProtocolRealCommand, AdapterProtocolRequest,
    AdapterProtocolResolvedPackageRelease, AdapterProtocolResolvedPackageReleases,
    AdapterProtocolResponse, Verdict, ADAPTER_PROTOCOL_VERSION,
};
use lfg::ecosystems::npm::{NpmPackumentClient, NpmRegistryResolver};
use lfg::managers::npm::NpmManagerAdapter;

#[test]
fn handshake_and_capability_messages_have_stable_json_contract() {
    let request = AdapterProtocolRequest::handshake("0.1.0");

    assert_eq!(
        serde_json::to_value(request).expect("handshake serializes"),
        json!({
            "type": "handshake",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "lfg_version": "0.1.0"
        })
    );

    let response = AdapterProtocolResponse::handshake_accepted(
        "demo-adapter",
        vec![
            AdapterCapability::manager_integration("demo-manager"),
            AdapterCapability::ecosystem_release_resolver("demo-registry"),
            AdapterCapability::llm_adapter("demo-llm"),
        ],
    );

    assert_eq!(
        serde_json::to_value(response).expect("handshake response serializes"),
        json!({
            "type": "handshake-accepted",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "adapter_id": "demo-adapter",
            "capabilities": [
                { "kind": "manager-integration", "id": "demo-manager" },
                { "kind": "ecosystem-release-resolver", "id": "demo-registry" },
                { "kind": "llm-adapter", "id": "demo-llm" }
            ]
        })
    );

    assert_eq!(
        serde_json::to_value(AdapterProtocolRequest::capabilities())
            .expect("capability request serializes"),
        json!({
            "type": "capabilities",
            "protocol_version": ADAPTER_PROTOCOL_VERSION
        })
    );

    let capabilities = AdapterProtocolResponse::capabilities(vec![
        AdapterCapability::manager_integration("npm"),
        AdapterCapability::ecosystem_release_resolver("npm-registry"),
    ]);

    assert_eq!(
        serde_json::to_value(capabilities).expect("capability response serializes"),
        json!({
            "type": "capabilities",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "capabilities": [
                { "kind": "manager-integration", "id": "npm" },
                { "kind": "ecosystem-release-resolver", "id": "npm-registry" }
            ]
        })
    );
}

#[test]
fn parse_resolve_and_review_messages_have_stable_json_contracts() {
    let parse_request = AdapterProtocolRequest::ParseInstall {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        manager_id: "npm".to_owned(),
        args: vec!["install".to_owned(), "left-pad".to_owned()],
    };

    assert_eq!(
        serde_json::to_value(parse_request).expect("parse request serializes"),
        json!({
            "type": "parse-install",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "manager_id": "npm",
            "args": ["install", "left-pad"]
        })
    );

    let parsed = AdapterProtocolResponse::InstallParsed {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        request: AdapterProtocolInstallRequest {
            manager_id: "npm".to_owned(),
            operation: AdapterProtocolInstallOperation::Install,
            targets: vec![AdapterProtocolInstallTarget {
                spec: "left-pad".to_owned(),
            }],
            manager_args: vec!["install".to_owned(), "left-pad".to_owned()],
            release_resolver_id: "npm-registry".to_owned(),
            release_decision_evaluator_id: "npm-release-policy".to_owned(),
        },
        real_command: AdapterProtocolRealCommand {
            program: "npm".to_owned(),
            args: vec!["install".to_owned(), "left-pad".to_owned()],
        },
    };

    assert_eq!(
        serde_json::to_value(parsed).expect("parse response serializes"),
        json!({
            "type": "install-parsed",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "request": {
                "manager_id": "npm",
                "operation": "install",
                "targets": [{ "spec": "left-pad" }],
                "manager_args": ["install", "left-pad"],
                "release_resolver_id": "npm-registry",
                "release_decision_evaluator_id": "npm-release-policy"
            },
            "real_command": {
                "program": "npm",
                "args": ["install", "left-pad"]
            }
        })
    );

    let resolve_request = AdapterProtocolRequest::ResolveRelease {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        resolver_id: "npm-registry".to_owned(),
        target: AdapterProtocolInstallTarget {
            spec: "left-pad".to_owned(),
        },
    };

    assert_eq!(
        serde_json::to_value(resolve_request).expect("resolve request serializes"),
        json!({
            "type": "resolve-release",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "resolver_id": "npm-registry",
            "target": { "spec": "left-pad" }
        })
    );

    let release_response = AdapterProtocolResponse::ReleaseResolved {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        releases: AdapterProtocolResolvedPackageReleases {
            package_name: "left-pad".to_owned(),
            target: AdapterProtocolResolvedPackageRelease {
                version: "1.1.0".to_owned(),
                published_at: "1970-01-02T00:00:00.000Z".to_owned(),
                archive: AdapterProtocolArchiveRef {
                    url: "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz".to_owned(),
                },
            },
            previous: AdapterProtocolResolvedPackageRelease {
                version: "1.0.0".to_owned(),
                published_at: "1970-01-01T00:00:00.000Z".to_owned(),
                archive: AdapterProtocolArchiveRef {
                    url: "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz".to_owned(),
                },
            },
        },
    };

    assert_eq!(
        serde_json::to_value(release_response).expect("resolve response serializes"),
        json!({
            "type": "release-resolved",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "releases": {
                "package_name": "left-pad",
                "target": {
                    "version": "1.1.0",
                    "published_at": "1970-01-02T00:00:00.000Z",
                    "archive": {
                        "url": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz"
                    }
                },
                "previous": {
                    "version": "1.0.0",
                    "published_at": "1970-01-01T00:00:00.000Z",
                    "archive": {
                        "url": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz"
                    }
                }
            }
        })
    );

    let review_request = AdapterProtocolRequest::Review {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        provider_id: "codex-cli".to_owned(),
        prompt: "review this diff".to_owned(),
        timeout_seconds: 60,
    };

    assert_eq!(
        serde_json::to_value(review_request).expect("review request serializes"),
        json!({
            "type": "review",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "provider_id": "codex-cli",
            "prompt": "review this diff",
            "timeout_seconds": 60
        })
    );

    let review_response = AdapterProtocolResponse::ReviewCompleted {
        protocol_version: ADAPTER_PROTOCOL_VERSION,
        raw_output: "verdict: pass\nreason: reviewed\n".to_owned(),
    };

    assert_eq!(
        serde_json::to_value(review_response).expect("review response serializes"),
        json!({
            "type": "review-completed",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "raw_output": "verdict: pass\nreason: reviewed\n"
        })
    );
}

#[test]
fn built_in_contracts_can_be_described_as_protocol_capabilities() {
    let manager = NpmManagerAdapter;
    let resolver = NpmRegistryResolver::new(NeverPackumentClient);

    assert_eq!(
        AdapterCapability::from_manager_adapter(&manager),
        AdapterCapability {
            kind: AdapterCapabilityKind::ManagerIntegration,
            id: "npm".to_owned(),
        }
    );
    assert_eq!(
        AdapterCapability::from_release_resolver(&resolver),
        AdapterCapability {
            kind: AdapterCapabilityKind::EcosystemReleaseResolver,
            id: "npm-registry".to_owned(),
        }
    );
}

#[test]
fn external_adapter_failures_map_to_ask_responses() {
    let failure = AdapterProtocolError::new(
        AdapterProtocolErrorCode::Timeout,
        "external adapter timed out",
    );

    assert_eq!(failure.verdict(), Verdict::Ask);
    assert_eq!(
        serde_json::to_value(failure.into_response()).expect("error response serializes"),
        json!({
            "type": "error",
            "protocol_version": ADAPTER_PROTOCOL_VERSION,
            "code": "timeout",
            "message": "external adapter timed out",
            "ask": true
        })
    );
}

struct NeverPackumentClient;

impl NpmPackumentClient for NeverPackumentClient {
    fn fetch_packument(
        &self,
        _package_name: &str,
    ) -> Result<String, lfg::ecosystems::npm::NpmFetchError> {
        unreachable!("adapter protocol test does not fetch packuments")
    }
}
