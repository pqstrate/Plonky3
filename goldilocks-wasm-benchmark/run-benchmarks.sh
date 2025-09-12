#!/bin/bash

echo "📊 Running Native Benchmark..."
echo "------------------------------"
if [ -f "./target/release/native-bench" ]; then
    ./target/release/native-bench
elif [ -f "../target/release/native-bench" ]; then
    ../target/release/native-bench
else
    echo "❌ Native benchmark not found. Run from workspace root:"
    echo "   cargo build --release -p goldilocks-wasm-benchmark --bin native-bench"
    echo "   ./target/release/native-bench"
fi
