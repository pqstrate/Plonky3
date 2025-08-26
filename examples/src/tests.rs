//! Adding tests for all our End-to-End proofs.

use std::fmt::Debug;

// BabyBear field and its Poseidon2 implementations
use p3_baby_bear::{BabyBear, GenericPoseidon2LinearLayersBabyBear, Poseidon2BabyBear};
// Blake3 cryptographic hash function AIR
use p3_blake3_air::Blake3Air;
// Parallel radix-2 decimation-in-time DFT implementation
use p3_dft::Radix2DitParallel;
// Binomial extension field construction
use p3_field::extension::BinomialExtensionField;
// Keccak vectorization constants
use p3_keccak::VECTOR_LEN;
// Keccak cryptographic hash function AIR
use p3_keccak_air::KeccakAir;
// KoalaBear field and its Poseidon2 implementations
use p3_koala_bear::{GenericPoseidon2LinearLayersKoalaBear, KoalaBear, Poseidon2KoalaBear};
// Mersenne31 field and its Poseidon2 implementations
use p3_mersenne_31::{GenericPoseidon2LinearLayersMersenne31, Mersenne31, Poseidon2Mersenne31};
// Recursive DFT implementation for Montgomery form fields
use p3_monty_31::dft::RecursiveDft;
// Poseidon2 AIR implementations and round constants
use p3_poseidon2_air::{Poseidon2Air, RoundConstants, VectorizedPoseidon2Air};
// Deterministic random number generation for reproducible tests
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::dfts::DftChoice;
use crate::proofs::{
    prove_m31_keccak, prove_m31_poseidon2, prove_monty31_keccak, prove_monty31_poseidon2,
};

// 128 rows for the Generic Poseidon2 AIR.
// Wider traces will be made shorter to maintain reasonable proof times.
const TRACE_SIZE: usize = 1 << 7;

// General constants for constructing the Poseidon2 AIR.
// Poseidon2 state width (number of field elements)
const P2_WIDTH: usize = 16;
// Number of full rounds at the beginning and end of Poseidon2
const P2_HALF_FULL_ROUNDS: usize = 4;
// Log base 2 of vector length for SIMD operations
const P2_LOG_VECTOR_LEN: u8 = 3;
// Vector length for SIMD operations (8 elements)
const P2_VECTOR_LEN: usize = 1 << P2_LOG_VECTOR_LEN;

// Test vectorized Poseidon2 hashing with KoalaBear field, recursive DFT, and Poseidon2 Merkle tree
#[test]
fn test_end_to_end_koalabear_vectorized_poseidon2_hashes_recursive_dft_poseidon2_merkle_tree()
-> Result<(), impl Debug> {
    // WARNING: Use a real cryptographic PRNG in applications!!
    // Using deterministic seed for reproducible tests
    let mut rng = SmallRng::seed_from_u64(1);

    type EF = BinomialExtensionField<KoalaBear, 4>;

    let constants = RoundConstants::from_rng(&mut rng);
    const SBOX_DEGREE: u64 = 3;
    const SBOX_REGISTERS: usize = 0;
    const PARTIAL_ROUNDS: usize = 20;

    let proof_goal: VectorizedPoseidon2Air<
        KoalaBear,
        GenericPoseidon2LinearLayersKoalaBear,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
        P2_VECTOR_LEN,
    > = VectorizedPoseidon2Air::new(constants);

    let dft = DftChoice::Recursive(RecursiveDft::new(TRACE_SIZE >> VECTOR_LEN));

    let perm16 = Poseidon2KoalaBear::<16>::new_from_rng_128(&mut rng);
    let perm24 = Poseidon2KoalaBear::<24>::new_from_rng_128(&mut rng);

    prove_monty31_poseidon2::<_, EF, _, _, _, _>(proof_goal, dft, TRACE_SIZE, perm16, perm24)
}

// Test non-vectorized Poseidon2 hashing with KoalaBear field, recursive DFT, and Keccak Merkle tree
#[test]
fn test_end_to_end_koalabear_poseidon2_hashes_recursive_dft_keccak_merkle_tree()
-> Result<(), impl Debug> {
    let num_hashes = TRACE_SIZE;

    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    type EF = BinomialExtensionField<KoalaBear, 4>;

    let constants = RoundConstants::from_rng(&mut rng);
    const SBOX_DEGREE: u64 = 3;
    const SBOX_REGISTERS: usize = 0;
    const PARTIAL_ROUNDS: usize = 20;

    let proof_goal: Poseidon2Air<
        KoalaBear,
        GenericPoseidon2LinearLayersKoalaBear,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
    > = Poseidon2Air::new(constants);

    let dft = DftChoice::Recursive(RecursiveDft::new(TRACE_SIZE << 1));

    prove_monty31_keccak::<_, EF, _, _>(proof_goal, dft, num_hashes)
}

// Test Keccak hashing with KoalaBear field, parallel DFT, and Keccak Merkle tree
#[test]
fn test_end_to_end_koalabear_keccak_hashes_parallel_dft_keccak_merkle_tree()
-> Result<(), impl Debug> {
    // Keccak AIR has wider trace, so fewer hashes fit in the same trace size
    let num_hashes = TRACE_SIZE / 24;

    type EF = BinomialExtensionField<KoalaBear, 4>;

    let proof_goal = KeccakAir {};

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    prove_monty31_keccak::<_, EF, _, _>(proof_goal, dft, num_hashes)
}

