use axum::{Json, http::StatusCode, response::IntoResponse};
use culinator_application::ApplicationError;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
}

impl From<ApplicationError> for ApiError {
    fn from(value: ApplicationError) -> Self {
        let status = match value {
            ApplicationError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApplicationError::InvalidInput(_)
            | ApplicationError::Parse(_)
            | ApplicationError::Validation => StatusCode::BAD_REQUEST,
            ApplicationError::Conflict(_) => StatusCode::CONFLICT,
            ApplicationError::Persistence(_) | ApplicationError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Self::new(status, value.to_string())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod test;
