use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::{collections::BTreeSet, sync::Arc};

use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct AccessPolicy {
    token: Arc<str>,
    allowed_origins: Arc<BTreeSet<String>>,
    allow_missing_origin: bool,
}

impl AccessPolicy {
    pub fn new(
        token: impl Into<String>,
        allowed_origins: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            token: Arc::from(token.into()),
            allowed_origins: Arc::new(allowed_origins.into_iter().collect()),
            allow_missing_origin: false,
        }
    }

    pub fn allow_missing_origin(mut self, allow: bool) -> Self {
        self.allow_missing_origin = allow;
        self
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn is_origin_allowed(&self, origin: Option<&HeaderValue>) -> bool {
        match origin.and_then(|value| value.to_str().ok()) {
            Some(value) => self.allowed_origins.contains(value),
            None => self.allow_missing_origin,
        }
    }
}

pub async fn require_local_client(
    State(policy): State<AccessPolicy>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let origin = request.headers().get(header::ORIGIN);
    if !policy.is_origin_allowed(origin) {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "Origin is not allowed",
        ));
    }

    if request.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let expected = format!("Bearer {}", policy.token());
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| constant_time_eq(value.as_bytes(), expected.as_bytes()));

    if !authorized {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Missing or invalid launch token",
        ));
    }

    Ok(next.run(request).await)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}
#[cfg(test)]
mod test;
