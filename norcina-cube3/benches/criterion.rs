use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput, criterion_group,
    criterion_main,
};
use norcina_cube3::{
    Alg, Cube,
    search::{kociemba, solve_manhattan},
};
use rand::{SeedableRng, rngs::SmallRng};
use std::hint::black_box;

fn benchmark_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("Solve 3x3"));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for len in [1, 3, 5, 6, 7, 8, 9] {
        if len >= 5 {
            group.sample_size(10);
        }

        // Manhattan takes ~15mins for 7 moves already.
        if len < 7 {
            group.bench_with_input(BenchmarkId::new("manhattan", len), &len, |b, &len| {
                let mut rng = SmallRng::seed_from_u64(64920);
                b.iter(|| {
                    let scramble = Alg::random(len as usize, &mut rng);
                    let cube = Cube::SOLVED.mov(scramble);
                    black_box(solve_manhattan(cube))
                });
            });
        }

        group.throughput(Throughput::Elements(len));
        group.bench_with_input(BenchmarkId::new("kociemba", len), &len, |b, &len| {
            let mut rng = SmallRng::seed_from_u64(64920);
            let prune_table = kociemba::PruneTable::load_or_generate();
            b.iter(|| {
                let scramble = Alg::random(len as usize, &mut rng);
                let cube = Cube::SOLVED.mov(scramble);
                black_box(kociemba::solve_with_table(cube, &prune_table))
            });
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_solve
}

criterion_main!(benches);
