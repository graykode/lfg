use serde_json::json;

use lfg::core::{
    AdapterProtocolArchiveRef, AdapterProtocolInstallOperation, AdapterProtocolInstallRequest,
    AdapterProtocolInstallTarget, AdapterProtocolRealCommand, AdapterProtocolRequest,
    AdapterProtocolResolvedPackageRelease, AdapterProtocolResolvedPackageReleases,
    AdapterProtocolResponse, ADAPTER_PROTOCOL_VERSION,
};

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
