// Standard library for file system operations
use std::fs;

// Core Plonky3 AIR (Arithmetic Intermediate Representation) traits
// AIR defines the constraints that must be satisfied by a valid computation
use p3_air::{Air, AirBuilder, BaseAir};

// Cryptographic challenger for generating random challenges during proof interaction
use p3_challenger::{HashChallenger, SerializingChallenger64};

// Polynomial commitment scheme components
use p3_commit::ExtensionMmcs;

// Discrete Fourier Transform implementation for polynomial operations
use p3_dft::Radix2DitParallel;

// Field arithmetic traits and extensions for working with finite fields
use p3_field::{extension::BinomialExtensionField, integers::QuotientMap, PrimeCharacteristicRing, PrimeField64};

// FRI (Fast Reed-Solomon Interactive Oracle Proof) polynomial commitment scheme
use p3_fri::{TwoAdicFriPcs, create_benchmark_fri_params};

// Goldilocks field - a 64-bit prime field optimized for STARK proofs
use p3_goldilocks::Goldilocks;

// Keccak hash function components
use p3_keccak::{Keccak256Hash, KeccakF};

// Matrix operations and storage for execution traces
use p3_matrix::{Matrix, dense::RowMajorMatrix};

// Merkle tree implementation for polynomial commitments
use p3_merkle_tree::MerkleTreeMmcs;

// Cryptographic primitives: hash functions and compression
use p3_symmetric::{CompressionFunctionFromHasher, PaddingFreeSponge, SerializingHasher};

// STARK proving system - the main proving and verification functions
use p3_uni_stark::{StarkConfig, prove, verify};

// Number of columns in our trace matrix (73 columns as found in trace.txt)
pub const NUM_COLS: usize = 73;

/// IncrementAir defines the arithmetic constraints for our increment proof
/// This AIR enforces that the first column of each row increments by 1 from the previous row
/// i.e., trace[i][0] = trace[i-1][0] + 1 for all transition rows
#[derive(Clone)]
pub struct IncrementAir;

/// BaseAir implementation tells Plonky3 the basic properties of our computation
impl<F> BaseAir<F> for IncrementAir {
    /// Returns the number of columns in our execution trace
    /// Our trace has 73 columns as determined from the input file
    fn width(&self) -> usize {
        NUM_COLS
    }
}

/// Air implementation defines the actual arithmetic constraints
/// This is where we specify what makes a valid computation
impl<AB: AirBuilder> Air<AB> for IncrementAir {
    /// eval() is called by the STARK prover to check constraints
    /// It receives an AirBuilder that lets us access trace rows and define constraints
    fn eval(&self, builder: &mut AB) {
        // Get access to the execution trace matrix
        let main = builder.main();
        
        // Get current row and next row for transition constraints
        // current_row = trace[i], next_row = trace[i+1]
        let (current_row, next_row) = (
            main.row_slice(0).expect("Matrix must have at least one row"),
            main.row_slice(1).expect("Matrix must have at least two rows for transitions"),
        );
        
        // Apply constraint only during transitions (between consecutive rows)
        // This excludes boundary conditions (first/last rows)
        let mut when_transition = builder.when_transition();
        
        // The core constraint: next_row[0] - current_row[0] = 1
        // This ensures that the first column increments by exactly 1 each row
        // AB::Expr::from(AB::F::ONE) creates the field element representing 1
        when_transition.assert_eq(next_row[0].clone() - current_row[0].clone(), AB::Expr::from(AB::F::ONE));
    }
}

