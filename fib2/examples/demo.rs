//! Demo of the new fib2 APIs
//!
//! This example demonstrates the new refactored API:
//! - trace_gen(fib_iter: usize) -> (MidenTrace, P3Trace)
//! - p3_generate_proof(p3_trace)  
//! - miden_generate_proof(miden_trace)

use fib2::{miden_generate_proof, p3_generate_proof, trace_gen};
use p3_matrix::Matrix;
use winter_prover::Trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Demo of New fib2 APIs");
    println!("========================\n");

    // === Demo 1: Generate traces for different iteration counts ===
    println!("📊 Demo 1: Generating traces for different iterations...");

    for &iterations in &[5, 15, 25] {
        println!(
            "\n   • Generating traces for {} Fibonacci iterations...",
            iterations
        );
        let (miden_trace, p3_trace) = trace_gen(iterations)?;

        println!(
            "     - Miden trace: {}×{}",
            miden_trace.length(),
            miden_trace.main_trace_width()
        );
        println!(
            "     - P3 trace: {}×{}",
            p3_trace.height(),
            p3_trace.width()
        );
        println!("     - Files: fib_{}_trace_[miden|p3].log", iterations);
    }

    // === Demo 2: Generate proof from Plonky3 trace ===
    println!("\n🔐 Demo 2: Generating proof from Plonky3 trace...");
    let (miden_trace, p3_trace) = trace_gen(30)?;
    println!(
        "   Using P3 trace ({}×{}) for proof generation...",
        p3_trace.height(),
        p3_trace.width()
    );

    match p3_generate_proof(p3_trace) {
        Ok(()) => println!("   ✅ P3 proof generation successful!"),
        Err(e) => println!("   ❌ P3 proof generation failed: {}", e),
    }

    println!(
        "   Using Miden trace ({}×{}) for proof generation...",
        miden_trace.length(),
        miden_trace.main_trace_width()
    );

    match miden_generate_proof(miden_trace) {
        Ok(()) => println!("   ✅ Miden proof generation successful!"),
        Err(e) => println!("   ❌ Miden proof generation failed: {}", e),
    }

    println!("\n🎉 All API demos completed!");
    println!("   Check the generated fib_*_trace_*.log files to see the traces.");

    Ok(())
}
