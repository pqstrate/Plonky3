use core::fmt::Debug;

// Challenger implementations for Fiat-Shamir transformations
use p3_challenger::{DuplexChallenger, SerializingChallenger32};
// Circle-based polynomial commitment scheme
use p3_circle::CirclePcs;
// Extension field MMCS (Merkle Multi-linear Commitment Scheme)
use p3_commit::ExtensionMmcs;
// Discrete Fourier Transform trait for two-adic subgroups
use p3_dft::TwoAdicSubgroupDft;
// Field extension implementations
use p3_field::extension::{BinomialExtensionField, ComplexExtendable};
// Core field traits
use p3_field::{ExtensionField, Field, PrimeField32, PrimeField64, TwoAdicField};
// FRI (Fast Reed-Solomon Interactive Oracle Proof) polynomial commitment scheme
use p3_fri::{TwoAdicFriPcs, create_benchmark_fri_params};
// Keccak hash function implementations
use p3_keccak::{Keccak256Hash, KeccakF};
// Mersenne31 prime field implementation
use p3_mersenne_31::Mersenne31;
// Symmetric cryptographic primitives
use p3_symmetric::{CryptographicPermutation, PaddingFreeSponge, SerializingHasher};
// STARK proof system implementation
use p3_uni_stark::{Proof, StarkGenericConfig, prove, verify};
// Random number generation for field elements
use rand::distr::StandardUniform;
use rand::prelude::Distribution;

use crate::airs::ExampleHashAir;
use crate::types::{
    KeccakCircleStarkConfig, KeccakCompressionFunction, KeccakMerkleMmcs, KeccakStarkConfig,
    Poseidon2CircleStarkConfig, Poseidon2Compression, Poseidon2MerkleMmcs, Poseidon2Sponge,
    Poseidon2StarkConfig,
};

/// Produce a MerkleTreeMmcs which uses the KeccakF permutation.
/// Creates a Merkle tree commitment scheme using Keccak-f[1600] as the hash function.
const fn get_keccak_mmcs<F: Field>() -> KeccakMerkleMmcs<F> {
    // Configure Keccak sponge with 25 words state, 17 rate, 4 output words
    let u64_hash = PaddingFreeSponge::<KeccakF, 25, 17, 4>::new(KeccakF {});

    // Wrap the sponge for field element hashing
    let field_hash = SerializingHasher::new(u64_hash);

    // Create compression function for Merkle tree internal nodes
    let compress = KeccakCompressionFunction::new(u64_hash);

    KeccakMerkleMmcs::new(field_hash, compress)
}

/// Produce a MerkleTreeMmcs from a pair of cryptographic field permutations.
///
/// The first permutation will be used for compression and the second for sponge hashing.
/// Currently this is only intended to be used with a pair of Poseidon2 hashes with width 16 and 24
/// but this can easily be generalised in future if we desire.
const fn get_poseidon2_mmcs<
    F: Field,
    Perm16: CryptographicPermutation<[F; 16]> + CryptographicPermutation<[F::Packing; 16]>,
    Perm24: CryptographicPermutation<[F; 24]> + CryptographicPermutation<[F::Packing; 24]>,
>(
    perm16: Perm16,
    perm24: Perm24,
) -> Poseidon2MerkleMmcs<F, Perm16, Perm24> {
    // Create sponge hasher using the 24-width permutation
    let hash = Poseidon2Sponge::new(perm24);

    // Create compression function using the 16-width permutation
    let compress = Poseidon2Compression::new(perm16);

    Poseidon2MerkleMmcs::<F, _, _>::new(hash, compress)
}

/// Prove the given ProofGoal using the Keccak hash function to build the merkle tree.
///
/// This allows the user to choose:
/// - The Field
/// - The Proof Goal (Choice of both hash function and desired number of hashes to prove)
/// - The DFT
#[inline]
pub fn prove_monty31_keccak<
    F: PrimeField32 + TwoAdicField,
    EF: ExtensionField<F>,
    DFT: TwoAdicSubgroupDft<F>,
    PG: ExampleHashAir<F, KeccakStarkConfig<F, EF, DFT>>,
>(
    proof_goal: PG,
    dft: DFT,
    num_hashes: usize,
) -> Result<(), impl Debug>
where
    StandardUniform: Distribution<F>,
{
    // Set up Keccak-based Merkle tree for polynomial commitments
    let val_mmcs = get_keccak_mmcs();

    // Create extension field MMCS for challenges and quotient polynomials
    let challenge_mmcs = ExtensionMmcs::<F, EF, _>::new(val_mmcs.clone());
    let fri_params = create_benchmark_fri_params(challenge_mmcs);

    // Generate execution trace for the specified number of hash operations
    let trace = proof_goal.generate_trace_rows(num_hashes, fri_params.log_blowup);

    // Create FRI-based polynomial commitment scheme
    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_params);
    // Initialize Fiat-Shamir challenger using Keccak-256
    let challenger = SerializingChallenger32::from_hasher(vec![], Keccak256Hash {});

    let config = KeccakStarkConfig::new(pcs, challenger);

    // Generate STARK proof
    let proof = prove(&config, &proof_goal, trace, &vec![]);
    report_proof_size(&proof);

    // Verify the generated proof
    verify(&config, &proof_goal, &proof, &vec![])
}

/// Prove the given ProofGoal using the Poseidon2 hash function to build the merkle tree.
///
/// This allows the user to choose:
/// - The Field
/// - The Proof Goal (Choice of Hash function and number of hashes to prove)
/// - The DFT
#[inline]
pub fn prove_monty31_poseidon2<
    F: PrimeField32 + TwoAdicField,
    EF: ExtensionField<F>,
    DFT: TwoAdicSubgroupDft<F>,
    Perm16: CryptographicPermutation<[F; 16]> + CryptographicPermutation<[F::Packing; 16]>,
    Perm24: CryptographicPermutation<[F; 24]> + CryptographicPermutation<[F::Packing; 24]>,
    PG: ExampleHashAir<F, Poseidon2StarkConfig<F, EF, DFT, Perm16, Perm24>>,
