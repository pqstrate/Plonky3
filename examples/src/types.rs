//! This contains a large number of type definitions which help simplify the code in other files and keep clippy happy.
//!
//! In particular this builds up to defining the types `KeccakStarkConfig`,
//! `KeccakCircleStarkConfig`, `Poseidon2StarkConfig`, `Poseidon2CircleStarkConfig`.
//! These are needed to define our proof functions.

// Challenger implementations for Fiat-Shamir transformations in STARK protocols
use p3_challenger::{DuplexChallenger, HashChallenger, SerializingChallenger32};
// Circle-based polynomial commitment scheme for certain field types
use p3_circle::CirclePcs;
// Extension field Multi-linear Merkle Commitment Scheme
use p3_commit::ExtensionMmcs;
// Core field trait definitions
use p3_field::Field;
// FRI (Fast Reed-Solomon Interactive Oracle Proof) polynomial commitment scheme
use p3_fri::TwoAdicFriPcs;
// Keccak-256 hash function and Keccak-f permutation implementations
use p3_keccak::{Keccak256Hash, KeccakF};
// Merkle tree-based multi-linear commitment scheme
use p3_merkle_tree::MerkleTreeMmcs;
// Symmetric cryptographic primitives: sponges, compression functions, etc.
use p3_symmetric::{
    CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher, TruncatedPermutation,
};
// STARK protocol configuration type
use p3_uni_stark::StarkConfig;

// Types related to using Keccak in the Merkle tree.
// Vector length for Keccak SIMD optimizations
const KECCAK_VECTOR_LEN: usize = p3_keccak::VECTOR_LEN;
// Keccak compression function: takes 2 field elements, outputs 4 field elements
pub(crate) type KeccakCompressionFunction =
    CompressionFunctionFromHasher<PaddingFreeSponge<KeccakF, 25, 17, 4>, 2, 4>;
// Merkle tree MMCS using Keccak with vectorized field elements
// Uses 4 output words from the compression function
pub(crate) type KeccakMerkleMmcs<F> = MerkleTreeMmcs<
    [F; KECCAK_VECTOR_LEN],
    [u64; KECCAK_VECTOR_LEN],
    SerializingHasher<PaddingFreeSponge<KeccakF, 25, 17, 4>>,
    KeccakCompressionFunction,
    4,
>;

// Complete STARK configuration using Keccak for Merkle trees and FRI-based PCS
pub(crate) type KeccakStarkConfig<F, EF, DFT> = StarkConfig<
    TwoAdicFriPcs<F, DFT, KeccakMerkleMmcs<F>, ExtensionMmcs<F, EF, KeccakMerkleMmcs<F>>>,
    EF,
    SerializingChallenger32<F, HashChallenger<u8, Keccak256Hash, 32>>,
>;
// STARK configuration using Circle PCS instead of FRI, with Keccak Merkle trees
pub(crate) type KeccakCircleStarkConfig<F, EF> = StarkConfig<
    CirclePcs<F, KeccakMerkleMmcs<F>, ExtensionMmcs<F, EF, KeccakMerkleMmcs<F>>>,
    EF,
    SerializingChallenger32<F, HashChallenger<u8, Keccak256Hash, 32>>,
>;

// Types related to using Poseidon2 in the Merkle tree.
// Poseidon2 sponge: 24-element state, 16-element rate, 8-element output
pub(crate) type Poseidon2Sponge<Perm24> = PaddingFreeSponge<Perm24, 24, 16, 8>;
// Poseidon2 compression function: takes 2 field elements, outputs 8, using 16-width permutation
pub(crate) type Poseidon2Compression<Perm16> = TruncatedPermutation<Perm16, 2, 8, 16>;
// Merkle tree MMCS using Poseidon2 with field packing optimizations
// Uses 8 output elements from the compression function
pub(crate) type Poseidon2MerkleMmcs<F, Perm16, Perm24> = MerkleTreeMmcs<
    <F as Field>::Packing,
    <F as Field>::Packing,
    Poseidon2Sponge<Perm24>,
    Poseidon2Compression<Perm16>,
    8,
>;
// Complete STARK configuration using Poseidon2 for Merkle trees and FRI-based PCS
// Uses duplex challenger with Poseidon2 permutation
pub(crate) type Poseidon2StarkConfig<F, EF, DFT, Perm16, Perm24> = StarkConfig<
    TwoAdicFriPcs<
        F,
        DFT,
        Poseidon2MerkleMmcs<F, Perm16, Perm24>,
        ExtensionMmcs<F, EF, Poseidon2MerkleMmcs<F, Perm16, Perm24>>,
    >,
    EF,
    DuplexChallenger<F, Perm24, 24, 16>,
>;
// STARK configuration using Circle PCS instead of FRI, with Poseidon2 Merkle trees
// Uses duplex challenger with Poseidon2 permutation
pub(crate) type Poseidon2CircleStarkConfig<F, EF, Perm16, Perm24> = StarkConfig<
    CirclePcs<
        F,
        Poseidon2MerkleMmcs<F, Perm16, Perm24>,
        ExtensionMmcs<F, EF, Poseidon2MerkleMmcs<F, Perm16, Perm24>>,
    >,
    EF,
    DuplexChallenger<F, Perm24, 24, 16>,
>;
