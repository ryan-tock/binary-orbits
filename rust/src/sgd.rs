//! Stochastic multistart optimizer for orbital fits.
//!
//! The orbital-fit loss surface has many local minima corresponding to
//! period harmonics (fitting 2 cycles looks OK, fitting 3 looks OK, etc.).
//! Pure local gradient descent from any one seed routinely ends up in
//! the wrong harmonic. We counter that by launching many independent
//! runs, each from a random uniform point in the bounds, and doing a
//! short noisy-gradient descent from each. The "stochastic" in the name
//! is the noise we inject into gradient steps (escapes shallow basins)
//! plus the random restarts (covers the space).
//!
//! Compared to DE, this doesn't maintain a population that trades
//! information between candidates — each start is independent. That
//! makes it easier to parallelize later (not done here) but means it
//! needs more total evaluations to match DE's robustness.

use crate::{calc_loss, Params, Point};
use fastrand::Rng;

pub const NUM_FREE: usize = 6;

#[inline]
fn expand(free: &[f64; NUM_FREE]) -> [f64; 7] {
    [0.0, free[0], free[1], free[2], free[3], free[4], free[5]]
}

#[inline]
fn free_bounds(bounds: &[(f64, f64); 7]) -> [(f64, f64); NUM_FREE] {
    [bounds[1], bounds[2], bounds[3], bounds[4], bounds[5], bounds[6]]
}

#[derive(Clone, Copy, Debug)]
pub struct SgdConfig {
    /// Random starting points. Each runs an independent SGD descent;
    /// we keep the best.
    pub num_starts: usize,
    /// Descent steps per start.
    pub steps: usize,
    /// Step size as a fraction of each dimension's bound range.
    pub initial_step_frac: f64,
    /// Step-size multiplier applied each iteration.
    pub step_decay: f64,
    /// Heavy-ball momentum coefficient.
    pub momentum: f64,
    /// Gaussian noise injected into each step, as a fraction of the step.
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

fn gradient(
    points: &[Point],
    free: &[f64; NUM_FREE],
    bounds: &[(f64, f64); NUM_FREE],
    scratch: &mut Vec<[f64; 2]>,
) -> [f64; NUM_FREE] {
    let mut g = [0.0; NUM_FREE];
    for i in 0..NUM_FREE {
        let h = (1e-6_f64 * free[i].abs()).max(1e-8);
        let mut fp = *free;
        let mut fm = *free;
        fp[i] = (free[i] + h).min(bounds[i].1);
        fm[i] = (free[i] - h).max(bounds[i].0);
        let eff_h = fp[i] - fm[i];
        if eff_h <= 0.0 {
            continue;
        }
        let (lp, _) = calc_loss(points, &Params::from_slice(&expand(&fp)), scratch);
        let (lm, _) = calc_loss(points, &Params::from_slice(&expand(&fm)), scratch);
        g[i] = (lp - lm) / eff_h;
    }
    g
}

fn standard_normal(rng: &mut Rng) -> f64 {
    // Box-Muller.
    let u1 = rng.f64().max(f64::MIN_POSITIVE);
    let u2 = rng.f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn clamp_free(free: &mut [f64; NUM_FREE], bounds: &[(f64, f64); NUM_FREE]) {
    for i in 0..NUM_FREE {
        free[i] = free[i].clamp(bounds[i].0, bounds[i].1);
    }
}

/// Multistart noisy gradient descent. Returns the best (params, loss,
/// total calc_loss evaluations performed).
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

    for _start in 0..cfg.num_starts {
        // Uniform random initial point.
        let mut free: [f64; NUM_FREE] =
            std::array::from_fn(|i| fbounds[i].0 + rng.f64() * ranges[i]);
        let mut velocity = [0.0; NUM_FREE];
        let mut step = cfg.initial_step_frac;

        for _ in 0..cfg.steps {
            let g = gradient(points, &free, &fbounds, scratch);
            evals += 2 * NUM_FREE; // central differences used 12 evals
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
            clamp_free(&mut free, &fbounds);
            step *= cfg.step_decay;
        }

        let (loss, _) = calc_loss(points, &Params::from_slice(&expand(&free)), scratch);
        evals += 1;
        if loss < best_loss {
            best_loss = loss;
            best_free = free;
        }
    }

    // Splice in the LS-optimal sm for the final return.
    let mut x_final = expand(&best_free);
    let (final_loss, sm) = calc_loss(points, &Params::from_slice(&x_final), scratch);
    x_final[0] = sm;
    (x_final, final_loss, evals)
}
