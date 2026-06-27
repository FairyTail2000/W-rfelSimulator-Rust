//! Library facade exposing the crate's internals for benchmarks and integration tests.
//!
//! The binary (`main.rs`) keeps its own module tree; this only re-exposes the pieces
//! needed to exercise the dice rolling hot paths from `benches/`.
#![allow(dead_code)]

pub mod common;
pub mod dice;
