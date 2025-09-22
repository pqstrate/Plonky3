//! Demo of the new fib2 APIs
//!
//! This example demonstrates the new refactored API:
//! - trace_gen(fib_iter: usize) -> (MidenTrace, P3Trace)
//! - p3_generate_proof(p3_trace)  
//! - miden_generate_proof(miden_trace)

use std::env;

use fib2::{miden_generate_proof, p3_generate_proof, trace_gen};
use p3_matrix::Matrix;
use winter_prover::Trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_threads = env::var("NUM_THREADS")
        .unwrap_or_else(|_| "8".to_string())
        .parse::<usize>()
        .unwrap_or(8);

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    // Initialize tracing subscriber for logging/benchmarking with span traces
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NEW)
        .with_ansi(atty::is(atty::Stream::Stdout))
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();

    println!("🎯 Demo of New fib2 APIs");
    println!("========================\n");

    let base = 1;
    for log_iter in 19..20 {
        let iteration = base << log_iter;
        println!("\n🔐 Generating proof from Plonky3 trace...");
        let (miden_trace, p3_trace, program, stack_inputs, advice_inputs) = trace_gen(iteration)?;
        println!(
            "========================\n   Using P3 trace ({}×{}) for proof generation...\n========================",
            p3_trace.height(),
            p3_trace.width()
        );

        println!("\n🔐 P3 with Keccak.");
        match p3_generate_proof(p3_trace.clone(), true) {
            Ok(()) => println!("   ✅ P3 Keccak proof generation successful!"),
            Err(e) => println!("   ❌ P3 Keccak proof generation failed: {}", e),
        }

        // println!("\n🔐 P3 with Poseidon2.");
        // match p3_generate_proof(p3_trace, false) {
        //     Ok(()) => println!("   ✅ P3 Poseidon2 proof generation successful!"),
        //     Err(e) => println!("   ❌ P3 Poseidon2 proof generation failed: {}", e),
        // }

        println!(
            "========================\n   Using Miden trace ({}×{}) for proof generation...\n========================",
            miden_trace.length(),
            miden_trace.main_trace_width()
        );

        println!("\n🔐 Miden with blake3.");
        match miden_generate_proof(&program, stack_inputs.clone(), advice_inputs.clone(), true) {
            Ok(()) => println!("   ✅ Miden proof generation successful!"),
            Err(e) => println!("   ❌ Miden proof generation failed: {}", e),
        }

        // println!("\n🔐 Miden with rpo256.");
        // match miden_generate_proof(&program, stack_inputs, advice_inputs, false) {
        //     Ok(()) => println!("   ✅ Miden proof generation successful!"),
        //     Err(e) => println!("   ❌ Miden proof generation failed: {}", e),
        // }

        println!("\n🎉 All API demos completed!");
        println!("   Check the generated fib_*_trace_*.log files to see the traces.");
    }

    Ok(())
}
