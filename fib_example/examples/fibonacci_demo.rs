//! # Interactive Fibonacci Proof Demo
//! 
//! This example demonstrates the complete Plonky3 proof workflow step-by-step.
//! Run with: `cargo run --example fibonacci_demo`

use fib_example::*;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Welcome to the Plonky3 Fibonacci Proof Demo!");
    println!("================================================");
    println!();
    println!("This demo will show you how to create zero-knowledge proofs");
    println!("for Fibonacci sequence computations using Plonky3.");
    println!();
    
    // Interactive mode
    if std::env::args().any(|arg| arg == "--interactive") {
        return run_interactive_demo();
    }
    
    // Run preset demonstrations
    println!("🎯 Running preset demonstrations...");
    
    // Demo 1: Basic Fibonacci proof
    println!("\n" + "=".repeat(60));
    println!("📊 DEMO 1: Basic Fibonacci Proof");
    println!("=".repeat(60));
    println!("Goal: Prove knowledge that F(8) = 21 (starting from F(1)=1, F(2)=1)");
    println!();
    
    fibonacci_proof_example()?;
    
    // Demo 2: Different starting values
    println!("\n" + "=".repeat(60));
    println!("🔄 DEMO 2: Custom Starting Values");
    println!("=".repeat(60));
    println!("Goal: Prove Fibonacci sequence starting from F(1)=2, F(2)=3");
    println!();
    
    prove_and_verify_fibonacci(2, 3, 4, 21, true)?;
    
    // Demo 3: Larger computation
    println!("\n" + "=".repeat(60));
    println!("🚀 DEMO 3: Larger Fibonacci Computation");
    println!("=".repeat(60));
    println!("Goal: Prove knowledge of F(16) = 987");
    println!();
    
    prove_and_verify_fibonacci(1, 1, 16, 1597, true)?;
    
    // Demo 4: Batch processing
    println!("\n" + "=".repeat(60));
    println!("⚡ DEMO 4: Batch Proof Generation");
    println!("=".repeat(60));
    println!("Goal: Prove multiple Fibonacci sequences in batch");
    println!();
    
    let test_cases = vec![
        (1, 1, 2, 2),   // F(3) = 2
        (1, 1, 4, 8),   // F(5) = 8  
        (1, 1, 8, 34),  // F(9) = 34
        (3, 5, 2, 13),  // Custom: 3,5,8,13
    ];
    
    batch_prove_fibonacci(test_cases, true)?;
    
    // Demo 5: Performance benchmark
    println!("\n" + "=".repeat(60));
    println!("📈 DEMO 5: Performance Benchmark");
    println!("=".repeat(60));
    println!("Goal: Measure proof generation and verification performance");
    println!();
    
    let (prove_time, verify_time) = benchmark_fibonacci_proof(1, 1, 8, 34, 3)?;
    
    println!("\n📊 Benchmark Results:");
    println!("   Average proving time:     {:.3}s", prove_time);
    println!("   Average verification time: {:.3}s", verify_time);
    println!("   Verification speedup:     {:.1}×", prove_time / verify_time);
    
    // Demo 6: Educational trace walkthrough
    println!("\n" + "=".repeat(60));
    println!("🎓 DEMO 6: Educational Trace Walkthrough");
    println!("=".repeat(60));
    println!("Goal: Understand the execution trace structure");
    println!();
    
    educational_trace_demo()?;
    
    // Conclusion
    println!("\n" + "=".repeat(60));
    println!("🎉 ALL DEMOS COMPLETED SUCCESSFULLY!");
    println!("=".repeat(60));
    println!();
    println!("What you've learned:");
    println!("• How to define computational constraints (AIR)");
    println!("• How to generate execution traces");
    println!("• How to configure cryptographic parameters");
    println!("• How to generate and verify zero-knowledge proofs");
    println!("• How zero-knowledge proofs enable privacy-preserving computation");
    println!();
    println!("Key concepts demonstrated:");
    println!("• Arithmetic Intermediate Representation (AIR)");
    println!("• Execution traces and polynomial commitment");
    println!("• Interactive Oracle Proofs (IOP) with FRI");
    println!("• Public vs private inputs in zero-knowledge systems");
    println!();
    println!("Try running with --interactive for hands-on exploration!");
    
    Ok(())
}

