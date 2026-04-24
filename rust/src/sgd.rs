//! Stochastic multistart optimizer. The orbital loss surface has narrow
//! harmonic basins along the period dimension, so any local method can
//! get trapped; we launch N independent noisy-gradient descents from
//! random uniform starts and keep the best. A BFGS polish is usually
//! layered on top afterwards.

use crate::free_params::{clamp, expand, free_bounds, gradient, NUM_FREE};
use crate::{calc_loss, Params, Point};
use fastrand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct SgdConfig {
    pub num_starts: usize,
    pub steps: usize,
    pub initial_step_frac: f64,
    pub step_decay: f64,
    pub momentum: f64,
    pub grad_noise_frac: f64,
    pub seed: u64,
}

impl Default for SgdConfig {
    fn default() -> Self {
        Self {
            num_starts: 64,
            steps: 40,
            initial_step_frac: 0.03,
            step_decay: 0.96,
            momentum: 0.85,
            grad_noise_frac: 0.08,
            seed: 0xC0FF_EE00_5E6D_5656,
        }
    }
}

fn standard_normal(rng: &mut Rng) -> f64 {
    // Box-Muller.
    let u1 = rng.f64().max(f64::MIN_POSITIVE);
    let u2 = rng.f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Run the multistart search. Returns (best params, best loss, total
/// calc_loss evaluations performed).
pub fn fit(
    points: &[Point],
    bounds: &[(f64, f64); 7],
    cfg: &SgdConfig,
    scratch: &mut Vec<[f64; 2]>,
) -> ([f64; 7], f64, usize) {
    let fbounds = free_bounds(bounds);
    let ranges: [f64; NUM_FREE] = std::array::from_fn(|i| fbounds[i].1 - fbounds[i].0);
    let mut rng = Rng::with_seed(cfg.seed);
    let mut evals = 0usize;

    let mut best_free = [0.0; NUM_FREE];
    let mut best_loss = f64::INFINITY;

    for _ in 0..cfg.num_starts {
        let mut free: [f64; NUM_FREE] =
            std::array::from_fn(|i| fbounds[i].0 + rng.f64() * ranges[i]);
        let mut velocity = [0.0; NUM_FREE];
        let mut step = cfg.initial_step_frac;

        for _ in 0..cfg.steps {
            let g = gradient(points, &free, &fbounds, scratch);
            evals += 2 * NUM_FREE;
            let gnorm: f64 = g.iter().map(|x| x * x).sum::<f64>().sqrt();
            if gnorm < 1e-12 {
                break;
            }
            for i in 0..NUM_FREE {
                let noise = cfg.grad_noise_frac * step * ranges[i] * standard_normal(&mut rng);
                velocity[i] =
                    cfg.momentum * velocity[i] - step * ranges[i] * (g[i] / gnorm) + noise;
                free[i] += velocity[i];
            }
            clamp(&mut free, &fbounds);
            step *= cfg.step_decay;
        }

        let (loss, _) = calc_loss(points, &Params::from_slice(&expand(&free)), scratch);
        evals += 1;
        if loss < best_loss {
            best_loss = loss;
            best_free = free;
        }
    }

    let mut x_final = expand(&best_free);
    let (final_loss, sm) = calc_loss(points, &Params::from_slice(&x_final), scratch);
    x_final[0] = sm;
    (x_final, final_loss, evals)
}
