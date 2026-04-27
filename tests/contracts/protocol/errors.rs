use serde_json::json;

use lfg::core::{
    AdapterProtocolError, AdapterProtocolErrorCode, Verdict, ADAPTER_PROTOCOL_VERSION,
};

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