#[test]
fn test_end_to_end_babybear_vectorized_poseidon2_hashes_recursive_dft_poseidon2_merkle_tree()
-> Result<(), impl Debug> {
    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    type EF = BinomialExtensionField<BabyBear, 4>;

    let constants = RoundConstants::from_rng(&mut rng);
    const SBOX_DEGREE: u64 = 3;
    const SBOX_REGISTERS: usize = 0;
    const PARTIAL_ROUNDS: usize = 20;

    let proof_goal: Poseidon2Air<
        BabyBear,
        GenericPoseidon2LinearLayersBabyBear,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
    > = Poseidon2Air::new(constants);

    let dft = DftChoice::Recursive(RecursiveDft::new(TRACE_SIZE << 1));

    let perm16 = Poseidon2BabyBear::<16>::new_from_rng_128(&mut rng);
    let perm24 = Poseidon2BabyBear::<24>::new_from_rng_128(&mut rng);

    prove_monty31_poseidon2::<_, EF, _, _, _, _>(proof_goal, dft, TRACE_SIZE, perm16, perm24)
}

#[test]
fn test_end_to_end_babybear_poseidon2_hashes_parallel_dft_poseidon2_merkle_tree()
-> Result<(), impl Debug> {
    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    type EF = BinomialExtensionField<BabyBear, 4>;

    let constants = RoundConstants::from_rng(&mut rng);
    const SBOX_DEGREE: u64 = 3;
    const SBOX_REGISTERS: usize = 0;
    const PARTIAL_ROUNDS: usize = 20;

    let proof_goal: Poseidon2Air<
        BabyBear,
        GenericPoseidon2LinearLayersBabyBear,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
    > = Poseidon2Air::new(constants);

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    let perm16 = Poseidon2BabyBear::<16>::new_from_rng_128(&mut rng);
    let perm24 = Poseidon2BabyBear::<24>::new_from_rng_128(&mut rng);

    prove_monty31_poseidon2::<_, EF, _, _, _, _>(proof_goal, dft, TRACE_SIZE, perm16, perm24)
}

#[test]
fn test_end_to_end_babybear_blake3_hashes_parallel_dft_poseidon2_merkle_tree()
-> Result<(), impl Debug> {
    let num_hashes = TRACE_SIZE >> 4;

    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    type EF = BinomialExtensionField<BabyBear, 4>;

    let proof_goal = Blake3Air {};

    let dft = DftChoice::Parallel(Radix2DitParallel::default());

    let perm16 = Poseidon2BabyBear::<16>::new_from_rng_128(&mut rng);
    let perm24 = Poseidon2BabyBear::<24>::new_from_rng_128(&mut rng);

    prove_monty31_poseidon2::<_, EF, _, _, _, _>(proof_goal, dft, num_hashes, perm16, perm24)
}

// Test Keccak hashing with Mersenne31 field and Circle PCS using Keccak Merkle tree
#[test]
fn test_end_to_end_mersenne_31_keccak_hashes_keccak_merkle_tree() -> Result<(), impl Debug> {
    // Keccak AIR has wider trace, so fewer hashes fit in the same trace size
    let num_hashes = TRACE_SIZE / 24;
    let proof_goal = KeccakAir {};

    prove_m31_keccak(proof_goal, num_hashes)
}

// Test Blake3 hashing with Mersenne31 field and Circle PCS using Keccak Merkle tree
#[test]
fn test_end_to_end_mersenne31_blake3_hashes_keccak_merkle_tree() -> Result<(), impl Debug> {
    // Blake3 AIR is quite wide, so use fewer hashes
    let num_hashes = TRACE_SIZE >> 4;
    let proof_goal = Blake3Air {};

    prove_m31_keccak(proof_goal, num_hashes)
}

#[test]
fn test_end_to_end_mersenne31_vectorized_poseidon2_hashes_poseidon2_merkle_tree()
-> Result<(), impl Debug> {
    type EF = BinomialExtensionField<Mersenne31, 3>;

    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    let constants = RoundConstants::from_rng(&mut rng);

    // Field specific constants for constructing the Poseidon2 AIR.
    const SBOX_DEGREE: u64 = 5;
    const SBOX_REGISTERS: usize = 1;
    const PARTIAL_ROUNDS: usize = 14;

    let proof_goal: VectorizedPoseidon2Air<
        Mersenne31,
        GenericPoseidon2LinearLayersMersenne31,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
        P2_VECTOR_LEN,
    > = VectorizedPoseidon2Air::new(constants);

    let perm16 = Poseidon2Mersenne31::<16>::new_from_rng_128(&mut rng);
    let perm24 = Poseidon2Mersenne31::<24>::new_from_rng_128(&mut rng);

    prove_m31_poseidon2::<_, EF, _, _, _>(proof_goal, TRACE_SIZE, perm16, perm24)
}

#[test]
fn test_end_to_end_mersenne31_poseidon2_hashes_keccak_merkle_tree() -> Result<(), impl Debug> {
    // WARNING: Use a real cryptographic PRNG in applications!!
    let mut rng = SmallRng::seed_from_u64(1);

    let constants = RoundConstants::from_rng(&mut rng);

    // Field specific constants for constructing the Poseidon2 AIR.
    const SBOX_DEGREE: u64 = 5;
    const SBOX_REGISTERS: usize = 1;
    const PARTIAL_ROUNDS: usize = 14;

    let proof_goal: Poseidon2Air<
        Mersenne31,
        GenericPoseidon2LinearLayersMersenne31,
        P2_WIDTH,
        SBOX_DEGREE,
        SBOX_REGISTERS,
        P2_HALF_FULL_ROUNDS,
        PARTIAL_ROUNDS,
    > = Poseidon2Air::new(constants);

    prove_m31_keccak(proof_goal, TRACE_SIZE)
}
