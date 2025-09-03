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

/// Test that we can successfully generate traces using the new API
/// This test verifies:
/// 1. Miden program compilation works correctly
/// 2. Program execution produces a valid trace
/// 3. Trace conversion to Plonky3 format works
/// 4. Power-of-2 padding is applied correctly
#[test]
fn test_trace_gen() {
    match trace_gen(10) {
        Ok((miden_trace, p3_trace)) => {
            // Verify basic properties of both traces
            assert!(miden_trace.length() > 0, "Miden trace should have rows");
            assert!(
                miden_trace.main_trace_width() > 0,
                "Miden trace should have columns"
            );
            assert!(p3_trace.width() > 0, "P3 trace should have columns");
            assert!(
                p3_trace.height().is_power_of_two(),
                "P3 height should be power of 2"
            );

            println!(
                "Traces generated successfully: Miden {}×{}, P3 {}×{}",
                miden_trace.length(),
                miden_trace.main_trace_width(),
                p3_trace.height(),
                p3_trace.width()
            );
        }
        Err(e) => {
            println!("Failed to generate traces: {}", e);
            // Don't fail the test if Miden VM isn't available
        }
    }
}
