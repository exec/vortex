//! # Vortex
//!
//! Lightning-fast ephemeral VM platform with hardware-level isolation.
//! 20x faster than Docker DevContainers with true security.

pub mod core;

// Re-export everything from core for convenience
pub use core::*;
