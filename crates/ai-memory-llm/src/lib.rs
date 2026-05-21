//! LLM provider abstraction.
//!
//! Native typed HTTP clients per provider, never a generic gateway. Lesson
//! from cognee #2840: `LiteLLM` + `instructor` silently drop unknown kwargs
//! and the wrapper layer ends up papering over wire-protocol drift forever.
//! Implementation lands in milestone M6.
