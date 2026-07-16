use super::*;
#[test]
fn bootstrap_serializes_expected_fields() {
    let value = ServiceBootstrap {
        endpoint: "http://127.0.0.1:1".into(),
        websocket_url: "ws://127.0.0.1:1/ws".into(),
        token: "secret".into(),
    };
    let json = serde_json::to_value(value).expect("serialize");
    assert_eq!(json["websocketUrl"], "ws://127.0.0.1:1/ws");
}
