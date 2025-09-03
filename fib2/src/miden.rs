use std::time::Instant;

use ark_std::{end_timer, start_timer};
pub use miden_processor::ExecutionTrace as MidenTrace;
use miden_prover::{ProvingOptions, prove};
use miden_verifier::verify;
use miden_vm::{AdviceInputs, DefaultHost, Program, ProgramInfo, StackInputs};

/// Generate a STARK proof using Miden's native proving system
///
/// # Arguments
/// * `program` - The Miden program to prove
/// * `stack_inputs` - Stack inputs for the program
/// * `advice_inputs` - Advice inputs for the program
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub fn miden_generate_proof(
    program: &Program,
    stack_inputs: StackInputs,
    advice_inputs: AdviceInputs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Generating native Miden STARK proof...");

    // Generate proof
    let proving_options = ProvingOptions::default();
    let mut host_for_proving = DefaultHost::default();

    let proof_start = Instant::now();
    let proof_timer = start_timer!(|| "Miden STARK proof generation");
    let (stack_outputs, proof) = prove(
        program,
        stack_inputs.clone(),
        advice_inputs.clone(),
        &mut host_for_proving,
        proving_options,
    )?;
    end_timer!(proof_timer);
    let proof_time = proof_start.elapsed();

    println!("   ✅ Miden proof generated successfully!");
    println!("   📊 Proof generation time: {:?}", proof_time);
    println!("   🔢 Final result on stack: {:?}", stack_outputs);

    // Verify the proof
    println!("   🔍 Verifying Miden proof...");
    let program_info: ProgramInfo = program.clone().into();

    let verify_start = Instant::now();
    let verify_timer = start_timer!(|| "Miden proof verification");
    match verify(program_info, stack_inputs, stack_outputs.clone(), proof) {
        Ok(security_level) => {
            end_timer!(verify_timer);
            let verify_time = verify_start.elapsed();
            println!("   ✅ Proof verification successful!");
            println!("   ⏱️  Verification time: {:?}", verify_time);
            println!("   🔒 Security level: {} bits", security_level);
        }
        Err(e) => {
            end_timer!(verify_timer);
            let verify_time = verify_start.elapsed();
            println!("   ❌ Proof verification failed: {:?}", e);
            println!("   ⏱️  Verification time: {:?}", verify_time);
            return Err(format!("Miden proof verification failed: {:?}", e).into());
        }
    }

    println!("   🎉 Successfully generated and verified native Miden STARK proof!");
    Ok(())
}
