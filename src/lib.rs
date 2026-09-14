//! occonfig: named model profiles for opencode configuration.
//!
//! The crate is exposed as a library so integration tests can exercise the
//! same code paths the binary does, rather than shelling out for everything.

pub mod backup;
pub mod commands;
pub mod config;
pub mod profile;
pub mod validate;
pub mod version;
