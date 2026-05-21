//! Karpathy "LLM Wiki" consolidation pipeline.
//!
//! Implements the three operations from the gist: ingest (write fan-out),
//! query (hierarchical retrieval), and lint (contradiction & orphan
//! detection). Plus the retention/decay sweep. Lands in milestones M7 / M8.
