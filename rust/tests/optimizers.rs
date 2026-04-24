//! Integration tests for the refine + SGD optimizers.
//!
//! None of these lock specific parameter vectors (all three algorithms
//! are stochastic); instead they check that each finds a loss close to
//! the noise-free truth given a synthetic well-conditioned orbit.

use binary_orbits_rs::{calc_loss, de, refine, sgd, Params, Point};

fn synthetic_orbit(n: usize, truth: [f64; 7]) -> Vec<Point> {
    let params = Params::from_slice(&truth);
    let fake: Vec<Point> = (0..n)
        .map(|k| Point {
            t: 1990.0 + (k as f64) * (15.0 / n as f64),
            x: 0.0,
            y: 0.0,
            weight: 1.0,
        })
        .collect();
    let mut pos = Vec::new();
    binary_orbits_rs::calc_positions(&fake, &params, &mut pos);
    fake.into_iter()
        .zip(pos.iter())
        .map(|(mut p, q)| {
            p.x = q[0];
            p.y = q[1];
            p
        })
        .collect()
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
fn refine_polishes_a_nearby_start() {
    // Start BFGS refine from a slightly-perturbed true answer; the
    // optimizer should recover the noise floor (essentially 0 loss on
    // synthetic, noise-free data).
    let truth = [1.4, 0.3, 0.6, 1.5, 1.2, 1.0, 8.0];
    let points = synthetic_orbit(60, truth);

    let mut start = truth;
    for i in 1..7 {
        start[i] *= 1.01; // 1% perturbation
    }

    let mut scratch = Vec::new();
    let (polished, loss) =
        refine::bfgs_refine(&points, &start, &bounds(), &refine::RefineConfig::default(), &mut scratch);

    assert!(
        loss < 1e-10,
        "refine should converge near-exactly on noise-free data; got {loss:.3e}, params={polished:?}"
    );
}

#[test]
fn refine_improves_on_coarse_de_output() {
    let truth = [1.4, 0.3, 0.6, 1.5, 1.2, 1.0, 8.0];
    let points = synthetic_orbit(60, truth);

    // Run a short, coarse DE and capture its result.
    let mut cfg = de::DeConfig::default();
    cfg.seed = 7;
    cfg.max_iter = 50;
    let mut scratch = Vec::new();
    let (coarse, coarse_loss, _) = de::differential_evolution(
        &bounds(),
        |x| {
            let p = Params::from_slice(x);
            calc_loss(&points, &p, &mut scratch).0
        },
        &cfg,
    );
    let coarse_arr: [f64; 7] = coarse.as_slice().try_into().unwrap();

    let (polished, polished_loss) = refine::bfgs_refine(
        &points,
        &coarse_arr,
        &bounds(),
        &refine::RefineConfig::default(),
        &mut scratch,
    );

    assert!(
        polished_loss <= coarse_loss + 1e-12,
        "refine must never increase the loss: coarse={coarse_loss:.3e}, polished={polished_loss:.3e}, params={polished:?}"
    );
}

#[test]
fn sgd_finds_global_min_on_best_of_several_seeds() {
    // Multistart SGD is stochastic and sometimes lands in a secondary
    // basin; take the best over a few seeds to make the test robust.
    let truth = [1.2, 0.25, 0.9, 2.0, 1.1, 1.7, 7.0];
    let points = synthetic_orbit(80, truth);
    let mut scratch = Vec::new();

    let best_loss = (0..4)
        .map(|s| {
            let mut cfg = sgd::SgdConfig::default();
            cfg.seed = 0xBEEF ^ (s as u64);
            let (x, _loss, _) = sgd::fit(&points, &bounds(), &cfg, &mut scratch);
            let (polished_loss, _) = calc_loss(&points, &Params::from_slice(&x), &mut scratch);
            // Short refine pass too — SGD without polish can be off by ~1e-3
            // because of the injected noise on the last step.
            let (refined, _) = refine::bfgs_refine(
                &points,
                &x,
                &bounds(),
                &refine::RefineConfig::default(),
                &mut scratch,
            );
            let (refined_loss, _) =
                calc_loss(&points, &Params::from_slice(&refined), &mut scratch);
            polished_loss.min(refined_loss)
        })
        .fold(f64::INFINITY, f64::min);

    assert!(
        best_loss < 1e-6,
        "multistart SGD should locate the global min on noise-free data within 4 seeds; got {best_loss:.3e}"
    );
}
