//! DE convergence tests. The optimizer is stochastic so we don't lock a
//! specific parameter vector; we lock the converged loss to an upper bound
//! that reflects "found a reasonable basin".

use binary_orbits_rs::{calc_loss, de, Params, Point};

fn synthetic_orbit(n: usize) -> (Vec<Point>, [f64; 7]) {
    // Make a well-conditioned synthetic orbit we can round-trip through DE.
    let truth = [1.3, 0.35, 0.7, 2.1, 1.5, 1.0, 7.0];
    let params = Params::from_slice(&truth);
    let mut positions = Vec::new();
    let fake_points: Vec<Point> = (0..n)
        .map(|k| Point {
            t: 1990.0 + (k as f64) * (15.0 / n as f64),
            x: 0.0,
            y: 0.0,
            weight: 1.0,
        })
        .collect();
    binary_orbits_rs::calc_positions(&fake_points, &params, &mut positions);
    let points: Vec<Point> = fake_points
        .into_iter()
        .zip(positions.iter())
        .map(|(mut p, pos)| {
            p.x = pos[0];
            p.y = pos[1];
            p
        })
        .collect();
    (points, truth)
}

fn bounds() -> [(f64, f64); 7] {
    [
        (0.0, 0.0),
        (0.0, 0.95),
        (0.0, std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        (2.0, 40.0),
    ]
}

#[test]
fn de_recovers_synthetic_orbit_with_enough_tries() {
    // Given a dataset generated from a known-true parameter vector, DE
    // should find a loss close to zero on most seeds. Best-of-N loop here
    // keeps the test robust against the occasional unlucky run — we just
    // want to confirm the optimizer is actually hunting the global min,
    // not stuck in some trivial basin.
    let (points, _truth) = synthetic_orbit(50);

    let best_loss = (0..5)
        .map(|s| {
            let mut scratch: Vec<[f64; 2]> = Vec::new();
            let mut cfg = de::DeConfig::default();
            cfg.seed = 0xA5A5_5A5A_A5A5_5A5A ^ (s as u64);
            let (_best, loss, _) = de::differential_evolution(
                &bounds(),
                |x| {
                    let params = Params::from_slice(x);
                    calc_loss(&points, &params, &mut scratch).0
                },
                &cfg,
            );
            loss
        })
        .fold(f64::INFINITY, f64::min);

    assert!(
        best_loss < 1e-3,
        "DE should find a near-zero loss on a noise-free synthetic orbit in at most 5 seeds; best was {best_loss:.3e}"
    );
}

#[test]
fn de_convergence_is_reproducible_for_fixed_seed() {
    let (points, _) = synthetic_orbit(20);
    let mut cfg = de::DeConfig::default();
    cfg.seed = 42;

    let run = || {
        let mut local_scratch: Vec<[f64; 2]> = Vec::new();
        de::differential_evolution(
            &bounds(),
            |x| {
                let params = Params::from_slice(x);
                calc_loss(&points, &params, &mut local_scratch).0
            },
            &cfg,
        )
    };

    let (best_a, loss_a, _) = run();
    let (best_b, loss_b, _) = run();

    assert_eq!(best_a, best_b, "same seed should produce identical runs");
    assert_eq!(loss_a, loss_b);
}