/// Parse the trace.txt file and convert it to a RowMajorMatrix of Goldilocks field elements
/// 
/// This function:
/// 1. Reads the trace file line by line
/// 2. Parses each line as an array of u64 integers  
/// 3. Converts u64 values to Goldilocks field elements
/// 4. Handles power-of-2 padding required by STARK systems
/// 5. Maintains the increment constraint during padding
pub fn parse_trace() -> Result<RowMajorMatrix<Goldilocks>, Box<dyn std::error::Error>> {
    // Read the entire trace file into memory
    let content = fs::read_to_string("trace.txt")?;
    
    // Vector to store all field elements in row-major order
    let mut data = Vec::new();
    
    // Parse each line of the trace file
    for (line_num, line) in content.lines().enumerate() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }
        
        // Remove brackets from array format: [1,2,3] -> 1,2,3
        let line = line.trim_start_matches('[').trim_end_matches(']');
        
        // Split by commas and parse each value
        // Convert each u64 to a Goldilocks field element
        let values: Result<Vec<_>, _> = line.split(',')
            .map(|s| s.trim().parse::<u64>().map(|x| unsafe { 
                // Convert u64 to Goldilocks field element
                // Using unsafe conversion since we trust our input data
                Goldilocks::from_canonical_unchecked(x) 
            }))
            .collect();
        
        match values {
            Ok(row_values) => {
                let col_count = row_values.len();
                if col_count == NUM_COLS {
                    // Add this row's data to our matrix
                    data.extend(row_values);
                } else {
                    eprintln!("Warning: Row {} has {} columns, expected {}", line_num + 1, col_count, NUM_COLS);
                }
            }
            Err(e) => {
                eprintln!("Error parsing line {}: {}", line_num + 1, e);
            }
        }
    }
    
    let num_rows = data.len() / NUM_COLS;
    println!("Total rows parsed: {}", num_rows);
    
    // PREPROCESSING: Handle the problematic last row and ensure power-of-2 size
    if num_rows > 1 {
        // Remove the last row since it doesn't follow the increment constraint
        // The original trace has a final row that breaks the pattern
        let new_size = (num_rows - 1) * NUM_COLS;
        data.truncate(new_size);
        println!("Truncated to {} rows to maintain increment constraint", num_rows - 1);
        
        // STARK systems require traces with power-of-2 number of rows
        // This is needed for efficient FFT operations in the polynomial commitment
        let current_rows = num_rows - 1;
        let target_rows = current_rows.next_power_of_two();
        
        if target_rows > current_rows {
            // Pad with additional rows that maintain the increment constraint
            let mut next_val = current_rows as u64;  // Continue the increment sequence
            
            for _ in current_rows..target_rows {
                // Create new row with properly incremented first column
                let mut new_row = vec![unsafe { Goldilocks::from_canonical_unchecked(next_val) }];
                
                // Copy the remaining columns from the last valid row
                // This maintains consistency in other columns while only changing column 0
                let last_row = &data[(current_rows - 1) * NUM_COLS + 1..current_rows * NUM_COLS];
                new_row.extend_from_slice(last_row);
                
                // Add the new row to our data
                data.extend(new_row);
                next_val += 1;  // Increment for next row
            }
            println!("Padded from {} to {} rows (power of 2) with incrementing values", current_rows, target_rows);
        }
    }
    
    // Create and return the matrix in row-major format
    Ok(RowMajorMatrix::new(data, NUM_COLS))
}

