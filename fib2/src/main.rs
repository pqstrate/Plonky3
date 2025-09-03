//! # Increment Constraint STARK Proof Example
//!
//! This is a complete example of how to build a STARK (Scalable Transparent Argument of Knowledge)
//! proof system using Plonky3. 
//!
//! ## What this example proves:
//!
//! Given an execution trace (stored in `trace.txt`), this program proves that the first column
//! follows an increment constraint: `trace[i][0] = trace[i-1][0] + 1` for all consecutive rows.
//!
//! ## Key Components:
//!
//! 1. **Field**: Goldilocks (64-bit prime field optimized for STARK proofs)
//! 2. **AIR**: Arithmetic Intermediate Representation defining the increment constraint
//! 3. **Polynomial Commitment**: FRI (Fast Reed-Solomon Interactive Oracle Proof)
//! 4. **Hash Function**: Keccak256 for cryptographic commitments
//! 5. **Trace Processing**: Parsing, validation, and power-of-2 padding
//!
//! ## How to use:
//!
//! 1. Ensure `trace.txt` is in the current directory
//! 2. Run: `cargo run --release`  
//! 3. The program will parse the trace, generate a proof, and verify it
//!
//! ## Expected Output:
//!
//! - Trace parsing with dimension information
//! - Proof generation timing
//! - Proof verification timing  
//! - Success confirmation if the constraint holds

use fib2::generate_proof;

/// Main entry point for the STARK proof generation and verification
///
/// This function serves as the entry point for demonstrating a complete STARK proof workflow:
/// 1. Parse execution trace from file
/// 2. Set up cryptographic primitives and polynomial commitment scheme
/// 3. Generate zero-knowledge proof that the trace satisfies increment constraints
/// 4. Verify the proof to ensure correctness
///
/// The entire process demonstrates the power of STARK proofs: proving knowledge of a
/// valid computation without revealing the intermediate steps, with succinct proof size
/// and efficient verification.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_proof()
}