//! Culinator application use cases.
//!
//! Shared DTOs and replaceable ports live in `culinator-models`.
//! This crate contains orchestration only.

pub mod services;

pub use culinator_models::*;
pub use services::*;

#[cfg(test)]
mod test;
