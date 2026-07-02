use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use two_sets::{solve_block_stride, solve_greedy_vector};

fn bench_two_sets(c: &mut Criterion) {
    let mut group = c.benchmark_group("Two Sets Partitioning");

    // Scale input sizes to analyze time complexity scaling and cache memory layout impact
    for &size in &[100, 1000, 10000, 100000, 1000000] {
        // Benchmark Algorithm A: Zero-Allocation Stride-Based Block Partitioning
        group.bench_with_input(
            BenchmarkId::new("Algorithm_A_Block_Stride", size),
            &size,
            |b, &n| {
                b.iter(|| {
                    let res = solve_block_stride(black_box(n));
                    black_box(res);
                });
            },
        );

        // Benchmark Algorithm B: State-Based Greedy Boolean Vector Partitioning
        group.bench_with_input(
            BenchmarkId::new("Algorithm_B_Greedy_Vector", size),
            &size,
            |b, &n| {
                b.iter(|| {
                    let res = solve_greedy_vector(black_box(n));
                    black_box(res);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_two_sets);
criterion_main!(benches);
