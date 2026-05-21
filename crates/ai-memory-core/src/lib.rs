//! Core domain types and errors for ai-memory.
//!
//! This crate is the closure of the project's vocabulary: identifiers, agent
//! kinds, and the workspace-wide error type. Nothing in here performs I/O,
//! which keeps it trivially unit-testable and free of platform concerns.

pub mod error;
pub mod ids;

pub use error::{MemoryError, MemoryResult};
pub use ids::{AgentKind, ObservationId, PagePath, ProjectId, SessionId, WorkspaceId};
