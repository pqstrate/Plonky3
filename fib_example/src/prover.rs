//! # Fibonacci Proof Generation and Verification
//! 
//! This module provides high-level functions for generating and verifying
//! zero-knowledge proofs of Fibonacci computations.

use p3_baby_bear::BabyBear;
use p3_uni_stark::{prove, verify, Proof};
use crate::{
    FibonacciAir, 
    FibonacciConfig, 
    create_fibonacci_config,
    generate_fibonacci_trace,
    print_trace
};

/// Result type for proof operations
pub type ProofResult<T> = Result<T, String>;

/// Complete proof generation for Fibonacci sequence
/// 
/// This function handles the entire proof generation pipeline:
/// 1. Generate execution trace
/// 2. Set up public values  
/// 3. Create proof
/// 
/// ## Parameters:
/// - `start_a`: First Fibonacci number (usually 1)
/// - `start_b`: Second Fibonacci number (usually 1)
/// - `num_steps`: Number of computation steps (must be power of 2)
/// - `expected_result`: The final Fibonacci number we're proving knowledge of
/// - `config`: Cryptographic configuration (or None for default)
/// - `verbose`: Whether to print detailed information during proving
/// 
/// ## Returns:
/// A proof object that can be verified later
/// 
/// ## Example:
/// ```rust
/// use fib_example::*;
/// 
/// // Prove knowledge of F(8) = 21 
/// let proof = prove_fibonacci_knowledge(1, 1, 8, 21, None, true)?;
/// ```
pub fn prove_fibonacci_knowledge(
    start_a: u64,
    start_b: u64, 
    num_steps: usize,
    expected_result: u64,
    config: Option<FibonacciConfig>,
    verbose: bool
) -> ProofResult<Proof<FibonacciConfig>> {
    if verbose {
        println!("🔧 Setting up proof generation...");
        println!("   Starting values: F(1)={}, F(2)={}", start_a, start_b);
        println!("   Computing {} steps", num_steps);
        println!("   Expected result: {}", expected_result);
    }
    
    // Use provided config or create default
    let config = config.unwrap_or_else(create_fibonacci_config);
    
    // Generate execution trace
    if verbose {
        println!("\n📊 Generating execution trace...");
    }
    let trace = generate_fibonacci_trace::<BabyBear>(start_a, start_b, num_steps);
    
    if verbose {
        print_trace(&trace, "Fibonacci Execution Trace");
    }
    
    // Set up public values
    let public_values = vec![
        BabyBear::new(start_a as u32),
        BabyBear::new(start_b as u32),
        BabyBear::new(expected_result as u32),
    ];
    
    if verbose {
        println!("\n🔓 Public values:");
        println!("   Initial left:  {}", start_a);
        println!("   Initial right: {}", start_b);
        println!("   Final result:  {}", expected_result);
    }
    
    // Create AIR instance
    let air = FibonacciAir::new();
    
    // Generate proof
    if verbose {
        println!("\n🔐 Generating zero-knowledge proof...");
        println!("   This may take a moment depending on trace size...");
    }
    
    let start_time = std::time::Instant::now();
    let proof = prove(&config, &air, trace, &public_values);
    let proof_time = start_time.elapsed();
    
    if verbose {
        println!("   ✅ Proof generated successfully in {:.3}s", proof_time.as_secs_f64());
    }
    
    Ok(proof)
}

/// Verify a Fibonacci proof
/// 
/// ## Parameters:
/// - `proof`: The proof to verify
/// - `start_a`: Expected first Fibonacci number
/// - `start_b`: Expected second Fibonacci number  
/// - `expected_result`: Expected final result
/// - `config`: Cryptographic configuration (or None for default)
/// - `verbose`: Whether to print verification details
/// 
/// ## Returns:
/// Ok(()) if verification succeeds, Err otherwise
pub fn verify_fibonacci_proof(
    proof: &Proof<FibonacciConfig>,
    start_a: u64,
    start_b: u64,
    expected_result: u64,
    config: Option<FibonacciConfig>,
    verbose: bool
) -> ProofResult<()> {
    if verbose {
        println!("\n🔍 Verifying proof...");
        println!("   Expected initial values: F(1)={}, F(2)={}", start_a, start_b);
        println!("   Expected result: {}", expected_result);
    }
    
    // Use provided config or create default
    let config = config.unwrap_or_else(create_fibonacci_config);
    
    // Set up public values (must match what was used in proving)
    let public_values = vec![
        BabyBear::new(start_a as u32),
        BabyBear::new(start_b as u32),
        BabyBear::new(expected_result as u32),
    ];
    
    // Create AIR instance
    let air = FibonacciAir::new();
    
    // Verify the proof
    let start_time = std::time::Instant::now();
    let result = verify(&config, &air, proof, &public_values);
    let verify_time = start_time.elapsed();
    
    match result {
        Ok(()) => {
            if verbose {
                println!("   ✅ Verification successful in {:.3}ms", verify_time.as_millis());
                println!("   The proof is valid!");
            }
            Ok(())
        }
        Err(e) => {
            if verbose {
                println!("   ❌ Verification failed: {:?}", e);
            }
            Err(format!("Verification failed: {:?}", e))
        }
    }
}

