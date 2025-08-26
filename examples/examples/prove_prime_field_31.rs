//! Command-line tool for generating and verifying STARK proofs of cryptographic hash functions.
//!
//! This tool demonstrates the Plonky3 STARK proving system by generating proofs that
//! a specified number of hash function evaluations have been computed correctly.
//! It supports multiple hash functions (Blake3, Keccak-f, Poseidon2), prime fields
//! (BabyBear, KoalaBear, Mersenne31), and different polynomial commitment schemes.

// Command-line argument parsing
use clap::Parser;
// BabyBear prime field and its Poseidon2 implementations
use p3_baby_bear::{BabyBear, GenericPoseidon2LinearLayersBabyBear, Poseidon2BabyBear};
// Blake3 cryptographic hash function AIR
use p3_blake3_air::Blake3Air;
// Parallel radix-2 decimation-in-time DFT
use p3_dft::Radix2DitParallel;
// Example library exports: AIR enum wrapper
use p3_examples::airs::ProofObjective;
// Example library exports: DFT implementation wrapper
use p3_examples::dfts::DftChoice;
// Example library exports: command-line parsing enums
use p3_examples::parsers::{DftOptions, FieldOptions, MerkleHashOptions, ProofOptions};
// Example library exports: proof generation and verification functions
use p3_examples::proofs::{
    prove_m31_keccak, prove_m31_poseidon2, prove_monty31_keccak, prove_monty31_poseidon2,
    report_result,
};
// Binomial extension field construction
use p3_field::extension::BinomialExtensionField;
// Keccak-f permutation AIR
use p3_keccak_air::KeccakAir;
// KoalaBear prime field and its Poseidon2 implementations
use p3_koala_bear::{GenericPoseidon2LinearLayersKoalaBear, KoalaBear, Poseidon2KoalaBear};
// Mersenne31 prime field and its Poseidon2 implementations
use p3_mersenne_31::{GenericPoseidon2LinearLayersMersenne31, Mersenne31, Poseidon2Mersenne31};
// Recursive DFT implementation for Montgomery form fields
use p3_monty_31::dft::RecursiveDft;
// Poseidon2 round constants and vectorized AIR
use p3_poseidon2_air::{RoundConstants, VectorizedPoseidon2Air};
// Deterministic random number generation for reproducible examples
use rand::SeedableRng;
use rand::rngs::SmallRng;
// Structured logging with forest-like output
use tracing_forest::ForestLayer;
use tracing_forest::util::LevelFilter;
// Tracing subscriber for logging configuration
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

// General constants for constructing the Poseidon2 AIR.
// Width of Poseidon2 permutation state (number of field elements)
const P2_WIDTH: usize = 16;
// Number of full rounds at the beginning and end of Poseidon2
const P2_HALF_FULL_ROUNDS: usize = 4;
// Log base 2 of vector length for SIMD operations
const P2_LOG_VECTOR_LEN: u8 = 3;
// Vector length for batch processing (8 parallel Poseidon2 instances)
const P2_VECTOR_LEN: usize = 1 << P2_LOG_VECTOR_LEN;

/// Command-line arguments for configuring STARK proof generation.
/// 
/// This structure defines all the configurable parameters for generating
/// cryptographic hash function proofs using the Plonky3 STARK system.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The prime field to use for STARK arithmetic.
    /// 
    /// Different fields have different security properties and performance characteristics:
    /// - BabyBear: 31-bit prime, good for general use
    /// - KoalaBear: 31-bit prime, optimized for certain operations  
    /// - Mersenne31: 2^31-1, allows for very efficient arithmetic
    #[arg(short, long, ignore_case = true, value_enum)]
    field: FieldOptions,

    /// The cryptographic hash function to prove.
    /// 
    /// Each hash function has different execution trace widths and complexities:
    /// - Blake3: Wide trace, modern cryptographic hash
    /// - KeccakF: Keccak-f[1600] permutation, used in SHA-3
    /// - Poseidon2: Arithmetic-friendly hash, efficient in STARK circuits
    #[arg(short, long, ignore_case = true, value_enum)]
    objective: ProofOptions,

    /// The log base 2 of the desired execution trace length.
    /// 
    /// Larger traces allow proving more hash computations but require more time.
    /// For example: 10 = 1024 rows, 15 = 32768 rows
    #[arg(short, long)]
    log_trace_length: u8,

    /// The discrete Fourier transform implementation to use.
    /// 
    /// DFT is used in polynomial operations during proof generation:
    /// - RecursiveDft: Memory-efficient for smaller traces
    /// - Radix2DitParallel: Faster for larger traces with parallelization
    /// - None: Use default (required for Mersenne31)
    #[arg(short, long, ignore_case = true, value_enum, default_value_t = DftOptions::None)]
    discrete_fourier_transform: DftOptions,

    /// The hash function to use for Merkle tree construction.
    /// 
    /// The Merkle tree is used in the polynomial commitment scheme:
    /// - KeccakF: Uses Keccak-f[1600] permutation
    /// - Poseidon2: Uses arithmetic-friendly Poseidon2 hash
    #[arg(short, long, ignore_case = true, value_enum)]
    merkle_hash: MerkleHashOptions,
}

