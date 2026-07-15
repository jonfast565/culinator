use super::*;
#[test]
fn required_string_reads_fields() {
    let value = serde_json::json!({"id":"abc"});
    assert_eq!(required_string(&value, "id").expect("string"), "abc");
}
#[test]
fn required_number_rejects_missing_fields() {
    assert!(required_f64(&serde_json::json!({}), "mass").is_err());
}
