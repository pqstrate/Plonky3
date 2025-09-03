//! # Step-by-Step Plonky3 Fibonacci Proof
//! 
//! This crate demonstrates how to build a complete zero-knowledge proof system
//! using Plonky3 for proving knowledge of the nth Fibonacci number.
//! 
//! ## Overview
//! 
//! A zero-knowledge proof system has several key components:
//! 1. **Arithmetic Intermediate Representation (AIR)** - defines the computation constraints
//! 2. **Execution Trace** - the step-by-step computation record
//! 3. **Polynomial Commitment Scheme (PCS)** - commits to polynomials
//! 4. **Interactive Oracle Proof (IOP)** - the proof protocol
//! 5. **Configuration** - ties everything together
//! 
//! ## Step 1: Define the AIR (Arithmetic Intermediate Representation)
//! 
//! The AIR defines what computations are valid. For Fibonacci, we need:
//! - Two columns: `left` and `right` representing consecutive Fibonacci numbers
//! - Transition constraint: `next.left = current.right` and `next.right = current.left + current.right`
//! - Boundary constraints: first row starts with (a, b), last row ends with target value

use core::borrow::Borrow;

// Core Plonky3 imports
use p3_matrix::Matrix;
use p3_uni_stark::{prove, verify};

// Field
use p3_baby_bear::BabyBear;
use p3_field::PrimeField32;

pub mod air;
pub mod trace;
pub mod config;
pub mod prover;
pub mod simple;

pub use air::*;
pub use trace::*;
pub use config::*;
pub use prover::*;

/// Number of columns in our Fibonacci trace
pub const NUM_FIBONACCI_COLS: usize = 2;

/// A single row in the Fibonacci execution trace
/// Each row contains two consecutive Fibonacci numbers
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FibonacciRow<F> {
    pub left: F,   // F(n-1) 
    pub right: F,  // F(n)
}

impl<F> FibonacciRow<F> {
    pub const fn new(left: F, right: F) -> Self {
        Self { left, right }
    }
}

/// This allows us to view a slice as a FibonacciRow
impl<F> Borrow<FibonacciRow<F>> for [F] {
    fn borrow(&self) -> &FibonacciRow<F> {
        debug_assert_eq!(self.len(), NUM_FIBONACCI_COLS);
        let (prefix, rows, suffix) = unsafe { self.align_to::<FibonacciRow<F>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert!(suffix.is_empty(), "Alignment should match");
        debug_assert_eq!(rows.len(), 1);
        &rows[0]
    }
}

/// Comprehensive example function that demonstrates the complete flow
pub fn fibonacci_proof_example() -> Result<(), String> {
    println!("🚀 Starting Plonky3 Fibonacci Proof Example");
    
    // Step 1: Set up parameters
    println!("\n📋 Step 1: Setting up parameters");
    let n = 8;        // Number of steps (must be power of 2)
    let target = 34;  // Expected F(9) = 34 (starting from F(1)=1, F(2)=1, doing 8 steps gets us F(9))
    
    println!("   • Computing {} Fibonacci steps", n);
    println!("   • Expected result: F({}) = {}", n + 1, target);
    
    // Step 2: Create configuration
    println!("\n⚙️  Step 2: Creating cryptographic configuration");
    let config = create_fibonacci_config();
    println!("   • Field: BabyBear (31-bit prime field)");
    println!("   • Extension field: 4-degree binomial extension");
    println!("   • Hash function: Poseidon2");
    println!("   • PCS: FRI (Fast Reed-Solomon Interactive Oracle Proof)");
    
    // Step 3: Generate execution trace
    println!("\n📊 Step 3: Generating execution trace");
    let trace = generate_fibonacci_trace::<BabyBear>(1, 1, n);
    println!("   • Generated {}×{} trace matrix", trace.height(), trace.width());
    println!("   • Starting values: F(1)=1, F(2)=1");
    
    // Print the trace for educational purposes
    println!("   • Trace contents:");
    for i in 0..trace.height() {
        let row = trace.row_slice(i).unwrap();
        let fib_row: &FibonacciRow<BabyBear> = (&*row).borrow();
        println!("     Row {}: left={:2}, right={:2}", i, fib_row.left.as_canonical_u32(), fib_row.right.as_canonical_u32());
    }
    
    // Step 4: Set up public values
    println!("\n🔓 Step 4: Setting up public values");
    let public_values = vec![
        BabyBear::new(1),      // F(1) = 1
        BabyBear::new(1),      // F(2) = 1  
        BabyBear::new(target), // F(n+1) = target
    ];
    println!("   • Public values: [F(1)={}, F(2)={}, F({})={}]", 1, 1, n + 1, target);
    
    // Step 5: Create AIR instance
    println!("\n🏗️  Step 5: Creating AIR (Arithmetic Intermediate Representation)");
    let air = FibonacciAir::new();
    println!("   • AIR defines the computation constraints");
    println!("   • Transition: next.left = current.right, next.right = current.left + current.right");
    println!("   • Boundary: first row = (1,1), last row ends with target");
    
    // Step 6: Generate proof
    println!("\n🔐 Step 6: Generating zero-knowledge proof");
    println!("   • This may take a moment...");
    
    let start_time = std::time::Instant::now();
    let proof = prove(&config, &air, trace, &public_values);
    let proof_time = start_time.elapsed();
    
    println!("   • Proof generated in {:.2}s", proof_time.as_secs_f64());
    println!("   • Proof contains commitment to execution trace");
    println!("   • Proof demonstrates knowledge of valid Fibonacci computation");
    
    // Step 7: Verify proof
    println!("\n✅ Step 7: Verifying the proof");
    let start_time = std::time::Instant::now();
    match verify(&config, &air, &proof, &public_values) {
        Ok(()) => {
            let verify_time = start_time.elapsed();
            println!("   • Verification completed in {:.2}ms", verify_time.as_millis());
            println!("   • ✅ Proof is valid!");
        }
        Err(e) => {
            return Err(format!("Verification failed: {:?}", e));
        }
    }
    
    // Step 8: Summary
    println!("\n🎉 Summary:");
    println!("   • Successfully proved knowledge of F({}) = {} using zero-knowledge", n + 1, target);
    println!("   • The prover demonstrated they computed the Fibonacci sequence correctly");
    println!("   • The verifier can be confident in the result without seeing intermediate steps");
    println!("   • This is the essence of zero-knowledge proofs: proving knowledge without revealing computation details");
    
    Ok(())
}

/// Convenience function for the most common use case
pub fn prove_fibonacci(n: usize, expected_result: u32) -> Result<(), String> {
    let config = create_fibonacci_config();
    let trace = generate_fibonacci_trace::<BabyBear>(1, 1, n);
    let public_values = vec![
        BabyBear::new(1),
        BabyBear::new(1),
        BabyBear::new(expected_result),
    ];
    
    let air = FibonacciAir::new();
    let proof = prove(&config, &air, trace, &public_values);
    match verify(&config, &air, &proof, &public_values) {
        Ok(()) => {
            println!("✅ Successfully proved F({}) = {}", n, expected_result);
            Ok(())
        }
        Err(e) => Err(format!("Verification failed: {:?}", e))
    }
}