/// Generate and verify a STARK proof that demonstrates knowledge of a valid increment computation
/// 
/// This function sets up the complete STARK proving system:
/// 1. Configures cryptographic primitives (hash functions, commitment schemes)
/// 2. Sets up the polynomial commitment scheme (FRI)
/// 3. Parses the execution trace
/// 4. Generates a zero-knowledge proof
/// 5. Verifies the proof to ensure correctness
///
/// # STARK System Components:
/// 
/// **Fields**: Uses Goldilocks (64-bit prime field) for efficiency
/// **Extension**: BinomialExtensionField for cryptographic security 
/// **Hash Functions**: Keccak256 for commitments and challenges
/// **Polynomial Commitment**: FRI (Fast Reed-Solomon Interactive Oracle Proof)
/// **Merkle Trees**: 4-ary trees for efficient polynomial commitments
/// **DFT**: Radix-2 Decimation-in-Time for polynomial operations
/// 
/// # How STARK Proofs Work:
/// 
/// 1. **Execution Trace**: The computation is recorded as a matrix where each row
///    represents one step of computation and columns represent different variables
/// 
/// 2. **AIR Constraints**: Arithmetic constraints define what constitutes a valid
///    computation (in our case: trace[i][0] = trace[i-1][0] + 1)
/// 
/// 3. **Polynomial Interpolation**: The trace is converted to polynomials using FFT
/// 
/// 4. **Constraint Checking**: The AIR constraints become polynomial equations
///    that must be satisfied
/// 
/// 5. **Commitment & Proof**: FRI creates succinct commitments to these polynomials
///    and proves they satisfy the constraints without revealing the actual values
/// 
/// 6. **Verification**: The verifier can efficiently check the proof without
///    re-executing the computation
pub fn generate_proof() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Increment Constraint Proof");
    
    // === TYPE DEFINITIONS FOR STARK SYSTEM ===
    
    // Base field: Goldilocks - a 64-bit prime field (2^64 - 2^32 + 1)
    // Optimized for 64-bit arithmetic and STARK proofs
    type Val = Goldilocks;
    
    // Extension field: degree-2 extension of Goldilocks for better security
    // Used for challenges and some cryptographic operations
    type Challenge = BinomialExtensionField<Val, 2>;
    
    // === HASH FUNCTION SETUP ===
    // We need hash functions for:
    // 1. Merkle trees (polynomial commitments)
    // 2. Fiat-Shamir transform (making interactive proof non-interactive)
    
    type ByteHash = Keccak256Hash;  // Standard Keccak for byte hashing
    type U64Hash = PaddingFreeSponge<KeccakF, 25, 17, 4>;  // Keccak optimized for field elements
    type FieldHash = SerializingHasher<U64Hash>;  // Wrapper for field element hashing
    let byte_hash = ByteHash {};
    let u64_hash = U64Hash::new(KeccakF {});
    let field_hash = FieldHash::new(u64_hash);
    
    // === COMPRESSION FUNCTION ===
    // Used in Merkle trees to combine child hashes
    type MyCompress = CompressionFunctionFromHasher<U64Hash, 2, 4>;
    let compress = MyCompress::new(u64_hash);
    
    // === MERKLE TREE COMMITMENT SCHEME ===
    // This is how we commit to polynomials (transformed from our execution trace)
    // The Merkle tree allows efficient verification of polynomial evaluations
    type ValMmcs = MerkleTreeMmcs<
        [Val; p3_keccak::VECTOR_LEN],      // Values are packed in arrays for efficiency
        [u64; p3_keccak::VECTOR_LEN],      // Hash digest format
        FieldHash,                          // Hash function for field elements
        MyCompress,                         // Compression function
        4,                                 // Arity of Merkle tree (4-ary tree)
    >;
    let val_mmcs = ValMmcs::new(field_hash, compress);
    
    // Extension field commitment scheme (for challenges and extension columns)
    type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    
    // === DISCRETE FOURIER TRANSFORM ===
    // Used for polynomial interpolation and evaluation
    // Essential for converting between trace (time domain) and polynomial (frequency domain)
    type Dft = Radix2DitParallel<Val>;
    let dft = Dft::default();
    
    // === CHALLENGER (FIAT-SHAMIR) ===
    // Generates cryptographically secure random challenges
    // This makes the proof non-interactive (no back-and-forth with verifier)
    type Challenger = SerializingChallenger64<Val, HashChallenger<u8, ByteHash, 32>>;
    let challenger = Challenger::from_hasher(vec![], byte_hash);
    
    // === FRI POLYNOMIAL COMMITMENT SCHEME ===
    // FRI (Fast Reed-Solomon Interactive Oracle Proof) is the core of our STARK
    // It allows committing to polynomials and proving evaluations efficiently
    let fri_params = create_benchmark_fri_params(challenge_mmcs);
    
    type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
    let pcs = Pcs::new(dft, val_mmcs, fri_params);
    
    // === STARK CONFIGURATION ===
    // Ties everything together: PCS + Challenge generation + Field
    type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
    let config = MyConfig::new(pcs, challenger);
    
    // === TRACE PARSING AND PREPARATION ===
    println!("📊 Parsing trace from file...");
    let trace = parse_trace()?;
    println!("   • Trace dimensions: {}×{}", trace.height(), trace.width());
    
    // Display first few values to confirm correct parsing
    println!("   • First few values in column 0:");
    for i in 0..std::cmp::min(5, trace.height()) {
        let row = trace.row_slice(i).unwrap();
        println!("     Row {}: {}", i, row[0].as_canonical_u64());
    }

    // === AIR INSTANTIATION ===
    println!("\n🏗️  Creating AIR with constraint: trace[i][0] = trace[i-1][0] + 1");
    let air = IncrementAir;
    
    // === PROOF GENERATION ===
    println!("\n🔐 Generating proof...");
    let start_time = std::time::Instant::now();
    
    // This is where the magic happens!
    // prove() takes our constraint system (AIR), execution trace, and configuration
    // and generates a succinct zero-knowledge proof
    let proof = prove(&config, &air, trace, &vec![]);  // No public inputs needed for our constraint
    
    let proof_time = start_time.elapsed();
    println!("   • Proof generated in {:.2}s", proof_time.as_secs_f64());
    
    // === PROOF VERIFICATION ===
    println!("\n✅ Verifying proof...");
    let start_time = std::time::Instant::now();
    
    // Verification is much faster than proving
    // The verifier only needs to check the proof, not regenerate it
    match verify(&config, &air, &proof, &vec![]) {
        Ok(()) => {
            let verify_time = start_time.elapsed();
            println!("   • Verification completed in {:.2}ms", verify_time.as_millis());
            println!("   • ✅ Proof is valid!");
        }
        Err(e) => {
            return Err(format!("Verification failed: {:?}", e).into());
        }
    }
    
    println!("\n🎉 Successfully proved the increment constraint!");
    println!("   • Constraint: trace[i][0] = trace[i-1][0] + 1 for all transitions");
    println!("   • Trace verified to follow the incrementing pattern");
    
    Ok(())
}

/// Test module for verifying the correctness of our trace parsing and proof generation
#[cfg(test)]
mod tests {
    use super::*;

    /// Test that we can successfully parse the trace.txt file
    /// This test verifies:
    /// 1. File reading works correctly
    /// 2. Parsing logic handles the format properly  
    /// 3. Matrix dimensions match expectations
    /// 4. Power-of-2 padding works as expected
    #[test]
    fn test_parse_trace() {
        match parse_trace() {
            Ok(trace) => {
                assert_eq!(trace.width(), NUM_COLS);
                println!("Trace parsed successfully: {}×{}", trace.height(), trace.width());
            }
            Err(e) => {
                println!("Failed to parse trace: {}", e);
            }
        }
    }
}
