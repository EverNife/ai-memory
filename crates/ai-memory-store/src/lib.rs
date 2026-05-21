//! SQLite storage layer for ai-memory.
//!
//! Hosts the single-writer actor and the read-only connection pool. Schema
//! migrations land via `refinery`. Implementation lands in milestone M1; this
//! crate is intentionally empty in M0.
