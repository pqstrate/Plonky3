use p3_fri::FriParameters;
use p3_keccak::KeccakF;
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_uni_stark::{StarkGenericConfig, prove, verify};

use crate::{
    Blake3ByteHash, Blake3ChallengeMmcs, Blake3Challenger, Blake3Compress, Blake3Config,
    Blake3FieldHash, Blake3Pcs, Blake3U64Hash, Blake3ValMmcs, ByteHash, ChallengeMmcs, Challenger,
    Dft, FieldHash, IncrementAir, KeccakConfig, MyCompress, Pcs, U64Hash, Val, ValMmcs,
};

/// Create a Keccak-based configuration for Plonky3 STARK proofs
pub fn create_keccak_config() -> KeccakConfig {
    let byte_hash = ByteHash {};
    let u64_hash = U64Hash::new(KeccakF {});
    let field_hash = FieldHash::new(u64_hash);
    let compress = MyCompress::new(u64_hash);

    // === MERKLE TREE COMMITMENT SCHEME ===
    let val_mmcs = ValMmcs::new(field_hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());

    // === DISCRETE FOURIER TRANSFORM ===
    let dft = Dft::default();

    // === CHALLENGER (FIAT-SHAMIR) ===
    let challenger = Challenger::from_hasher(vec![], byte_hash);

    // === FRI POLYNOMIAL COMMITMENT SCHEME ===
    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 100,
        proof_of_work_bits: 1,
        mmcs: challenge_mmcs,
    };

    let pcs = Pcs::new(dft, val_mmcs, fri_params);

    // === STARK CONFIGURATION ===
    KeccakConfig::new(pcs, challenger)
}

/// Create a Blake3-based configuration for Plonky3 STARK proofs
pub fn create_blake3_config() -> Blake3Config {
    let blake3_byte_hash = Blake3ByteHash {};
    let blake3_u64_hash = Blake3U64Hash::new(KeccakF {});
    let field_hash = Blake3FieldHash::new(blake3_u64_hash);
    let compress = Blake3Compress::new(blake3_u64_hash);

    // === MERKLE TREE COMMITMENT SCHEME ===
    let val_mmcs = Blake3ValMmcs::new(field_hash, compress);
    let challenge_mmcs = Blake3ChallengeMmcs::new(val_mmcs.clone());

    // === DISCRETE FOURIER TRANSFORM ===
    let dft = Dft::default();

    // === CHALLENGER (FIAT-SHAMIR) ===
    let challenger = Blake3Challenger::from_hasher(vec![], blake3_byte_hash);

    // === FRI POLYNOMIAL COMMITMENT SCHEME ===
    let fri_params = FriParameters {
        log_blowup: 1,
        log_final_poly_len: 0,
        num_queries: 100,
        proof_of_work_bits: 1,
        mmcs: challenge_mmcs,
    };

    let pcs = Blake3Pcs::new(dft, val_mmcs, fri_params);

    // === STARK CONFIGURATION ===
    Blake3Config::new(pcs, challenger)
}

/// Generate a Plonky3 STARK proof using Keccak hash function
pub fn p3_generate_proof_keccak(
    p3_trace: RowMajorMatrix<Val>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = create_keccak_config();
    p3_generate_proof_with_config(p3_trace, config, "Keccak")
}

/// Generate a Plonky3 STARK proof using Blake3 hash function
pub fn p3_generate_proof_blake3(
    p3_trace: RowMajorMatrix<Val>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = create_blake3_config();
    p3_generate_proof_with_config(p3_trace, config, "Blake3")
}

/// Generic proof generation function that works with any StarkGenericConfig
fn p3_generate_proof_with_config<C: StarkGenericConfig>(
    p3_trace: RowMajorMatrix<p3_uni_stark::Val<C>>,
    config: C,
    hash_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "   • P3 trace dimensions: {}×{}",
        p3_trace.height(),
        p3_trace.width()
    );

    // === AIR INSTANTIATION ===
    tracing::info!(
        "\n🏗️  Using synthetic increment AIR with constraint: trace[i][0] = trace[i-1][0] + 1"
    );
    let air = IncrementAir;

    // === PROOF GENERATION ===
    tracing::info!("\n🔐 Generating proof with {}...", hash_name);
    let start_time = std::time::Instant::now();

    let proof = prove(&config, &air, p3_trace, &vec![]);

    let proof_time = start_time.elapsed();
    tracing::info!("   • Proof generated in {:.2}s", proof_time.as_secs_f64());

    // === PROOF VERIFICATION ===
    tracing::info!("\n✅ Verifying proof...");
    let start_time = std::time::Instant::now();

    match verify(&config, &air, &proof, &vec![]) {
        Ok(()) => {
            let verify_time = start_time.elapsed();
            tracing::info!(
                "   • Verification completed in {:.2}ms",
                verify_time.as_millis()
            );
            tracing::info!("   • ✅ Proof is valid!");
        }
        Err(e) => {
            return Err(format!("Verification failed: {:?}", e).into());
        }
    }

    tracing::info!(
        "\n🎉 Successfully proved the increment constraint using Plonky3 with {}!",
        hash_name
    );
    tracing::info!("   • Constraint: trace[i][0] = trace[i-1][0] + 1 for all transitions");

    Ok(())
}
