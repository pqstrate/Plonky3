use p3_blake3::Blake3;
use p3_challenger::{HashChallenger, SerializingChallenger64};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_fri::TwoAdicFriPcs;
use p3_goldilocks::Goldilocks;
use p3_keccak::{Keccak256Hash, KeccakF};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher};
use p3_uni_stark::StarkConfig;

// Number of columns in our trace matrix (will be updated dynamically based on Miden trace)
pub const NUM_COLS: usize = 80; // Updated to match Miden VM trace width

// Number of columns for synthetic Plonky3 traces
pub const SYNTHETIC_TRACE_COLS: usize = 4;

// Number of Fibonacci steps to compute in the Miden program
pub const FIBONACCI_STEPS: usize = 70;

// Type aliases for cleaner signatures
// Base field: Goldilocks - a 64-bit prime field (2^64 - 2^32 + 1)
// Optimized for 64-bit arithmetic and STARK proofs
pub type Val = Goldilocks;

// Extension field: degree-2 extension of Goldilocks for better security
// Used for challenges and some cryptographic operations
pub type Challenge = BinomialExtensionField<Val, 2>;

// We need hash functions for:
// 1. Merkle trees (polynomial commitments)
// 2. Fiat-Shamir transform (making interactive proof non-interactive)

pub type ByteHash = Keccak256Hash; // Standard Keccak for byte hashing
pub type U64Hash = PaddingFreeSponge<KeccakF, 25, 17, 4>; // Keccak optimized for field elements
pub type FieldHash = SerializingHasher<U64Hash>; // Wrapper for field element hashing
pub type MyCompress = CompressionFunctionFromHasher<U64Hash, 2, 4>;
pub type ValMmcs = MerkleTreeMmcs<
    [Val; p3_keccak::VECTOR_LEN],
    [u64; p3_keccak::VECTOR_LEN],
    FieldHash,
    MyCompress,
    4,
>;
pub type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
pub type Dft = Radix2DitParallel<Val>;
pub type Challenger = SerializingChallenger64<Val, HashChallenger<u8, ByteHash, 32>>;
pub type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
pub type KeccakConfig = StarkConfig<Pcs, Challenge, Challenger>;

// Blake3-specific type definitions - using Blake3 for byte hashing like Keccak256Hash
pub type Blake3ByteHash = Blake3;
pub type Blake3U64Hash = PaddingFreeSponge<KeccakF, 25, 17, 4>; // Use KeccakF for field elements
pub type Blake3FieldHash = SerializingHasher<Blake3U64Hash>;
pub type Blake3Compress = CompressionFunctionFromHasher<Blake3U64Hash, 2, 4>;
pub type Blake3ValMmcs = MerkleTreeMmcs<
    [Val; p3_keccak::VECTOR_LEN],
    [u64; p3_keccak::VECTOR_LEN],
    Blake3FieldHash,
    Blake3Compress,
    4,
>;
pub type Blake3ChallengeMmcs = ExtensionMmcs<Val, Challenge, Blake3ValMmcs>;
pub type Blake3Challenger = SerializingChallenger64<Val, HashChallenger<u8, Blake3ByteHash, 32>>;
pub type Blake3Pcs = TwoAdicFriPcs<Val, Dft, Blake3ValMmcs, Blake3ChallengeMmcs>;
pub type Blake3Config = StarkConfig<Blake3Pcs, Challenge, Blake3Challenger>;
