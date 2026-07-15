use super::*;
use axum::http::HeaderValue;
#[test]
fn origin_policy_is_exact() {
    let p = AccessPolicy::new("secret", ["tauri://localhost".to_owned()]);
    assert!(p.is_origin_allowed(Some(&HeaderValue::from_static("tauri://localhost"))));
    assert!(!p.is_origin_allowed(Some(&HeaderValue::from_static("http://localhost:1420"))));
}
#[test]
fn missing_origin_can_be_enabled() {
    let p = AccessPolicy::new("secret", Vec::<String>::new()).allow_missing_origin(true);
    assert!(p.is_origin_allowed(None));
}
#[test]
fn constant_time_comparison_checks_length_and_value() {
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"a", b"aa"));
}
