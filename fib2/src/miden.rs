pub use miden_processor::ExecutionTrace as MidenTrace;
use winter_prover::Trace;

/// Generate a STARK proof directly from a Miden trace
///
/// # Arguments
/// * `miden_trace` - The Miden VM execution trace
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub fn miden_generate_proof(
    miden_trace: miden_processor::ExecutionTrace,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Generating STARK proof from Miden trace...");

    println!(
        "   📏 Miden trace dimensions: {}×{}",
        miden_trace.length(),
        miden_trace.main_trace_width()
    );

    // // Convert Miden trace to Plonky3 format
    // println!("   🔄 Converting to Plonky3 format...");
    // let p3_trace = convert_miden_trace::<Goldilocks>(&miden_trace)?;

    // // Generate proof using the Plonky3 trace
    // p3_generate_proof(p3_trace)
    todo!()
}
