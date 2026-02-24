// E2E test entry point
// These tests require krunvm to be installed and available

#![allow(unused_imports)]

#[path = "e2e/mod.rs"]
mod e2e;

// Export test functions for cargo test
pub use e2e::dev_templates_tests::*;
pub use e2e::vm_lifecycle_tests::*;
pub use e2e::workspace_tests::*;