/// Complete prove-and-verify workflow
/// 
/// This is a convenience function that combines proof generation and verification.
/// Useful for testing and demonstrations.
/// 
/// ## Parameters:
/// - `start_a`: First Fibonacci number
/// - `start_b`: Second Fibonacci number
/// - `num_steps`: Number of computation steps
/// - `expected_result`: Expected final result
/// - `verbose`: Whether to print detailed progress
pub fn prove_and_verify_fibonacci(
    start_a: u64,
    start_b: u64,
    num_steps: usize,
    expected_result: u64,
    verbose: bool
) -> ProofResult<()> {
    if verbose {
        println!("🚀 Starting complete Fibonacci proof workflow");
        println!("===============================================");
    }
    
    // Generate proof
    let proof = prove_fibonacci_knowledge(
        start_a, start_b, num_steps, expected_result, None, verbose
    )?;
    
    // Verify proof
    verify_fibonacci_proof(
        &proof, start_a, start_b, expected_result, None, verbose
    )?;
    
    if verbose {
        println!("\n🎉 Complete workflow successful!");
        println!("   Proved and verified F({}) = {} starting from F(1)={}, F(2)={}", 
                 num_steps + 1, expected_result, start_a, start_b);
    }
    
    Ok(())
}

/// Batch proving for multiple Fibonacci computations
/// 
/// This function can prove multiple Fibonacci sequences in sequence.
/// Useful for benchmarking or testing different parameters.
pub fn batch_prove_fibonacci(test_cases: Vec<(u64, u64, usize, u64)>, verbose: bool) -> ProofResult<()> {
    if verbose {
        println!("🔄 Starting batch Fibonacci proving...");
        println!("   {} test cases to process", test_cases.len());
    }
    
    for (i, (start_a, start_b, num_steps, expected_result)) in test_cases.iter().enumerate() {
        if verbose {
            println!("\n--- Test Case {} ---", i + 1);
        }
        
        prove_and_verify_fibonacci(*start_a, *start_b, *num_steps, *expected_result, verbose)?;
        
        if verbose {
            println!("   ✅ Test case {} passed", i + 1);
        }
    }
    
    if verbose {
        println!("\n🎉 All batch tests completed successfully!");
    }
    
    Ok(())
}

/// Benchmark proof generation and verification
/// 
/// Times the proof generation and verification process for performance analysis.
pub fn benchmark_fibonacci_proof(
    start_a: u64,
    start_b: u64, 
    num_steps: usize,
    expected_result: u64,
    num_iterations: usize
) -> ProofResult<(f64, f64)> {
    println!("⏱️  Benchmarking Fibonacci proof ({}×{} trace, {} iterations)", 
             num_steps, 2, num_iterations);
    
    let config = create_fibonacci_config();
    let air = FibonacciAir::new();
    let public_values = vec![
        BabyBear::new(start_a as u32),
        BabyBear::new(start_b as u32),
        BabyBear::new(expected_result as u32),
    ];
    
    // Benchmark proving
    let mut total_prove_time = 0.0;
    let mut proofs = Vec::new();
    
    for i in 0..num_iterations {
        let trace = generate_fibonacci_trace::<BabyBear>(start_a, start_b, num_steps);
        
        let start_time = std::time::Instant::now();
        let proof = prove(&config, &air, trace, &public_values);
        let prove_time = start_time.elapsed();
        
        total_prove_time += prove_time.as_secs_f64();
        proofs.push(proof);
        
        if i == 0 {
            println!("   First proof: {:.3}s", prove_time.as_secs_f64());
        }
    }
    
    let avg_prove_time = total_prove_time / num_iterations as f64;
    
    // Benchmark verification
    let mut total_verify_time = 0.0;
    
    for proof in &proofs {
        let start_time = std::time::Instant::now();
        match verify(&config, &air, proof, &public_values) {
            Ok(()) => {
                let verify_time = start_time.elapsed();
                total_verify_time += verify_time.as_secs_f64();
            }
            Err(e) => return Err(format!("Benchmark verification failed: {:?}", e))
        }
    }
    
    let avg_verify_time = total_verify_time / num_iterations as f64;
    
    println!("   Average prove time: {:.3}s", avg_prove_time);
    println!("   Average verify time: {:.3}s", avg_verify_time);
    
    Ok((avg_prove_time, avg_verify_time))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_fibonacci_proof() {
        // Test basic Fibonacci proof: F(8) = 21
        let result = prove_and_verify_fibonacci(1, 1, 8, 34, false);
        assert!(result.is_ok(), "Basic Fibonacci proof should succeed");
    }
    
    #[test]
    fn test_different_starting_values() {
        // Test with different starting values: 2, 3 -> 5, 8, 13, 21
        let result = prove_and_verify_fibonacci(2, 3, 4, 21, false);
        assert!(result.is_ok(), "Fibonacci proof with different start should succeed");
    }
    
    #[test]
    fn test_single_step() {
        // Test minimal case: just one step
        let result = prove_and_verify_fibonacci(5, 8, 1, 8, false);
        assert!(result.is_ok(), "Single-step Fibonacci proof should succeed");
    }
    
    #[test] 
    fn test_batch_proving() {
        let test_cases = vec![
            (1, 1, 4, 8),   // F(5) = 8
            (1, 1, 8, 34),  // F(9) = 34
            (2, 3, 2, 8),   // Custom sequence
        ];
        
        let result = batch_prove_fibonacci(test_cases, false);
        assert!(result.is_ok(), "Batch proving should succeed");
    }
    
    #[test]
    fn test_wrong_result_fails() {
        // This should fail because 99 is not F(9)
        let proof_result = prove_fibonacci_knowledge(1, 1, 8, 99, None, false);
        
        // The proof generation might succeed but verification should fail
        // when we try to verify with the correct expected result
        match proof_result {
            Ok(proof) => {
                let verify_result = verify_fibonacci_proof(&proof, 1, 1, 34, None, false);
                // This should fail because we proved 99 but are verifying against 34
                assert!(verify_result.is_err(), "Verification with wrong result should fail");
            }
            Err(_) => {
                // Or the proof generation itself might fail, which is also acceptable
            }
        }
    }
}