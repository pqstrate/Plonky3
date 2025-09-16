//! Benchmark comparison between Poseidon2 implementations for Goldilocks vs Goldilocks-Montgomery
//!
//! This benchmark suite compares the performance of Poseidon2 hash function between:
//! - Goldilocks field (standard implementation)
//! - Goldilocks-Montgomery field (Montgomery arithmetic)
//!
//! ## Running Benchmarks
//!
//! To run this benchmark:
//! ```bash
//! cargo bench --bench poseidon_comparison
//! ```
//!
//! With target CPU features:
//! ```bash
//! RUSTFLAGS="-C target-cpu=native" cargo bench --bench poseidon_comparison
//! ```

use core::array;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use p3_field::PrimeCharacteristicRing;
// Import both Goldilocks implementations
use p3_goldilocks::{
    Goldilocks as GoldilocksStd, HL_GOLDILOCKS_8_EXTERNAL_ROUND_CONSTANTS as HL_STD_EXT_CONSTANTS,
    HL_GOLDILOCKS_8_INTERNAL_ROUND_CONSTANTS as HL_STD_INT_CONSTANTS,
    Poseidon2GoldilocksHL as Poseidon2GoldilocksHLStd,
};
use p3_goldilocks_monty::{
    Goldilocks as GoldilocksMonty,
    HL_GOLDILOCKS_MONTY_8_EXTERNAL_ROUND_CONSTANTS as HL_MONTY_EXT_CONSTANTS,
    HL_GOLDILOCKS_MONTY_8_INTERNAL_ROUND_CONSTANTS as HL_MONTY_INT_CONSTANTS,
    Poseidon2GoldilocksHL as Poseidon2GoldilocksHLMonty,
};
use p3_poseidon2::{ExternalLayerConstants, Poseidon2};
use p3_symmetric::Permutation;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const WIDTH: usize = 8;

fn bench_poseidon2_std_single(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLStd<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksStd, WIDTH>::new_from_saved_array(
            HL_STD_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksStd::from_u64(x)),
        ),
        HL_STD_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksStd::from_u64(x))
            .collect(),
    );

    // Precompute input
    let input: [GoldilocksStd; WIDTH] = array::from_fn(|_| rng.random::<GoldilocksStd>());

    c.bench_function("poseidon2_std_single", |b| {
        b.iter(|| {
            let mut input_copy = black_box(input);
            poseidon2.permute_mut(&mut input_copy);
            input_copy
        })
    });
}

fn bench_poseidon2_monty_single(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLMonty<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksMonty, WIDTH>::new_from_saved_array(
            HL_MONTY_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksMonty::from_u64(x)),
        ),
        HL_MONTY_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksMonty::from_u64(x))
            .collect(),
    );

    // Precompute input
    let input: [GoldilocksMonty; WIDTH] = array::from_fn(|_| rng.random::<GoldilocksMonty>());

    c.bench_function("poseidon2_monty_single", |b| {
        b.iter(|| {
            let mut input_copy = black_box(input);
            poseidon2.permute_mut(&mut input_copy);
            input_copy
        })
    });
}

fn bench_poseidon2_std_batch(c: &mut Criterion) {
    const BATCH_SIZE: usize = 1000;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLStd<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksStd, WIDTH>::new_from_saved_array(
            HL_STD_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksStd::from_u64(x)),
        ),
        HL_STD_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksStd::from_u64(x))
            .collect(),
    );

    // Precompute all inputs
    let inputs: Vec<[GoldilocksStd; WIDTH]> = (0..BATCH_SIZE)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksStd>()))
        .collect();

    c.bench_function("poseidon2_std_batch_1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(BATCH_SIZE);
            for &input in &inputs {
                let mut input_copy = black_box(input);
                poseidon2.permute_mut(&mut input_copy);
                results.push(input_copy);
            }
            results
        })
    });
}

fn bench_poseidon2_monty_batch(c: &mut Criterion) {
    const BATCH_SIZE: usize = 1000;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLMonty<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksMonty, WIDTH>::new_from_saved_array(
            HL_MONTY_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksMonty::from_u64(x)),
        ),
        HL_MONTY_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksMonty::from_u64(x))
            .collect(),
    );

    // Precompute all inputs
    let inputs: Vec<[GoldilocksMonty; WIDTH]> = (0..BATCH_SIZE)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksMonty>()))
        .collect();

    c.bench_function("poseidon2_monty_batch_1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(BATCH_SIZE);
            for &input in &inputs {
                let mut input_copy = black_box(input);
                poseidon2.permute_mut(&mut input_copy);
                results.push(input_copy);
            }
            results
        })
    });
}

