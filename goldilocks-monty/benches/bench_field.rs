//! Benchmarks for Goldilocks Montgomery field implementation.
//!
//! To run all benchmarks:
//! ```bash
//! cargo bench --bench bench_field
//! ```
//!
//! To run benchmarks with AVX2 acceleration:
//! ```bash  
//! RUSTFLAGS="-C target-feature=+avx2" cargo bench --bench bench_field
//! ```
//!
//! To run only the AVX2 vs scalar comparison benchmarks:
//! ```bash
//! RUSTFLAGS="-C target-feature=+avx2" cargo bench --bench bench_field -- avx2
//! ```

use core::any::type_name;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use core::hint::black_box;
use p3_field::{Field, PrimeCharacteristicRing, PackedValue};
use p3_field_testing::bench_func::{
    benchmark_add_latency, benchmark_add_throughput, benchmark_inv, benchmark_iter_sum,
    benchmark_sub_latency, benchmark_sub_throughput,
};
use p3_field_testing::{
    benchmark_dot_array, benchmark_mul_latency, benchmark_mul_throughput, benchmark_sum_array,
};
use p3_goldilocks_monty::Goldilocks;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
use p3_goldilocks_monty::PackedGoldilocksMontyAVX2;

type F = Goldilocks;

fn bench_field(c: &mut Criterion) {
    let name = "GoldilocksMonty";
    const REPS: usize = 200;
    benchmark_mul_latency::<F, 100>(c, name);
    benchmark_mul_throughput::<F, 25>(c, name);
    benchmark_inv::<F>(c, name);
    benchmark_iter_sum::<F, 4, REPS>(c, name);
    benchmark_sum_array::<F, 4, REPS>(c, name);

    benchmark_dot_array::<F, 1>(c, name);
    benchmark_dot_array::<F, 2>(c, name);
    benchmark_dot_array::<F, 3>(c, name);
    benchmark_dot_array::<F, 4>(c, name);
    benchmark_dot_array::<F, 5>(c, name);
    benchmark_dot_array::<F, 6>(c, name);

    // Note that each round of throughput has 10 operations
    // So we should have 10 * more repetitions for latency tests.
    const L_REPS: usize = 10 * REPS;
    benchmark_add_latency::<F, L_REPS>(c, name);
    benchmark_add_throughput::<F, REPS>(c, name);
    benchmark_sub_latency::<F, L_REPS>(c, name);
    benchmark_sub_throughput::<F, REPS>(c, name);

    let mut rng = SmallRng::seed_from_u64(1);
    c.bench_function("7th_root_monty", |b| {
        b.iter_batched(
            || rng.random::<F>(),
            |x| x.exp_u64(10540996611094048183),
            BatchSize::SmallInput,
        )
    });
}

fn bench_packedfield(c: &mut Criterion) {
    let name = type_name::<<F as Field>::Packing>().to_string();
    // Note that each round of throughput has 10 operations
    // So we should have 10 * more repetitions for latency tests.
    const REPS: usize = 100;
    const L_REPS: usize = 10 * REPS;

    benchmark_add_latency::<<F as Field>::Packing, L_REPS>(c, &name);
    benchmark_add_throughput::<<F as Field>::Packing, REPS>(c, &name);
    benchmark_sub_latency::<<F as Field>::Packing, L_REPS>(c, &name);
    benchmark_sub_throughput::<<F as Field>::Packing, REPS>(c, &name);
    benchmark_mul_latency::<<F as Field>::Packing, L_REPS>(c, &name);
    benchmark_mul_throughput::<<F as Field>::Packing, REPS>(c, &name);
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
fn bench_avx2_operations(c: &mut Criterion) {
    const SIZE: usize = 1024;
    let mut rng = SmallRng::seed_from_u64(42);
    
    // Generate test vectors
    let scalar_a: Vec<Goldilocks> = (0..SIZE).map(|_| rng.random()).collect();
    let scalar_b: Vec<Goldilocks> = (0..SIZE).map(|_| rng.random()).collect();
    
    // Convert to packed vectors (4 elements per AVX2 vector)
    let packed_a: Vec<PackedGoldilocksMontyAVX2> = scalar_a
        .chunks_exact(4)
        .map(|chunk| PackedGoldilocksMontyAVX2([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    let packed_b: Vec<PackedGoldilocksMontyAVX2> = scalar_b
        .chunks_exact(4)
        .map(|chunk| PackedGoldilocksMontyAVX2([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    // Benchmark scalar addition
    c.bench_function("scalar_add_array", |b| {
        b.iter(|| {
            let mut result = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                result.push(black_box(scalar_a[i] + scalar_b[i]));
            }
            result
        })
    });

    // Benchmark AVX2 addition
    c.bench_function("avx2_add_array", |b| {
        b.iter(|| {
            let mut result = Vec::with_capacity(SIZE / 4);
            for i in 0..(SIZE / 4) {
                result.push(black_box(packed_a[i] + packed_b[i]));
            }
            result
        })
    });

    // Benchmark scalar multiplication
    c.bench_function("scalar_mul_array", |b| {
        b.iter(|| {
            let mut result = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                result.push(black_box(scalar_a[i] * scalar_b[i]));
            }
            result
        })
    });

    // Benchmark AVX2 multiplication
    c.bench_function("avx2_mul_array", |b| {
        b.iter(|| {
            let mut result = Vec::with_capacity(SIZE / 4);
            for i in 0..(SIZE / 4) {
                result.push(black_box(packed_a[i] * packed_b[i]));
            }
            result
        })
    });

    // Benchmark scalar sum
    c.bench_function("scalar_sum", |b| {
        b.iter(|| {
            let mut sum = Goldilocks::ZERO;
            for &val in &scalar_a {
                sum += black_box(val);
            }
            sum
        })
    });

    // Benchmark AVX2 sum
    c.bench_function("avx2_sum", |b| {
        b.iter(|| {
            let mut sum = PackedGoldilocksMontyAVX2::ZERO;
            for &val in &packed_a {
                sum += black_box(val);
            }
            // Sum the components of the packed result
            sum.as_slice().iter().fold(Goldilocks::ZERO, |acc, &x| acc + x)
        })
    });

    // Benchmark dot product scalar
    c.bench_function("scalar_dot_product", |b| {
        b.iter(|| {
            let mut result = Goldilocks::ZERO;
            for i in 0..SIZE {
                result += black_box(scalar_a[i] * scalar_b[i]);
            }
            result
        })
    });

    // Benchmark dot product AVX2
    c.bench_function("avx2_dot_product", |b| {
        b.iter(|| {
            let mut sum = PackedGoldilocksMontyAVX2::ZERO;
            for i in 0..(SIZE / 4) {
                sum += black_box(packed_a[i] * packed_b[i]);
            }
            // Sum the components of the packed result
            sum.as_slice().iter().fold(Goldilocks::ZERO, |acc, &x| acc + x)
        })
    });
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
))]
criterion_group!(
    goldilocks_monty_arithmetic, 
    bench_field, 
    bench_packedfield, 
    bench_avx2_operations
);

#[cfg(not(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(target_feature = "avx512f")
)))]
criterion_group!(goldilocks_monty_arithmetic, bench_field, bench_packedfield);

criterion_main!(goldilocks_monty_arithmetic);