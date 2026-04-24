//! Speed benchmarks — disjoint from `tests/parity.rs` (which measures
//! accuracy). Run with `cargo bench` from the `rust/` directory.
//!
//! The "small" case (~4 points) mirrors the tightest real orbit file the
//! app has shipped with; "large" (128 points) is a synthesized stress
//! case approximating a heavy fit. Each bench reuses a scratch buffer so
//! we're measuring the *steady-state* cost a DE iteration would see.

use binary_orbits_rs::{calc_loss, calc_positions, Params, Point};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn synthetic_dataset(n: usize) -> Vec<Point> {
    // Evenly-spaced years around 2000 on a ~6yr orbit, unit weights.
    (0..n)
        .map(|k| {
            let t = 1985.0 + (k as f64) * (30.0 / n as f64);
            // Fake observations near a reasonable ellipse.
            let phase = 2.0 * std::f64::consts::PI * (t - 2000.0) / 6.0;
            Point {
                t,
                x: phase.cos(),
                y: 0.7 * phase.sin(),
                weight: 1.0,
            }
        })
        .collect()
}

fn bench_calc_loss(c: &mut Criterion) {
    let params = Params {
        sm: 0.0,
        e: 0.3,
        i: 0.6,
        node: 1.2,
        periapsis: 0.9,
        m0: 1.5,
        p: 6.0,
    };
    let mut scratch = Vec::with_capacity(128);

    let small = synthetic_dataset(4);
    c.bench_function("calc_loss/n=4", |b| {
        b.iter(|| {
            let r = calc_loss(black_box(&small), black_box(&params), &mut scratch);
            black_box(r);
        })
    });

    let medium = synthetic_dataset(32);
    c.bench_function("calc_loss/n=32", |b| {
        b.iter(|| {
            let r = calc_loss(black_box(&medium), black_box(&params), &mut scratch);
            black_box(r);
        })
    });

    let large = synthetic_dataset(128);
    c.bench_function("calc_loss/n=128", |b| {
        b.iter(|| {
            let r = calc_loss(black_box(&large), black_box(&params), &mut scratch);
            black_box(r);
        })
    });
}

fn bench_calc_positions(c: &mut Criterion) {
    let params = Params {
        sm: 1.0,
        e: 0.3,
        i: 0.6,
        node: 1.2,
        periapsis: 0.9,
        m0: 1.5,
        p: 6.0,
    };
    let mut out = Vec::with_capacity(128);

    let medium = synthetic_dataset(32);
    c.bench_function("calc_positions/n=32", |b| {
        b.iter(|| {
            calc_positions(black_box(&medium), black_box(&params), &mut out);
            black_box(&out);
        })
    });
}

criterion_group!(benches, bench_calc_loss, bench_calc_positions);
criterion_main!(benches);