fn bench_poseidon2_std_throughput(c: &mut Criterion) {
    const ARRAY_SIZE: usize = 10000;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLStd<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksStd, WIDTH>::new_from_saved_array(
            HL_STD_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksStd::from_u64(x)),
        ),
        HL_STD_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksStd::from_u64(x))
            .collect(),
    );

    // Precompute all inputs
    let inputs: Vec<[GoldilocksStd; WIDTH]> = (0..ARRAY_SIZE)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksStd>()))
        .collect();

    c.bench_function("poseidon2_std_throughput_10k", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(ARRAY_SIZE);
            for &input in &inputs {
                let mut input_copy = black_box(input);
                poseidon2.permute_mut(&mut input_copy);
                results.push(input_copy);
            }
            results
        })
    });
}

fn bench_poseidon2_monty_throughput(c: &mut Criterion) {
    const ARRAY_SIZE: usize = 10000;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLMonty<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksMonty, WIDTH>::new_from_saved_array(
            HL_MONTY_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksMonty::from_u64(x)),
        ),
        HL_MONTY_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksMonty::from_u64(x))
            .collect(),
    );

    // Precompute all inputs
    let inputs: Vec<[GoldilocksMonty; WIDTH]> = (0..ARRAY_SIZE)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksMonty>()))
        .collect();

    c.bench_function("poseidon2_monty_throughput_10k", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(ARRAY_SIZE);
            for &input in &inputs {
                let mut input_copy = black_box(input);
                poseidon2.permute_mut(&mut input_copy);
                results.push(input_copy);
            }
            results
        })
    });
}

fn bench_poseidon2_tree_hashing_std(c: &mut Criterion) {
    const LEAF_COUNT: usize = 1024;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLStd<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksStd, WIDTH>::new_from_saved_array(
            HL_STD_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksStd::from_u64(x)),
        ),
        HL_STD_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksStd::from_u64(x))
            .collect(),
    );

    // Precompute all leaf data
    let leaves: Vec<[GoldilocksStd; 4]> = (0..LEAF_COUNT)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksStd>()))
        .collect();

    c.bench_function("poseidon2_std_tree_hash_1024", |b| {
        b.iter(|| {
            let mut current_level = black_box(leaves.clone());

            while current_level.len() > 1 {
                let mut next_level = Vec::new();

                for chunk in current_level.chunks(2) {
                    let mut state = [GoldilocksStd::ZERO; WIDTH];

                    // Copy first leaf to positions 0-3
                    state[0..4].copy_from_slice(&chunk[0]);

                    // Copy second leaf to positions 4-7 if it exists
                    if chunk.len() > 1 {
                        state[4..8].copy_from_slice(&chunk[1]);
                    }

                    poseidon2.permute_mut(&mut state);

                    // Take first 4 elements as the hash
                    next_level.push([state[0], state[1], state[2], state[3]]);
                }

                current_level = next_level;
            }

            current_level[0]
        })
    });
}

fn bench_poseidon2_tree_hashing_monty(c: &mut Criterion) {
    const LEAF_COUNT: usize = 1024;
    let mut rng = SmallRng::seed_from_u64(42);

    let poseidon2: Poseidon2GoldilocksHLMonty<WIDTH> = Poseidon2::new(
        ExternalLayerConstants::<GoldilocksMonty, WIDTH>::new_from_saved_array(
            HL_MONTY_EXT_CONSTANTS,
            |arr| arr.map(|x| GoldilocksMonty::from_u64(x)),
        ),
        HL_MONTY_INT_CONSTANTS
            .iter()
            .map(|&x| GoldilocksMonty::from_u64(x))
            .collect(),
    );

    // Precompute all leaf data
    let leaves: Vec<[GoldilocksMonty; 4]> = (0..LEAF_COUNT)
        .map(|_| array::from_fn(|_| rng.random::<GoldilocksMonty>()))
        .collect();

    c.bench_function("poseidon2_monty_tree_hash_1024", |b| {
        b.iter(|| {
            let mut current_level = black_box(leaves.clone());

            while current_level.len() > 1 {
                let mut next_level = Vec::new();

                for chunk in current_level.chunks(2) {
                    let mut state = [GoldilocksMonty::ZERO; WIDTH];

                    // Copy first leaf to positions 0-3
                    state[0..4].copy_from_slice(&chunk[0]);

                    // Copy second leaf to positions 4-7 if it exists
                    if chunk.len() > 1 {
                        state[4..8].copy_from_slice(&chunk[1]);
                    }

                    poseidon2.permute_mut(&mut state);

                    // Take first 4 elements as the hash
                    next_level.push([state[0], state[1], state[2], state[3]]);
                }

                current_level = next_level;
            }

            current_level[0]
        })
    });
}

criterion_group!(
    poseidon_comparison,
    bench_poseidon2_std_single,
    bench_poseidon2_monty_single,
    bench_poseidon2_std_batch,
    bench_poseidon2_monty_batch,
    bench_poseidon2_std_throughput,
    bench_poseidon2_monty_throughput,
    bench_poseidon2_tree_hashing_std,
    bench_poseidon2_tree_hashing_monty
);

criterion_main!(poseidon_comparison);
