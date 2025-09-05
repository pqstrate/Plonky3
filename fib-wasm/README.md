# Fibonacci WASM Prover

A WebAssembly implementation of a Fibonacci proof generator using Plonky3 with Goldilocks field and Keccak hash.

## Features

- **Field**: Goldilocks (64-bit prime field)
- **Hash Function**: Keccak-256  
- **PCS**: FRI (Fast Reed-Solomon Interactive Oracle Proof)
- **WebAssembly**: Runs in the browser for zero-knowledge proof generation

## Building

### Prerequisites

1. Install `wasm-pack`:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Ensure you have the Rust toolchain with wasm32 target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Build the WASM module

```bash
./build.sh
```

This will:
- Build the WASM module using `wasm-pack`
- Generate JavaScript bindings in the `pkg/` directory
- Create an `index.html` example file

## Running

Start a local web server:

```bash
python3 -m http.server 8000
```

Then open `http://localhost:8000` in your browser.

## Usage

The WASM module exports three main functions:

### `fibonacci(n: number): number`
Calculates the nth Fibonacci number for reference.
- `n`: Must be between 0 and 47 (u32 overflow protection)

### `prove_fibonacci(n: number, expected_result: number): string`
Generates a **real zero-knowledge proof** that the nth Fibonacci number equals the expected result.
- `n`: Must be between 1 and 8 (optimized for WASM performance)
- `expected_result`: Must fit in u32 range (0 to 4,294,967,295)

**Note**: 
- Generates actual ZK proofs using Plonky3 STARK system
- The trace is automatically padded to the next power of 2
- Proof generation takes a few seconds in the browser

### `get_prover_info(): string`
Returns information about the prover configuration.

## Example Usage

```javascript
import init, { fibonacci, prove_fibonacci, get_prover_info } from './pkg/fib_wasm.js';

await init();

// Calculate F(10)
const result = fibonacci(10); // Returns 55

// Generate and verify proof for F(8) = 21
try {
    const proof_result = prove_fibonacci(8, 21);
    console.log(proof_result); // "Proof generated and verified successfully for F(8) = 21"
} catch (error) {
    console.error("Proof failed:", error);
}
```

## Technical Details

The implementation uses:
- **Fibonacci AIR (Algebraic Intermediate Representation)**: Defines the constraints for the Fibonacci recurrence relation
- **Goldilocks Field**: 64-bit prime field optimized for efficiency
- **Keccak-256**: Cryptographic hash function for Merkle tree commitments
- **FRI PCS**: Polynomial commitment scheme with excellent concrete efficiency
- **Two-adic domain**: Leverages efficient FFTs for polynomial operations

The proof demonstrates knowledge of a sequence of Fibonacci numbers without revealing intermediate values, proving that F(n) equals the expected result.