fn run_interactive_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Interactive Mode");
    println!("==================");
    
    loop {
        println!("\nWhat would you like to do?");
        println!("1. Generate a basic Fibonacci proof");
        println!("2. Custom Fibonacci computation");
        println!("3. View execution trace");
        println!("4. Benchmark performance"); 
        println!("5. Educational walkthrough");
        println!("6. Exit");
        print!("\nEnter your choice (1-6): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                println!("\n🔧 Generating basic Fibonacci proof (F(8) = 21)...");
                fibonacci_proof_example()?;
            }
            "2" => {
                println!("\n🔧 Custom Fibonacci computation");
                print!("Enter starting values (a b): ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let values: Vec<u64> = input
                    .trim()
                    .split_whitespace()
                    .map(|s| s.parse().unwrap_or(1))
                    .collect();
                
                let (a, b) = if values.len() >= 2 {
                    (values[0], values[1])
                } else {
                    (1, 1)
                };
                
                print!("Enter number of steps (power of 2): ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let steps = input.trim().parse().unwrap_or(8);
                
                // Calculate expected result
                let trace = generate_fibonacci_trace::<p3_baby_bear::BabyBear>(a, b, steps);
                let last_row = trace.row_slice(steps - 1);
                let last_fib_row: &FibonacciRow<_> = last_row.borrow();
                let expected = last_fib_row.right.as_canonical_u64();
                
                println!("\n🎯 Computing Fibonacci sequence:");
                println!("   Starting: F(1)={}, F(2)={}", a, b);
                println!("   Steps: {}", steps);
                println!("   Expected result: {}", expected);
                
                prove_and_verify_fibonacci(a, b, steps, expected, true)?;
            }
            "3" => {
                educational_trace_demo()?;
            }
            "4" => {
                print!("\nEnter number of benchmark iterations: ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let iterations = input.trim().parse().unwrap_or(3);
                
                benchmark_fibonacci_proof(1, 1, 8, 34, iterations)?;
            }
            "5" => {
                educational_walkthrough()?;
            }
            "6" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid choice. Please enter 1-6.");
            }
        }
    }
    
    Ok(())
}

fn educational_trace_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎓 Understanding Execution Traces");
    println!("=================================");
    println!();
    println!("An execution trace is a matrix that records every step of computation.");
    println!("For Fibonacci, each row contains two consecutive numbers: (F(i), F(i+1))");
    println!();
    
    let trace = generate_fibonacci_trace::<p3_baby_bear::BabyBear>(1, 1, 8);
    print_trace(&trace, "Educational Fibonacci Trace");
    
    println!("\n📋 Key observations:");
    println!("• Each row transition follows F(n+1) = F(n-1) + F(n)");
    println!("• Left column shifts: current.right → next.left");
    println!("• Right column adds: current.left + current.right → next.right");
    println!("• The trace must have power-of-2 rows for efficient polynomial operations");
    println!("• Public values constrain the first and last rows");
    
    Ok(())
}

fn educational_walkthrough() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎓 Zero-Knowledge Proof Concepts");
    println!("================================");
    println!();
    println!("A zero-knowledge proof system has three key properties:");
    println!();
    println!("1. 🔐 COMPLETENESS");
    println!("   If the statement is true, an honest prover can convince the verifier");
    println!();
    println!("2. 🛡️  SOUNDNESS"); 
    println!("   If the statement is false, no dishonest prover can convince the verifier");
    println!();
    println!("3. 🤐 ZERO-KNOWLEDGE");
    println!("   The verifier learns nothing except that the statement is true");
    println!();
    println!("In our Fibonacci example:");
    println!("• Statement: 'I know how to compute F(n) = result'");
    println!("• Public: Starting values and final result");
    println!("• Private: The intermediate computation steps");
    println!("• Proof: Demonstrates correct computation without revealing steps");
    println!();
    
    println!("🔧 Technical Components:");
    println!("• AIR (Arithmetic Intermediate Representation): Defines valid computations");
    println!("• Execution Trace: Records the step-by-step computation");
    println!("• Polynomial Commitment: Commits to trace polynomials");
    println!("• Interactive Oracle Proof: Verifies polynomial relationships");
    println!("• Random Challenges: Prevent cheating through randomness");
    
    Ok(())
}

// Add a simple CLI argument parser
#[allow(dead_code)]
fn parse_args() -> (bool, bool) {
    let args: Vec<String> = std::env::args().collect();
    let interactive = args.iter().any(|arg| arg == "--interactive");
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    (interactive, verbose)
}