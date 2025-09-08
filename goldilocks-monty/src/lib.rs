//! Goldilocks field implementation using Montgomery arithmetic with extension field support.
//! 
//! This crate provides a Montgomery form implementation of the Goldilocks prime field,
//! with optional AVX2 vectorization support for improved performance.
//!
//! ## AVX2 Support
//!
//! When compiled with AVX2 support, this crate provides vectorized operations through
//! `PackedGoldilocksMontyAVX2`, which processes 4 field elements simultaneously.
//!
//! ### Building with AVX2
//! 
//! To enable AVX2 optimizations:
//! ```bash
//! RUSTFLAGS="-C target-feature=+avx2" cargo build --release
//! ```
//!
//! ### Benchmarking
//!
//! To run benchmarks comparing scalar vs AVX2 performance:
//! ```bash
//! ./bench_avx2.sh
//! ```
//!
//! Or manually:
//! ```bash
//! RUSTFLAGS="-C target-feature=+avx2" cargo bench --bench bench_field
//! ```

#![no_std]

extern crate alloc;

mod extension;
mod goldilocks;

pub use goldilocks::*;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
mod x86_64_avx2;

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
pub use x86_64_avx2::*;
