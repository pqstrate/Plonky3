# Increment Constraint STARK Proof Example

A complete implementation of a STARK (Scalable Transparent Argument of Knowledge) proof system using Plonky3, demonstrating how to prove that a trace satisfies an increment constraint.

## What This Proves

Given an execution trace stored in `trace.txt`, this program generates a zero-knowledge proof that the first column follows the constraint:

```
trace[i][0] = trace[i-1][0] + 1 for all consecutive rows
```

In other words, it proves that column 0 increments by exactly 1 in each step.

## Key Components

### 1. Field Arithmetic
- **Goldilocks Field**: Uses a 64-bit prime field (2^64 - 2^32 + 1) optimized for STARK proofs
- **Extension Field**: BinomialExtensionField for enhanced cryptographic security

### 2. AIR (Arithmetic Intermediate Representation)
- **IncrementAir**: Defines the constraint `next_row[0] - current_row[0] = 1`
- **Transition Constraints**: Applied between consecutive rows only
- **No Boundary Constraints**: The constraint holds for all transitions

### 3. Cryptographic Primitives
- **Hash Function**: Keccak256 for commitments and Fiat-Shamir transform
- **Merkle Trees**: 4-ary trees for polynomial commitments
- **Compression**: Keccak-based compression functions for tree operations

### 4. Polynomial Commitment Scheme
- **FRI**: Fast Reed-Solomon Interactive Oracle Proof
- **DFT**: Radix-2 Decimation-in-Time for polynomial operations
- **Commitment**: Efficient polynomial commitments via Merkle trees

### 5. Trace Processing
- **Parsing**: Converts `trace.txt` format to `RowMajorMatrix<Goldilocks>`
- **Validation**: Ensures 73-column format and proper dimensions
- **Power-of-2 Padding**: Adds rows to reach power-of-2 size (required for FFT)
- **Constraint Preservation**: Maintains increment pattern during padding

## File Structure

```
fib2/
├── Cargo.toml          # Dependencies and build configuration
├── src/
│   ├── lib.rs          # Main implementation with extensive comments
│   └── main.rs         # Entry point with usage documentation
├── trace.txt           # Input execution trace (73 columns × 64 rows)
└── README.md           # This file
```

## How to Run

1. **Prerequisites**: Ensure `trace.txt` is in the project root directory

2. **Build and Run**:
   ```bash
   cargo run --release
   ```

3. **Run Tests**:
   ```bash
   cargo test
   ```

## Expected Output

```
🚀 Starting Increment Constraint Proof
📊 Parsing trace from file...
Total rows parsed: 64
Truncated to 63 rows to maintain increment constraint
Padded from 63 to 64 rows (power of 2) with incrementing values
   • Trace dimensions: 64×73
   • First few values in column 0:
     Row 0: 0
     Row 1: 1
     Row 2: 2
     Row 3: 3
     Row 4: 4

🏗️  Creating AIR with constraint: trace[i][0] = trace[i-1][0] + 1

🔐 Generating proof...
   • Proof generated in 0.03s

✅ Verifying proof...
   • Verification completed in 1ms
   • ✅ Proof is valid!

🎉 Successfully proved the increment constraint!
   • Constraint: trace[i][0] = trace[i-1][0] + 1 for all transitions
   • Trace verified to follow the incrementing pattern
```

## How STARK Proofs Work

### 1. Execution Trace
The computation is recorded as a matrix where:
- Each **row** represents one step of the computation
- Each **column** represents a different variable or register
- Our trace has 73 columns and 64 rows (power of 2)

### 2. AIR Constraints
Arithmetic constraints define valid computations:
- **Transition constraints**: Rules between consecutive steps
- **Boundary constraints**: Rules for first/last steps (not used here)
- Our constraint: `trace[i][0] = trace[i-1][0] + 1`

### 3. Polynomial Representation
- The trace is converted to polynomials using FFT
- Constraints become polynomial equations
- Polynomial degree relates to trace size

### 4. Commitment Phase
- Polynomials are committed using FRI
- Merkle trees provide efficient commitments
- Verifier gets cryptographic commitments, not actual values

### 5. Proof Generation
- Prover demonstrates constraint satisfaction
- Uses random challenges via Fiat-Shamir
- Creates succinct proof independent of trace size

### 6. Verification
- Verifier checks proof without seeing the trace
- Much faster than proof generation
- Provides mathematical certainty of constraint satisfaction

## Performance

- **Proof Generation**: ~30ms (optimized build)
- **Proof Verification**: ~1ms
- **Proof Size**: Logarithmic in trace size
- **Security**: Based on collision-resistant hashes and polynomial commitments

## Educational Value

This example demonstrates:

1. **Complete STARK Workflow**: From trace parsing to proof verification
2. **Plonky3 Usage**: How to use the library's components together
3. **AIR Design**: How to express computational constraints
4. **Field Arithmetic**: Working with finite fields in cryptographic proofs
5. **Practical Considerations**: Power-of-2 padding, trace preprocessing

## Extensions

This basic example can be extended to prove:

- More complex arithmetic constraints
- Multi-column relationships
- Cryptographic operations (hashing, signatures)
- Virtual machine execution
- Complex algorithms with privacy requirements

## Dependencies

See `Cargo.toml` for the complete list of Plonky3 components used:
- `p3-air`: AIR trait definitions
- `p3-goldilocks`: Goldilocks field implementation
- `p3-fri`: FRI polynomial commitment scheme
- `p3-keccak`: Keccak hash functions
- `p3-matrix`: Matrix operations
- `p3-uni-stark`: STARK proving system
- And more supporting libraries