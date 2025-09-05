//! Goldilocks field implementation using Montgomery arithmetic with extension field support.

#![no_std]

extern crate alloc;

mod extension;
mod goldilocks;

pub use goldilocks::*;
