//! BFGS polish on an already-good orbit fit. Closes the ~0.3% loss gap
//! between our Rust DE (which exits at its convergence criterion) and
//! scipy DE (which runs L-BFGS-B as a polish step).
//!
//! Gradients come from central differences — 12 calc_loss calls each,
//! cheap at our vectorized speeds. Bounds are enforced by clipping
//! during the Armijo line search.

use crate::free_params::{clamp, compress, expand, free_bounds, gradient, NUM_FREE};
use crate::{calc_loss, Params, Point};

#[derive(Clone, Copy, Debug)]
pub struct RefineConfig {
    pub max_iter: usize,
    pub grad_tol: f64,
    pub rel_tol: f64,
}

impl Default for RefineConfig {
    fn default() -> Self {
        Self {
            max_iter: 200,
            grad_tol: 1e-10,
            rel_tol: 1e-12,
        }
    }
}

/// BFGS with backtracking Armijo line search, starting from `x_init`.
/// `hinv` (inverse-Hessian approx) is kept as a flat [NUM_FREE²] array.
pub fn bfgs_refine(
    points: &[Point],
    x_init: &[f64; 7],
    bounds: &[(f64, f64); 7],
    cfg: &RefineConfig,
    scratch: &mut Vec<[f64; 2]>,
) -> ([f64; 7], f64) {
    let fbounds = free_bounds(bounds);
    let mut free = compress(x_init);
    let (mut loss, _) = calc_loss(points, &Params::from_slice(&expand(&free)), scratch);
    let mut g = gradient(points, &free, &fbounds, scratch);
    let mut hinv = identity();

    for _ in 0..cfg.max_iter {
        let mut dir = hinv_mul_neg(&hinv, &g);

        // If BFGS gave a non-descent direction, fall back to steepest
        // descent and reset H⁻¹.
        let mut g_dot_d: f64 = g.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        if g_dot_d >= 0.0 {
            hinv = identity();
            for i in 0..NUM_FREE {
                dir[i] = -g[i];
            }
            g_dot_d = g.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        }

        // Armijo backtracking.
        let mut step = 1.0;
        let mut new_free = free;
        let mut new_loss = loss;
        let mut accepted = false;
        for _ in 0..30 {
            for i in 0..NUM_FREE {
                new_free[i] = free[i] + step * dir[i];
            }
            clamp(&mut new_free, &fbounds);
            let (tl, _) = calc_loss(points, &Params::from_slice(&expand(&new_free)), scratch);
            if tl <= loss + 1e-4 * step * g_dot_d {
                new_loss = tl;
                accepted = true;
                break;
            }
            step *= 0.5;
        }
        if !accepted {
            break;
        }

        let new_g = gradient(points, &new_free, &fbounds, scratch);
        bfgs_update(&mut hinv, &free, &new_free, &g, &new_g);

        let rel_change = (loss - new_loss).abs() / loss.abs().max(1e-300);
        free = new_free;
        g = new_g;
        loss = new_loss;

        let gnorm: f64 = g.iter().map(|x| x * x).sum::<f64>().sqrt();
        if gnorm < cfg.grad_tol || rel_change < cfg.rel_tol {
            break;
        }
    }

    // Splice in the LS-optimal sm for the final return.
    let mut x_final = expand(&free);
    let (final_loss, sm) = calc_loss(points, &Params::from_slice(&x_final), scratch);
    x_final[0] = sm;
    (x_final, final_loss)
}

fn identity() -> [f64; NUM_FREE * NUM_FREE] {
    let mut m = [0.0; NUM_FREE * NUM_FREE];
    for i in 0..NUM_FREE {
        m[i * NUM_FREE + i] = 1.0;
    }
    m
}

fn hinv_mul_neg(hinv: &[f64; NUM_FREE * NUM_FREE], g: &[f64; NUM_FREE]) -> [f64; NUM_FREE] {
    let mut dir = [0.0; NUM_FREE];
    for i in 0..NUM_FREE {
        for j in 0..NUM_FREE {
            dir[i] -= hinv[i * NUM_FREE + j] * g[j];
        }
    }
    dir
}

fn bfgs_update(
    hinv: &mut [f64; NUM_FREE * NUM_FREE],
    free: &[f64; NUM_FREE],
    new_free: &[f64; NUM_FREE],
    g: &[f64; NUM_FREE],
    new_g: &[f64; NUM_FREE],
) {
    let mut s = [0.0; NUM_FREE];
    let mut y = [0.0; NUM_FREE];
    for i in 0..NUM_FREE {
        s[i] = new_free[i] - free[i];
        y[i] = new_g[i] - g[i];
    }
    let sy: f64 = s.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    if sy <= 1e-10 {
        return;
    }
    let rho = 1.0 / sy;
    let mut hy = [0.0; NUM_FREE];
    for i in 0..NUM_FREE {
        for j in 0..NUM_FREE {
            hy[i] += hinv[i * NUM_FREE + j] * y[j];
        }
    }
    let yhy: f64 = y.iter().zip(hy.iter()).map(|(a, b)| a * b).sum();
    for i in 0..NUM_FREE {
        for j in 0..NUM_FREE {
            hinv[i * NUM_FREE + j] += (1.0 + rho * yhy) * rho * s[i] * s[j]
                - rho * (hy[i] * s[j] + s[i] * hy[j]);
        }
    }
}
