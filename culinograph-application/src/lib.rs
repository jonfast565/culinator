//! Culinograph application use cases.
//!
//! Shared DTOs and replaceable ports live in `culinograph-models`.
//! This crate contains orchestration only.

pub mod services;

pub use culinograph_models::*;
pub use services::*;

#[cfg(test)]
mod test;
