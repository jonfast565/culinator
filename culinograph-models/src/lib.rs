//! Shared contracts and transport-neutral models for Culinograph.
//!
//! This crate contains no use-case orchestration and no infrastructure. It is
//! safe for adapters, application services, CLI, LSP, and delivery layers to
//! depend on it without creating a dependency on the application crate.

pub mod error;
pub mod haccp;
pub mod models;
pub mod ports;

pub use error::ApplicationError;
pub use haccp::*;
pub use models::*;
pub use ports::*;

#[cfg(test)]
mod test;
