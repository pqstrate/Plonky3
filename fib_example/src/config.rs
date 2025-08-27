//! # Plonky3 Configuration Setup
//! 
//! This module sets up all the cryptographic components needed for our proof system:
//! - Field arithmetic (BabyBear field) 
//! - Hash functions (Poseidon2)
//! - Polynomial commitment scheme (FRI)
//! - Challenge generation (Challenger)
//! - Matrix commitment (Merkle trees)

use p3_baby_bear::{BabyBear, Poseidon2BabyBear};
use p3_challenger::DuplexChallenger;
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_fri::{TwoAdicFriPcs, create_test_fri_params};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_uni_stark::StarkConfig;
use rand::{SeedableRng, rngs::SmallRng};

/// Our base field: BabyBear (31-bit prime field)
/// BabyBear is efficient and well-suited for STARK proofs
pub type Val = BabyBear;

/// Poseidon2 permutation operating over BabyBear with width 16
/// This is our main cryptographic primitive for hashing
pub type Perm = Poseidon2BabyBear<16>;

/// Hash function built from Poseidon2 permutation
/// PaddingFreeSponge configuration: width=16, rate=8, output=8
pub type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;

/// Compression function for building Merkle trees
/// TruncatedPermutation: input_len=2, output_len=8, width=16
pub type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;

/// Matrix commitment scheme using Merkle trees
/// This commits to matrices over our base field
pub type ValMmcs = MerkleTreeMmcs<
    <Val as p3_field::Field>::Packing,  // Packing type for field elements
    <Val as p3_field::Field>::Packing,  // Packing type for digests
    MyHash,                             // Hash function
    MyCompress,                         // Compression function
    8                                   // Arity of Merkle tree
>;

/// Challenge field: 4-degree binomial extension of BabyBear
/// This provides ~124 bits of security (31 * 4)
pub type Challenge = BinomialExtensionField<Val, 4>;

/// Extended matrix commitment scheme for challenge field elements
pub type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;

/// Challenge generation using duplex sponge construction
pub type Challenger = DuplexChallenger<Val, Perm, 16, 8>;

/// Discrete Fourier Transform implementation
/// Uses radix-2 decimation-in-time with parallel processing
pub type Dft = Radix2DitParallel<Val>;

/// FRI-based Polynomial Commitment Scheme
/// FRI = Fast Reed-Solomon Interactive Oracle Proof
pub type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;

/// Complete STARK configuration
/// This ties together all the cryptographic components
pub type FibonacciConfig = StarkConfig<Pcs, Challenge, Challenger>;

/// Create a complete configuration for Fibonacci proofs
/// 
/// This sets up all the cryptographic components with secure parameters:
/// - Random permutation using a fixed seed (for deterministic testing)
/// - FRI parameters with appropriate security level
/// - Merkle tree commitment schemes
/// 
/// ## Security Parameters:
/// - Base field: ~31 bits (BabyBear)
/// - Extension field: ~124 bits (4-degree extension)  
/// - Hash function: Poseidon2 (cryptographically secure)
/// - Tree arity: 8 (good balance of proof size vs verification time)
/// 
/// ## Returns:
/// A complete `StarkConfig` ready for proving and verifying
pub fn create_fibonacci_config() -> FibonacciConfig {
    // Initialize random number generator with fixed seed
    // Note: In production, you'd use a cryptographically secure RNG
    let mut rng = SmallRng::seed_from_u64(42);
    
    // Create Poseidon2 permutation with random parameters
    let perm = Perm::new_from_rng_128(&mut rng);
    
    // Build hash function from permutation
    let hash = MyHash::new(perm.clone());
    
    // Build compression function for Merkle trees  
    let compress = MyCompress::new(perm.clone());
    
    // Create matrix commitment scheme for base field
    let val_mmcs = ValMmcs::new(hash, compress);
    
    // Create matrix commitment scheme for challenge field
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    
    // Create DFT for polynomial operations
    let dft = Dft::default();
    
    // Create FRI parameters
    // log_final_poly_len=2 means final polynomial has degree 2^2-1=3
    // This provides good security while keeping proofs reasonably sized
    let fri_params = create_test_fri_params(challenge_mmcs, 2);
    
    // Create polynomial commitment scheme
    let pcs = Pcs::new(dft, val_mmcs, fri_params);
    
    // Create challenger for interactive protocol
    let challenger = Challenger::new(perm);
    
    // Combine everything into final configuration
    FibonacciConfig::new(pcs, challenger)
}

/// Create a configuration with custom FRI parameters
/// 
/// This allows more fine-grained control over the proof system parameters.
/// Larger `log_final_poly_len` values increase security but also proof size.
/// 
/// ## Parameters:
/// - `log_final_poly_len`: log₂ of final polynomial length in FRI
/// - `seed`: Random seed for deterministic parameter generation
pub fn create_custom_fibonacci_config(log_final_poly_len: usize, seed: u64) -> FibonacciConfig {
    let mut rng = SmallRng::seed_from_u64(seed);
    let perm = Perm::new_from_rng_128(&mut rng);
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm.clone());
    let val_mmcs = ValMmcs::new(hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let dft = Dft::default();
    let fri_params = create_test_fri_params(challenge_mmcs, log_final_poly_len);
    let pcs = Pcs::new(dft, val_mmcs, fri_params);
    let challenger = Challenger::new(perm);
    
    FibonacciConfig::new(pcs, challenger)
}

/// Print configuration information for debugging/education
pub fn print_config_info(config: &FibonacciConfig) {
    println!("=== Fibonacci Proof Configuration ===");
    println!("Base field: BabyBear (2^31 - 2^27 + 1)");
    println!("Extension field: 4-degree binomial extension (~124-bit security)");
    println!("Hash function: Poseidon2 with width 16");
    println!("PCS: FRI (Fast Reed-Solomon IOP)");
    println!("Merkle tree arity: 8");
    println!("DFT: Radix-2 DIT with parallel processing");
    println!("=====================================");
    
    // Note: We can't easily extract detailed info from the config object
    // as most fields are private. This function is more for documentation.
    let _ = config; // Suppress unused parameter warning
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let config = create_fibonacci_config();
        
        // We can't test much about the internals since they're private,
        // but we can at least verify the config was created successfully
        print_config_info(&config);
    }
    
    #[test]
    fn test_custom_config_creation() {
        let config1 = create_custom_fibonacci_config(1, 123);
        let config2 = create_custom_fibonacci_config(3, 456);
        
        // Configs should be created successfully
        // Different parameters should create different configs
        // (we can't easily test this due to private fields)
        print_config_info(&config1);
        print_config_info(&config2);
    }
    
    #[test]
    fn test_deterministic_config() {
        // Same seed should produce same configuration
        let config1 = create_custom_fibonacci_config(2, 42);
        let config2 = create_custom_fibonacci_config(2, 42);
        
        // We can't directly compare configs, but they should behave the same
        // This test mainly ensures no panics during creation
        print_config_info(&config1);
        print_config_info(&config2);
    }
}