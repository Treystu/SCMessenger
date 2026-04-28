// DSPy Integration Module
// This module provides DSPy-style programmatic orchestration for the SCMessenger swarm.

//! DSPy Programmatic Integration
//!
//! This module enables deterministic model routing and autonomous prompt optimization
//! using DSPy (Declarative Self-optimizing Language Programs) principles.
//!
//! Key Components:
//! - Signatures: Type-like definitions for input/output schemas
//! - Modules: Chain-of-Thought pipelines for logic flow
//! - Teleprompters: Automatic prompt optimization against Golden Examples

// Re-export DSPy bindings (generated from Python module via UniFFI)
pub mod signatures;
pub mod modules;
pub mod teleprompt;

pub use signatures::*;
pub use modules::*;
pub use teleprompt::*;
