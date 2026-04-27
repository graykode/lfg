use serde_json::json;

use lfg::core::{
    AdapterCapability, AdapterProtocolRequest, AdapterProtocolResponse, ADAPTER_PROTOCOL_VERSION,
};

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
