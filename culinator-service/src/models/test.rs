use super::*;
#[test]
fn health_response_serializes_camel_case_fields() {
    let value = serde_json::to_value(HealthResponse {
        status: "ok",
        service: "culinator",
        api_version: "v1",
    })
    .expect("serialize");
    assert_eq!(value["apiVersion"], "v1");
}
