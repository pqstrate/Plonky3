use std::fs::File;
use std::io::Write;

use miden_assembly::Assembler;
use miden_processor::{AdviceInputs, DefaultHost, ExecutionOptions, StackInputs, execute};
use winter_prover::Trace;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::{PrimeCharacteristicRing, PrimeField64, extension::BinomialExtensionField};
use p3_fri::{TwoAdicFriPcs, create_benchmark_fri_params};
use p3_goldilocks::Goldilocks;
use p3_keccak::{Keccak256Hash, KeccakF};
use p3_matrix::{Matrix, dense::RowMajorMatrix};
pub use miden_processor::ExecutionTrace as MidenTrace;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher};
use p3_uni_stark::{StarkConfig, prove, verify};
use crate::trace_gen;

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
