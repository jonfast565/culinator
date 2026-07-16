use super::*;
use axum::{http::StatusCode, response::IntoResponse};
#[test]
fn not_found_maps_to_404() {
    assert_eq!(
        ApiError::not_found("missing").into_response().status(),
        StatusCode::NOT_FOUND
    );
}
