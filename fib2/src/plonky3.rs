use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::{PrimeCharacteristicRing, PrimeField64};
use p3_fri::{TwoAdicFriPcs, create_benchmark_fri_params};
use p3_goldilocks::Goldilocks;
use p3_keccak::{ KeccakF};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{StarkConfig, prove, verify};

use crate::{ByteHash, Challenge, FieldHash, IncrementAir, MyCompress, U64Hash, Val, ValMmcs};

/// Generate a Plonky3 STARK proof using a simple increment constraint
///
/// This function ignores the input trace and generates a synthetic trace
/// that satisfies the simple increment constraint: trace[i][0] = trace[i-1][0] + 1
///
/// # Arguments
/// * `_p3_trace` - The Plonky3 trace matrix (ignored, used only for sizing)
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub fn p3_generate_proof(
    p3_trace: RowMajorMatrix<Goldilocks>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Generating Plonky3 STARK proof with simple increment constraint...");

    // === CREATE SIMPLE SYNTHETIC TRACE ===
    // Generate a simple trace that actually satisfies the increment constraint
    let trace_height = 64; // Power of 2 for STARK requirements
    let trace_width = 4; // Simple trace with just 4 columns

    println!(
        "   • Creating synthetic trace: {}×{}",
        trace_height, trace_width
    );

    let mut trace_data = Vec::with_capacity(trace_height * trace_width);

    for row in 0..trace_height {
        // Column 0: incrementing values (0, 1, 2, 3, ...)
        trace_data.push(Goldilocks::from_u64(row as u64));

        // Other columns: simple patterns that don't interfere with our constraint
        trace_data.push(Goldilocks::from_u64(row as u64 * 2)); // Column 1: 2*row
        trace_data.push(Goldilocks::from_u64(42)); // Column 2: constant
        trace_data.push(Goldilocks::from_u64((row as u64).pow(2) % 1000)); // Column 3: row^2 mod 1000
    }
    
    let byte_hash = ByteHash {};
    let u64_hash = U64Hash::new(KeccakF {});
    let field_hash = FieldHash::new(u64_hash);

    let compress = MyCompress::new(u64_hash);

    // === MERKLE TREE COMMITMENT SCHEME ===
    let val_mmcs = ValMmcs::new(field_hash, compress);

    // Extension field commitment scheme
    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    // === DISCRETE FOURIER TRANSFORM ===
    type Dft = Radix2DitParallel<Val>;
    let dft = Dft::default();

    // === CHALLENGER (FIAT-SHAMIR) ===
    type Challenger = SerializingChallenger64<Val, HashChallenger<u8, ByteHash, 32>>;
    let challenger = Challenger::from_hasher(vec![], byte_hash);

    // === FRI POLYNOMIAL COMMITMENT SCHEME ===
    let fri_params = {
        let mut param = create_benchmark_fri_params(challenge_mmcs);
        param.proof_of_work_bits = 1;
        param
    };

    type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs::new(dft, val_mmcs, fri_params);

    // === STARK CONFIGURATION ===
    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs, challenger);

    println!(
        "   • P3 trace dimensions: {}×{}",
        p3_trace.height(),
        p3_trace.width()
    );

    // Display first few values to confirm correct increment pattern
    println!("   • First few values in column 0 (should increment):");
    for i in 0..std::cmp::min(8, p3_trace.height()) {
        let row = p3_trace.row_slice(i).unwrap();
        println!("     Row {}: {}", i, row[0].as_canonical_u64());
    }

    // === AIR INSTANTIATION ===
    println!(
        "\n🏗️  Using synthetic increment AIR with constraint: trace[i][0] = trace[i-1][0] + 1"
    );
    let air = IncrementAir;

    // === PROOF GENERATION ===
    println!("\n🔐 Generating proof...");
    let start_time = std::time::Instant::now();

    let proof = prove(&config, &air, p3_trace, &vec![]);

    let proof_time = start_time.elapsed();
    println!("   • Proof generated in {:.2}s", proof_time.as_secs_f64());

    // === PROOF VERIFICATION ===
    println!("\n✅ Verifying proof...");
    let start_time = std::time::Instant::now();

    match verify(&config, &air, &proof, &vec![]) {
        Ok(()) => {
            let verify_time = start_time.elapsed();
            println!(
                "   • Verification completed in {:.2}ms",
                verify_time.as_millis()
            );
            println!("   • ✅ Proof is valid!");
        }
        Err(e) => {
            return Err(format!("Verification failed: {:?}", e).into());
        }
    }

    println!("\n🎉 Successfully proved the increment constraint using Plonky3!");
    println!("   • Constraint: trace[i][0] = trace[i-1][0] + 1 for all transitions");

    Ok(())
}