>(
    proof_goal: PG,
    dft: DFT,
    num_hashes: usize,
    perm16: Perm16,
    perm24: Perm24,
) -> Result<(), impl Debug>
where
    StandardUniform: Distribution<F>,
{
    let val_mmcs = get_poseidon2_mmcs::<F, _, _>(perm16, perm24.clone());

    let challenge_mmcs = ExtensionMmcs::<F, EF, _>::new(val_mmcs.clone());
    let fri_params = create_benchmark_fri_params(challenge_mmcs);

    let trace = proof_goal.generate_trace_rows(num_hashes, fri_params.log_blowup);

    let pcs = TwoAdicFriPcs::new(dft, val_mmcs, fri_params);
    let challenger = DuplexChallenger::new(perm24);

    let config = Poseidon2StarkConfig::new(pcs, challenger);

    let proof = prove(&config, &proof_goal, trace, &vec![]);
    report_proof_size(&proof);

    verify(&config, &proof_goal, &proof, &vec![])
}

/// Prove the given ProofGoal using the Keccak hash function to build the merkle tree.
///
/// This fixes the field and Mersenne31 and makes use of the circle stark.
///
/// It currently allows the user to choose:
/// - The Proof Goal (Choice of Hash function and number of hashes to prove)
#[inline]
pub fn prove_m31_keccak<
    PG: ExampleHashAir<
            Mersenne31,
            KeccakCircleStarkConfig<Mersenne31, BinomialExtensionField<Mersenne31, 3>>,
        >,
>(
    proof_goal: PG,
    num_hashes: usize,
) -> Result<(), impl Debug> {
    type F = Mersenne31;
    type EF = BinomialExtensionField<Mersenne31, 3>;

    let val_mmcs = get_keccak_mmcs();
    let challenge_mmcs = ExtensionMmcs::<F, EF, _>::new(val_mmcs.clone());
    let fri_params = create_benchmark_fri_params(challenge_mmcs);

    let trace = proof_goal.generate_trace_rows(num_hashes, fri_params.log_blowup);

    // for (index, row) in trace.row_slices().enumerate() {
    //     println!("trace[{}] {} elems: {:?}", index, row.len(), row);
    // }

    println!("Trace done");

    let pcs = CirclePcs::new(val_mmcs, fri_params);
    let challenger = SerializingChallenger32::from_hasher(vec![], Keccak256Hash {});

    let config = KeccakCircleStarkConfig::new(pcs, challenger);

    let proof = prove(&config, &proof_goal, trace, &vec![]);
    report_proof_size(&proof);

    verify(&config, &proof_goal, &proof, &vec![])
}

/// Prove the given ProofGoal using the Keccak hash function to build the merkle tree.
///
/// This fixes the field and Mersenne31 and makes use of the circle stark.
///
/// It currently allows the user to choose:
/// - The Proof Goal (Choice of Hash function and number of hashes to prove)
#[inline]
pub fn prove_m31_poseidon2<
    F: PrimeField64 + ComplexExtendable,
    EF: ExtensionField<F>,
    Perm16: CryptographicPermutation<[F; 16]> + CryptographicPermutation<[F::Packing; 16]>,
    Perm24: CryptographicPermutation<[F; 24]> + CryptographicPermutation<[F::Packing; 24]>,
    PG: ExampleHashAir<F, Poseidon2CircleStarkConfig<F, EF, Perm16, Perm24>>,
>(
    proof_goal: PG,
    num_hashes: usize,
    perm16: Perm16,
    perm24: Perm24,
) -> Result<(), impl Debug>
where
    StandardUniform: Distribution<F>,
{
    let val_mmcs = get_poseidon2_mmcs::<F, _, _>(perm16, perm24.clone());

    let challenge_mmcs = ExtensionMmcs::<F, EF, _>::new(val_mmcs.clone());
    let fri_params = create_benchmark_fri_params(challenge_mmcs);

    let trace = proof_goal.generate_trace_rows(num_hashes, fri_params.log_blowup);

    let pcs = CirclePcs::new(val_mmcs, fri_params);
    let challenger = DuplexChallenger::new(perm24);

    let config = Poseidon2CircleStarkConfig::new(pcs, challenger);

    let proof = prove(&config, &proof_goal, trace, &vec![]);
    report_proof_size(&proof);

    verify(&config, &proof_goal, &proof, &vec![])
}

/// Report the result of the proof.
///
/// Either print that the proof was successful or panic and return the error.
/// This is a utility function for demonstration purposes.
#[inline]
pub fn report_result(result: Result<(), impl Debug>) {
    if let Err(e) = result {
        panic!("{e:?}");
    } else {
        println!("Proof Verified Successfully")
    }
}

/// Report the size of the serialized proof.
///
/// Serializes the given proof instance using bincode and prints the size in bytes.
/// This helps evaluate the efficiency of different proof configurations.
/// Panics if serialization fails.
#[inline]
pub fn report_proof_size<SC>(proof: &Proof<SC>)
where
    SC: StarkGenericConfig,
{
    // Configure bincode for consistent serialization
    let config = bincode::config::standard()
        .with_little_endian()
        .with_fixed_int_encoding();
    // Serialize proof to measure its size
    let proof_bytes =
        bincode::serde::encode_to_vec(proof, config).expect("Failed to serialize proof");
    println!("Proof size: {} bytes", proof_bytes.len());
}
