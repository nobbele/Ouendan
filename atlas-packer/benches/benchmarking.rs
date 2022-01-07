use atlas_packer::PackSolver;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn solve_n_random(count: u32) {
    use rand::RngCore;

    let seed: u64 = rand::random();
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);

    let mut rects = Vec::new();
    for _ in 0..count {
        rects.push(black_box(mint::Vector2 {
            x: rng.next_u32() / count,
            y: rng.next_u32() / count,
        }));
    }

    let solver = PackSolver::new(&rects);
    black_box(solver.solve());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("1 rectangle", |b| b.iter(|| solve_n_random(black_box(1))));
    c.bench_function("2 rectangle", |b| b.iter(|| solve_n_random(black_box(2))));
    c.bench_function("10 rectangle", |b| b.iter(|| solve_n_random(black_box(10))));
    c.bench_function("20 rectangle", |b| b.iter(|| solve_n_random(black_box(20))));
    c.bench_function("100 rectangle", |b| {
        b.iter(|| solve_n_random(black_box(100)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
