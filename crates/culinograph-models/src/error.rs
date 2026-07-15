use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    #[error("{entity} was not found")]
    NotFound { entity: &'static str },
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("parsing failed: {0}")]
    Parse(String),
    #[error("validation failed")]
    Validation,
    #[error("persistence failed: {0}")]
    Persistence(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApplicationError {
    pub fn not_found(entity: &'static str) -> Self {
        Self::NotFound { entity }
    }
}

#[cfg(test)]
mod test;
