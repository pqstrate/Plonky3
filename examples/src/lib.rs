// AIR (Algebraic Intermediate Representation) definitions for hash functions
pub mod airs;
// Discrete Fourier Transform implementations and wrappers
pub mod dfts;
// Command-line argument parsers for different configuration options
pub mod parsers;
// Proof generation and verification functions for different STARK configurations
pub mod proofs;
// Type definitions for STARK configurations and Merkle tree setups
pub mod types;

#[cfg(test)]
// End-to-end integration tests for the proof system
mod tests;