fn main() {
    // Initialize structured logging with environment-based filtering
    // Default to INFO level, but can be overridden with RUST_LOG environment variable
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    // Set up the tracing subscriber with forest-style output for better readability
    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    // Calculate the actual trace height from the logarithmic input
    let trace_height = 1 << args.log_trace_length;

    // Calculate the number of hash operations based on the chosen hash function
    // Each hash function has different trace width requirements
    let num_hashes = match args.objective {
        ProofOptions::Blake3Permutations => {
            // Blake3 has a moderate trace width, so we can do one hash per trace row
            println!("Proving 2^{} Blake-3 permutations", {
                args.log_trace_length
            });
            trace_height
        }
        ProofOptions::Poseidon2Permutations => {
            // Poseidon2 uses vectorization to pack multiple hashes per trace row
            // We can fit P2_VECTOR_LEN (8) Poseidon2 evaluations per row
            println!("Proving 2^{} native Poseidon-2 permutations", {
                args.log_trace_length + P2_LOG_VECTOR_LEN
            });
            trace_height << P2_LOG_VECTOR_LEN
        }
        ProofOptions::KeccakFPermutations => {
            // Keccak-f has a very wide trace (24 columns per round, 24 rounds)
            // So we can only fit a fraction of hashes compared to the trace height
            let num_hashes = trace_height / 24;
            println!("Proving {num_hashes} Keccak-F permutations");
            num_hashes
        }
    };

    // WARNING: Use a real cryptographic PRNG in applications!!
    // This uses a deterministic seed for reproducible examples and testing
    let mut rng = SmallRng::seed_from_u64(1);

    // Match on the selected prime field and configure the proof system accordingly
    // Each field requires different extension field configurations and parameters
    match args.field {
        FieldOptions::KoalaBear => {
            // Use degree-4 extension of KoalaBear for enhanced security margins
            type EF = BinomialExtensionField<KoalaBear, 4>;

            // Create the appropriate AIR (Algebraic Intermediate Representation) based on the objective
            let proof_goal = match args.objective {
                ProofOptions::Blake3Permutations => ProofObjective::Blake3(Blake3Air {}),
                ProofOptions::KeccakFPermutations => ProofObjective::Keccak(KeccakAir {}),
                ProofOptions::Poseidon2Permutations => {
                    let constants = RoundConstants::from_rng(&mut rng);

                    // Field specific constants for constructing the Poseidon2 AIR.
                    const SBOX_DEGREE: u64 = 3;
                    const SBOX_REGISTERS: usize = 0;
                    const PARTIAL_ROUNDS: usize = 20;

                    let p2_air: VectorizedPoseidon2Air<
                        KoalaBear,
                        GenericPoseidon2LinearLayersKoalaBear,
                        P2_WIDTH,
                        SBOX_DEGREE,
                        SBOX_REGISTERS,
                        P2_HALF_FULL_ROUNDS,
                        PARTIAL_ROUNDS,
                        P2_VECTOR_LEN,
                    > = VectorizedPoseidon2Air::new(constants);
                    ProofObjective::Poseidon2(p2_air)
                }
            };

            let dft = match args.discrete_fourier_transform {
                DftOptions::RecursiveDft => {
                    DftChoice::Recursive(RecursiveDft::new(trace_height << 1))
                }
                DftOptions::Radix2DitParallel => DftChoice::Parallel(Radix2DitParallel::default()),
                DftOptions::None => panic!(
                    "Please specify what dft to use. Options are recursive-dft and radix-2-dit-parallel"
                ),
            };

            match args.merkle_hash {
                MerkleHashOptions::KeccakF => {
                    let result = prove_monty31_keccak::<_, EF, _, _>(proof_goal, dft, num_hashes);
                    report_result(result);
                }
                MerkleHashOptions::Poseidon2 => {
                    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng);
                    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng);
                    let result = prove_monty31_poseidon2::<_, EF, _, _, _, _>(
                        proof_goal, dft, num_hashes, perm16, perm24,
                    );
                    report_result(result);
                }
            };
        }
        FieldOptions::BabyBear => {
            // Use degree-4 extension of BabyBear field
            type EF = BinomialExtensionField<BabyBear, 4>;

            let proof_goal = match args.objective {
                ProofOptions::Blake3Permutations => ProofObjective::Blake3(Blake3Air {}),
                ProofOptions::KeccakFPermutations => ProofObjective::Keccak(KeccakAir {}),
                ProofOptions::Poseidon2Permutations => {
                    let constants = RoundConstants::from_rng(&mut rng);

                    // BabyBear-specific constants for constructing the Poseidon2 AIR
                    // These parameters are optimized for the BabyBear field characteristics
                    const SBOX_DEGREE: u64 = 7;        // Degree-7 S-box (x^7)
                    const SBOX_REGISTERS: usize = 1;    // Partial S-box (only 1 element per round)
                    const PARTIAL_ROUNDS: usize = 13;   // Fewer partial rounds due to higher S-box degree

                    let p2_air: VectorizedPoseidon2Air<
                        BabyBear,
                        GenericPoseidon2LinearLayersBabyBear,
                        P2_WIDTH,
                        SBOX_DEGREE,
                        SBOX_REGISTERS,
                        P2_HALF_FULL_ROUNDS,
                        PARTIAL_ROUNDS,
                        P2_VECTOR_LEN,
                    > = VectorizedPoseidon2Air::new(constants);
                    ProofObjective::Poseidon2(p2_air)
                }
            };

            let dft = match args.discrete_fourier_transform {
                DftOptions::RecursiveDft => {
                    DftChoice::Recursive(RecursiveDft::new(trace_height << 1))
                }
                DftOptions::Radix2DitParallel => DftChoice::Parallel(Radix2DitParallel::default()),
                DftOptions::None => panic!(
                    "Please specify what dft to use. Options are recursive-dft and radix-2-dit-parallel"
                ),
            };

            match args.merkle_hash {
                MerkleHashOptions::KeccakF => {
                    let result = prove_monty31_keccak::<_, EF, _, _>(proof_goal, dft, num_hashes);
                    report_result(result);
                }
                MerkleHashOptions::Poseidon2 => {
                    let perm16 = Poseidon2BabyBear::<16>::new_from_rng_128(&mut rng);
                    let perm24 = Poseidon2BabyBear::<24>::new_from_rng_128(&mut rng);
                    let result = prove_monty31_poseidon2::<_, EF, _, _, _, _>(
                        proof_goal, dft, num_hashes, perm16, perm24,
                    );
                    report_result(result);
                }
            };
        }
        FieldOptions::Mersenne31 => {
            // Use degree-3 extension of Mersenne31 field for optimal performance
            type EF = BinomialExtensionField<Mersenne31, 3>;

            let proof_goal = match args.objective {
                ProofOptions::Blake3Permutations => ProofObjective::Blake3(Blake3Air {}),
                ProofOptions::KeccakFPermutations => ProofObjective::Keccak(KeccakAir {}),
                ProofOptions::Poseidon2Permutations => {
                    let constants = RoundConstants::from_rng(&mut rng);

                    // Mersenne31-specific constants for constructing the Poseidon2 AIR
                    // These parameters leverage Mersenne31's efficient arithmetic properties
                    const SBOX_DEGREE: u64 = 5;        // Degree-5 S-box (x^5)
                    const SBOX_REGISTERS: usize = 1;    // Partial S-box layer
                    const PARTIAL_ROUNDS: usize = 14;   // Balanced security/performance tradeoff

                    let p2_air: VectorizedPoseidon2Air<
                        Mersenne31,
                        GenericPoseidon2LinearLayersMersenne31,
                        P2_WIDTH,
                        SBOX_DEGREE,
                        SBOX_REGISTERS,
                        P2_HALF_FULL_ROUNDS,
                        PARTIAL_ROUNDS,
                        P2_VECTOR_LEN,
                    > = VectorizedPoseidon2Air::new(constants);
                    ProofObjective::Poseidon2(p2_air)
                }
            };

            // Mersenne31 uses Circle PCS which doesn't require explicit DFT selection
            match args.discrete_fourier_transform {
                DftOptions::None => {
                    // This is expected for Mersenne31 - Circle PCS handles polynomial operations
                }
                _ => panic!(
                    "Currently there are no available DFT options when using Mersenne31. Please remove the --discrete_fourier_transform flag."
                ),
            };

            // Execute proof generation using Circle PCS (no DFT parameter needed)
            match args.merkle_hash {
                MerkleHashOptions::KeccakF => {
                    // Use Mersenne31 with Circle PCS and Keccak Merkle tree
                    let result = prove_m31_keccak(proof_goal, num_hashes);
                    report_result(result);
                }
                MerkleHashOptions::Poseidon2 => {
                    // Use Mersenne31 with Circle PCS and Poseidon2 Merkle tree
                    let perm16 = Poseidon2Mersenne31::<16>::new_from_rng_128(&mut rng);
                    let perm24 = Poseidon2Mersenne31::<24>::new_from_rng_128(&mut rng);
                    let result = prove_m31_poseidon2::<_, EF, _, _, _>(
                        proof_goal, num_hashes, perm16, perm24,
                    );
                    report_result(result);
                }
            };
        }
    }
}
